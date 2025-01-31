use vulkanalia::vk;
use vulkanalia::vk::HasBuilder;

pub type Mat4 = cgmath::Matrix4<f32>;
pub type Vec3 = cgmath::Vector3<f32>;
pub type Vec2 = cgmath::Vector2<f32>;
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct UniformBufferObject {
    pub model: Mat4,
    pub view: Mat4,
    pub proj: Mat4
}



