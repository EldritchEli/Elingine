use vulkanalia::{vk, Device, Instance};
use vulkanalia::vk::{DeviceV1_0, HasBuilder, InstanceV1_0};
use crate::render_app::AppData;
use anyhow::{anyhow, Result};
use crate::image_util::{create_image, create_image_view};

pub unsafe fn create_framebuffers(device: &Device, data: &mut AppData) -> anyhow::Result<()> {
    data.framebuffers = data
        .swapchain_image_views
        .iter()
        .map(|i| {
            let attachments = &[*i, data.depth_image_view];
            let create_info = vk::FramebufferCreateInfo::builder()
                .render_pass(data.render_pass)
                .attachments(attachments)
                .width(data.swapchain_extent.width)
                .height(data.swapchain_extent.height)
                .layers(1);


            device.create_framebuffer(&create_info, None)
        })
        .collect::<anyhow::Result<Vec<_>, _>>()?;
    Ok(())
}


pub unsafe fn create_depth_objects(instance: &Instance, device: &Device, data: &mut AppData) -> Result<()> {
    // Image + Image Memory

    let format = get_depth_format(instance, data)?;

    let (depth_image, depth_image_memory) = create_image(
        instance,
        device,
        data,
        data.swapchain_extent.width,
        data.swapchain_extent.height,
        format,
        vk::ImageTiling::OPTIMAL,
        vk::ImageUsageFlags::DEPTH_STENCIL_ATTACHMENT,
        vk::MemoryPropertyFlags::DEVICE_LOCAL,
    )?;

    data.depth_image = depth_image;
    data.depth_image_memory = depth_image_memory;

    // Image View

    data.depth_image_view = create_image_view(device, data.depth_image, format, vk::ImageAspectFlags::DEPTH)?;

    Ok(())
}

unsafe fn get_supported_format(
    instance: &Instance,
    data: &AppData,
    candidates: &[vk::Format],
    tiling: vk::ImageTiling,
    features: vk::FormatFeatureFlags,
) -> Result<vk::Format> {
    candidates
        .iter()
        .cloned()
        .find(|f| {
            let properties = instance.get_physical_device_format_properties(
                data.physical_device,
                *f,
            );
            match tiling {
                vk::ImageTiling::LINEAR => properties.linear_tiling_features.contains(features),
                vk::ImageTiling::OPTIMAL => properties.optimal_tiling_features.contains(features),
                _ => false,
            }

        })
        .ok_or_else(|| anyhow!("Failed to find supported format!"))
}

pub unsafe fn get_depth_format(instance: &Instance, data: &AppData) -> Result<vk::Format> {
    let candidates = &[
        vk::Format::D32_SFLOAT,
        vk::Format::D32_SFLOAT_S8_UINT,
        vk::Format::D24_UNORM_S8_UINT,
    ];

    get_supported_format(
        instance,
        data,
        candidates,
        vk::ImageTiling::OPTIMAL,
        vk::FormatFeatureFlags::DEPTH_STENCIL_ATTACHMENT,
    )
}
