use cgmath::{BaseFloat, Deg, Matrix4, Rad};
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


pub fn vulkanperspective(fovy: f32, aspect: f32, near: f32, far: f32) -> Matrix4<f32> {
    let correction = Mat4::new(
        1.0,  0.0,       0.0, 0.0,
        // We're also flipping the Y-axis with this line's `-1.0`.
        0.0, -1.0,       0.0, 0.0,
        0.0,  0.0, 1.0 / 2.0, 0.0,
        0.0,  0.0, 1.0 / 2.0, 1.0,
    );

    correction
        * cgmath::perspective(
        Deg(fovy),
        aspect,
        near,
        far,
    )
}
