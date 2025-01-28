use anyhow::anyhow;
use vulkanalia::{vk, Device, Entry, Instance};
use vulkanalia::loader::{LibloadingLoader, LIBRARY};
use vulkanalia::vk::{DeviceV1_0, ExtDebugUtilsExtension, Handle, HasBuilder, InstanceV1_0, KhrSurfaceExtension, KhrSwapchainExtension};
use winit::window::Window;
use vulkanalia::window as vk_window;
use crate::{MAX_FRAMES_IN_FLIGHT, VALIDATION_ENABLED};
use crate::command_buffer_util::create_command_buffers;
use crate::command_pool::{create_command_pool, create_transient_command_pool};
use crate::device_util::{create_logical_device, pick_physical_device};
use crate::framebuffer_util::create_framebuffers;
use crate::instance_util::create_instance;
use crate::pipeline_util::create_pipeline;
use crate::render_pass_util::create_render_pass;
use crate::swapchain_util::{create_swapchain, create_swapchain_image_views};
use crate::sync_util::create_sync_objects;
use crate::vertexbuffer_util::{create_index_buffer, create_vertex_buffer};

/// Our Vulkan app.
#[derive(Clone, Debug)]
pub struct App {
    pub entry: Entry,
    pub instance: Instance,
    pub data: AppData,
    pub(crate) device: Device,
    frame: usize,
    pub(crate) resized: bool,
}


impl App {
    /// Creates our Vulkan app.
    pub(crate) unsafe fn create(window: &Window) -> anyhow::Result<Self> {
        let resized = false;
        let loader = LibloadingLoader::new(LIBRARY)?;
        let entry = Entry::new(loader).map_err(|b| anyhow!("{}", b))?;
        let mut data = AppData::default();
        let instance = create_instance(window, &entry, &mut data)?;
        data.surface = vk_window::create_surface(&instance, &window, &window)?;
        pick_physical_device(&instance, &mut data)?;
        let device = create_logical_device(&entry, &instance, &mut data)?;
        create_swapchain(window, &instance, &device, &mut data)?;
        create_swapchain_image_views(&device, &mut data)?;
        create_render_pass(&instance, &device, &mut data)?;
        create_pipeline(&device, &mut data)?;
        create_framebuffers(&device, &mut data)?;
        create_command_pool(&instance, &device, &mut data)?;
        create_transient_command_pool(&instance, &device, &mut data)?;
        create_vertex_buffer(&instance, &device, &mut data)?;
        create_index_buffer(&instance, &device, &mut data)?;
        create_command_buffers(&device, &mut data)?;
        create_sync_objects(&device, &mut data)?;
        Ok(Self { entry, instance, data, device, frame: 0, resized})
    }

    unsafe fn recreate_swapchain(&mut self, window: &Window) -> anyhow::Result<()> {
        self.device.device_wait_idle()?;
        self.destroy_swapchain();
        create_swapchain(window, &self.instance, &self.device, &mut self.data)?;
        create_swapchain_image_views(&self.device, &mut self.data)?;
        create_render_pass(&self.instance, &self.device, &mut self.data)?;
        create_pipeline(&self.device, &mut self.data)?;
        create_framebuffers(&self.device, &mut self.data)?;
        create_command_buffers(&self.device, &mut self.data)?;
        self.data
            .images_in_flight
            .resize(self.data.swapchain_images.len(), vk::Fence::null());
        Ok(())
    }

    /// Renders a frame for our Vulkan app.
    pub(crate) unsafe fn render(&mut self, window: &Window) -> anyhow::Result<()> {

        self.device.wait_for_fences(
            &[self.data.in_flight_fences[self.frame]],
            true,
            u64::MAX,
        )?;

        let result = self.device.acquire_next_image_khr(
            self.data.swapchain,
            u64::MAX,
            self.data.image_available_semaphores[self.frame],
            vk::Fence::null(),
        );

        let image_index = match result {
            Ok((_, vk::SuccessCode::SUBOPTIMAL_KHR)) => return self.recreate_swapchain(window),
            Ok((image_index, _)) => image_index as usize,
            Err(vk::ErrorCode::OUT_OF_DATE_KHR) => return self.recreate_swapchain(window),
            Err(e) => return Err(anyhow!(e)),
        };


        let image_in_flight = self.data.images_in_flight[image_index];
        if !image_in_flight.is_null() {
            self.device.wait_for_fences(&[image_in_flight], true, u64::MAX)?;
        }

        self.data.images_in_flight[image_index] = self.data.in_flight_fences[self.frame];


        let wait_semaphores = &[self.data.image_available_semaphores[self.frame]];
        let wait_stages = &[vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT];
        let command_buffers = &[self.data.command_buffers[image_index]];
        let signal_semaphores = &[self.data.render_finished_semaphores[self.frame]];
        let submit_info = vk::SubmitInfo::builder()
            .wait_semaphores(wait_semaphores)
            .wait_dst_stage_mask(wait_stages)
            .command_buffers(command_buffers)
            .signal_semaphores(signal_semaphores);

        self.device.reset_fences(&[self.data.in_flight_fences[self.frame]])?;

        self.device.queue_submit(
            self.data.graphics_queue,
            &[submit_info],
            self.data.in_flight_fences[self.frame],
        )?;

        let swapchains = &[self.data.swapchain];
        let image_indices = &[image_index as u32];
        let present_info = vk::PresentInfoKHR::builder()
            .wait_semaphores(signal_semaphores)
            .swapchains(swapchains)
            .image_indices(image_indices);

        let result = self.device.queue_present_khr(self.data.present_queue, &present_info);
        let changed = result == Ok(vk::SuccessCode::SUBOPTIMAL_KHR)
            || result == Err(vk::ErrorCode::OUT_OF_DATE_KHR);

        if self.resized || changed {
            self.resized = false;
            self.recreate_swapchain(window)?;
        } else if let Err(e) = result {
            return Err(anyhow!(e));
        }
        self.device.queue_wait_idle(self.data.present_queue)?;




        self.frame = (self.frame + 1) % MAX_FRAMES_IN_FLIGHT;
        Ok(())
    }


