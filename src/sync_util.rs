use vulkanalia::{vk, Device};
use vulkanalia::vk::{DeviceV1_0, Handle, HasBuilder};
use crate::MAX_FRAMES_IN_FLIGHT;
use crate::render_app::AppData;

pub unsafe fn create_sync_objects(device: &Device, data: &mut AppData) -> anyhow::Result<()> {
    let semaphore_info = vk::SemaphoreCreateInfo::builder();
    let fence_info = vk::FenceCreateInfo::builder()
        .flags(vk::FenceCreateFlags::SIGNALED);

    for _ in 0..MAX_FRAMES_IN_FLIGHT {
        data.image_available_semaphores
            .push(device.create_semaphore(&semaphore_info, None)?);
        data.render_finished_semaphores
            .push(device.create_semaphore(&semaphore_info, None)?);

        data.in_flight_fences.push(device.create_fence(&fence_info, None)?);
    }
    data.images_in_flight = data.swapchain_images
        .iter()
        .map(|_| vk::Fence::null())
        .collect();

    Ok(())
}