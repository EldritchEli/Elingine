#![allow(
    dead_code,
    unused_variables,
    clippy::too_many_arguments,
    clippy::unnecessary_wraps
)]

mod swapchain_util;
mod device_util;
mod render_app;
mod pipeline_util;
mod instance_util;
mod queue_family_indices;
mod command_buffer_util;
mod render_pass_util;
mod sync_util;
mod command_pool;
mod framebuffer_util;
mod shader_module_util;
mod vertexbuffer_util;
mod buffer_util;

use anyhow::{Result};
use winit::dpi::LogicalSize;
use winit::event::{Event, WindowEvent};
use winit::event_loop::EventLoop;
use winit::window::{WindowBuilder};

use vulkanalia::prelude::v1_0::*;
use vulkanalia::Version;


use crate::queue_family_indices::QueueFamilyIndices;
use crate::render_app::{App, AppData};

type Vec2 = cgmath::Vector2<f32>;
type Vec3 = cgmath::Vector3<f32>;

const PORTABILITY_MACOS_VERSION: Version = Version::new(1, 3, 216);
const VALIDATION_ENABLED: bool =
    cfg!(debug_assertions);
const VALIDATION_LAYER: vk::ExtensionName =
    vk::ExtensionName::from_bytes(b"VK_LAYER_KHRONOS_validation");
const DEVICE_EXTENSIONS: &[vk::ExtensionName] = &[vk::KHR_SWAPCHAIN_EXTENSION.name];
const MAX_FRAMES_IN_FLIGHT: usize = 2;

fn main() -> Result<()> {
    pretty_env_logger::init();
    // Window
    let event_loop = EventLoop::new()?;
    let window = WindowBuilder::new()
        .with_title("Vulkan Tutorial (Rust)")
        .with_inner_size(LogicalSize::new(1024, 768))
        .build(&event_loop)?;

    // App

    let mut app = unsafe { App::create(&window)? };
    let mut minimized = false; //window minimized
    event_loop.run(move |event, elwt| {
        match event {
            // Request a redraw when all events were processed.
            Event::AboutToWait => window.request_redraw(),
            Event::WindowEvent { event: WindowEvent::Resized(_), .. } => app.resized = true,
            Event::WindowEvent { event, .. } => match event {
                // Render a frame if our Vulkan app is not being destroyed.
                WindowEvent::RedrawRequested if !elwt.exiting() && !minimized => unsafe { app.render(&window) }.unwrap(),
                WindowEvent::Resized(size) => {
                    if size.width == 0 || size.height == 0 {
                        minimized = true;
                    } else {
                        minimized = false;
                        app.resized = true;
                    }
                }
                // Destroy our Vulkan app.
                WindowEvent::CloseRequested => {
                    elwt.exit();
                    unsafe { app.device.device_wait_idle().unwrap(); }
                    unsafe { app.destroy(); }
                }
                _ => {}
            }
            _ => {}
        }
    })?;
    Ok(())
}