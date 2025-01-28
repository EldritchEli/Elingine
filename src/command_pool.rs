use vulkanalia::{vk, Device, Instance};
use vulkanalia::vk::{DeviceV1_0, HasBuilder};
use crate::queue_family_indices::QueueFamilyIndices;
use crate::render_app::AppData;

pub unsafe fn create_command_pool(
    instance: &Instance,
    device: &Device,
    data: &mut AppData,
) -> anyhow::Result<()> {

    let indices = QueueFamilyIndices::get(instance, data, data.physical_device)?;

    let info = vk::CommandPoolCreateInfo::builder()
        .flags(vk::CommandPoolCreateFlags::empty()) // Optional.
        .queue_family_index(indices.graphics);
    data.command_pool = device.create_command_pool(&info, None)?;

    Ok(())
}

pub unsafe fn create_transient_command_pool(
    instance: &Instance,
    device: &Device,
    data: &mut AppData,
) -> anyhow::Result<()> {

    let indices = QueueFamilyIndices::get(instance, data, data.physical_device)?;

    let info = vk::CommandPoolCreateInfo::builder()
        .flags(vk::CommandPoolCreateFlags::TRANSIENT) // Optional.
        .queue_family_index(indices.graphics);
    data.transient_command_pool = device.create_command_pool(&info, None)?;

    Ok(())
}