    /// Destroys our Vulkan app.
    pub(crate) unsafe fn destroy(&mut self) {

        self.device.device_wait_idle().unwrap();
        self.destroy_swapchain();

        self.data.in_flight_fences.iter().for_each(|f| self.device.destroy_fence(*f, None));
        self.data.render_finished_semaphores.iter().for_each(|s| self.device.destroy_semaphore(*s, None));
        self.data.image_available_semaphores.iter().for_each(|s| self.device.destroy_semaphore(*s, None));
        self.device.free_memory(self.data.vertex_buffer_memory, None);
        self.device.destroy_buffer(self.data.vertex_buffer, None);
        self.device.free_memory(self.data.index_buffer_memory, None);
        self.device.destroy_buffer(self.data.index_buffer, None);


        self.device.destroy_command_pool(self.data.command_pool, None);
        self.device.destroy_command_pool(self.data.transient_command_pool, None);

        self.device.destroy_device(None);
        self.instance.destroy_surface_khr(self.data.surface, None);
        if VALIDATION_ENABLED {
            self.instance.destroy_debug_utils_messenger_ext(self.data.messenger, None);
        }
        self.instance.destroy_instance(None);
    }

    unsafe fn destroy_swapchain(&mut self) {
        self.data.framebuffers
            .iter()
            .for_each(|f| self.device.destroy_framebuffer(*f, None));
        self.device.free_command_buffers(self.data.command_pool, &self.data.command_buffers);
        self.device.destroy_pipeline(self.data.pipeline, None);
        self.device.destroy_pipeline_layout(self.data.pipeline_layout, None);
        self.device.destroy_render_pass(self.data.render_pass, None);
        self.data.swapchain_image_views
            .iter()
            .for_each(|v| self.device.destroy_image_view(*v, None));
        self.device.destroy_swapchain_khr(self.data.swapchain, None);
    }
}

/// The Vulkan handles and associated properties used by our Vulkan app.
#[derive(Clone, Debug, Default)]
pub struct AppData {
   pub surface: vk::SurfaceKHR,
    pub messenger: vk::DebugUtilsMessengerEXT,
    pub physical_device: vk::PhysicalDevice,
    pub graphics_queue: vk::Queue,
    pub present_queue: vk::Queue,

    pub swapchain_format: vk::Format,
    pub swapchain_extent: vk::Extent2D,
    pub swapchain: vk::SwapchainKHR,
    pub swapchain_images: Vec<vk::Image>,
    pub swapchain_image_views: Vec<vk::ImageView>,

    pub render_pass: vk::RenderPass,
    pub pipeline_layout: vk::PipelineLayout,
    pub pipeline: vk::Pipeline,

    pub framebuffers: Vec<vk::Framebuffer>,

    pub command_pool: vk::CommandPool,
    pub transient_command_pool: vk::CommandPool,
    pub command_buffers: Vec<vk::CommandBuffer>,
    pub transient_command_buffers: Vec<vk::CommandBuffer>,

    pub image_available_semaphores: Vec<vk::Semaphore>,
    pub render_finished_semaphores: Vec<vk::Semaphore>,

    pub in_flight_fences: Vec<vk::Fence>,
    pub images_in_flight: Vec<vk::Fence>,

    pub vertex_buffer: vk::Buffer,
    pub vertex_buffer_memory: vk::DeviceMemory,
    pub index_buffer: vk::Buffer,
    pub index_buffer_memory: vk::DeviceMemory,
}
