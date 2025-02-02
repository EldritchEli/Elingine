use std::mem::size_of;
use anyhow::{Result};
use cgmath::{vec2, vec3};
use vulkanalia::{vk, Device, Instance};
use vulkanalia::vk::{DeviceV1_0, HasBuilder};
use crate::render_app::AppData;
use std::ptr::copy_nonoverlapping as memcpy;
use crate::buffer_util::{copy_buffer, create_buffer};
use varlen::*;
use varlen_macro::define_varlen;
use std::collections::HashMap;
use std::fs::File;
use std::hash::{Hash, Hasher};
use std::io::BufReader;

type Vec2 = cgmath::Vector2<f32>;
type Vec3 = cgmath::Vector3<f32>;

#[repr(C)]
#[derive(Debug)]
/// texture coordinates and paths to texture file
pub struct Texture { pub tex_string : String, pub tex_coords : Vec<Vec2> }
/// color is either encoded as RGB triplets or texture coordinates and paths to texture file
#[repr(C)]
#[derive(Debug)]
pub enum Colors { RGB(Vec<Vec3>), Texture(Texture) }



 #[derive(Clone, Debug, Default)]
 pub struct VertexData {
     pub vertices: Vec<Vertex>,
     pub indices: Vec<u32>,
     pub vertex_buffer: vk::Buffer,
     pub vertex_buffer_memory: vk::DeviceMemory,
 }

pub fn load_model(data: &mut AppData) -> Result<()> {
    let mut reader = BufReader::new(File::open("src/resources/viking_room.obj")?);

    let (models,_) = tobj::load_obj_buf(
        &mut reader,
        &tobj::LoadOptions { triangulate: true, ..Default::default() },
        |_| Ok(Default::default()),
    )?;

    let mut unique_vertices = HashMap::new();

    for model in &models {
        for index in &model.mesh.indices {
            let pos_offset = (3 * index) as usize;
            let tex_coord_offset = (2 * index) as usize;
            let vertex = Vertex {
                pos: vec3(
                    model.mesh.positions[pos_offset],
                    model.mesh.positions[pos_offset + 1],
                    model.mesh.positions[pos_offset + 2],
                ),
                color: vec3(1.0, 1.0, 1.0),
                tex_coord: vec2(
                    model.mesh.texcoords[tex_coord_offset],
                    1.0 - model.mesh.texcoords[tex_coord_offset + 1],
                ),

            };


            if let Some(index) = unique_vertices.get(&vertex) {
                data.indices.push(*index as u32);
            } else {
                let index = data.vertices.len();
                unique_vertices.insert(vertex, index);
                data.vertices.push(vertex);
                data.indices.push(index as u32);
            }


        }
    }
    Ok(())
}

#[repr(C)]
#[define_varlen]
pub struct MeshData {
    #[controls_layout]
    pub s: usize,
    #[varlen_array]
    pub positions: [Vec3;*s],
    #[varlen_array]
    pub normals:  [Vec3;*s],
    pub indices: Option<Vec<u16>>,
    pub colors : Colors
}
/*

impl MeshData {
    pub fn new(vertices: Vec<Vec3>, indices: &[u16], colors: &[Colors]) -> Result<MeshData> {

        let v = vec![9];
    }

    }

*/

pub static VERTICES: [Vertex; 8] = [
    Vertex::new(vec3(-0.5, -0.5, 0.0), vec3(1.0, 0.0, 0.0), vec2(1.0, 0.0)),
    Vertex::new(vec3(0.5, -0.5, 0.0), vec3(0.0, 1.0, 0.0), vec2(0.0, 0.0)),
    Vertex::new(vec3(0.5, 0.5, 0.0), vec3(0.0, 0.0, 1.0), vec2(0.0, 1.0)),
    Vertex::new(vec3(-0.5, 0.5, 0.0), vec3(1.0, 1.0, 1.0), vec2(1.0, 1.0)),
    Vertex::new(vec3(-0.5, -0.5, -0.5), vec3(1.0, 0.0, 0.0), vec2(1.0, 0.0)),
    Vertex::new(vec3(0.5, -0.5, -0.5), vec3(0.0, 1.0, 0.0), vec2(0.0, 0.0)),
    Vertex::new(vec3(0.5, 0.5, -0.5), vec3(0.0, 0.0, 1.0), vec2(0.0, 1.0)),
    Vertex::new(vec3(-0.5, 0.5, -0.5), vec3(1.0, 1.0, 1.0), vec2(1.0, 1.0)),
];

pub const INDICES: &[u16] = &[
    0, 1, 2, 2, 3, 0,
    4, 5, 6, 6, 7, 4,
];
/*
pub fn test_mesh1() -> MeshData{ MeshData {
    positions: [vec3(-0.5, -0.5, 0.0),
        vec3(0.5, -0.5, 0.0),
        vec3(0.5, 0.5, 0.0),
        vec3(-0.5, 0.5, 0.0),
        vec3(-0.5, -0.5, -0.5),
        vec3(0.5, -0.5, -0.5),
        vec3(0.5, 0.5, -0.5),
        vec3(-0.5, 0.5, -0.5)],
    normals: None,
    indices: Some(Vec::from([0u16, 1u16, 2u16, 2u16, 3u16, 0u16,
        4u16, 5u16, 6u16, 6u16, 7u16, 4u16])),
    vertex_count: 8,
    colors: Colors::Texture(Texture {
        tex_string: "src/resources/birk.png".to_string(),
        tex_coords: Vec::from([vec2(1.0, 0.0), vec2(0.0, 0.0),
            vec2(0.0, 1.0), vec2(1.0, 1.0),
            vec2(1.0, 0.0), vec2(0.0, 0.0),
            vec2(0.0, 1.0), vec2(1.0, 1.0, )]
        ),
    })
}
}*/

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct Vertex {
    pub pos: Vec3,
    pub color:Vec3,
    pub tex_coord: Vec2,
}

impl Vertex {
    pub const fn new(pos: Vec3,
                     color: Vec3,
                     tex_coord: Vec2, ) -> Self { Self { pos, color, tex_coord}
    }

    pub fn binding_description() -> vk::VertexInputBindingDescription {
        vk::VertexInputBindingDescription::builder()
            .binding(0)
            .stride(size_of::<Vertex>() as u32)
            .input_rate(vk::VertexInputRate::VERTEX)
            .build()
    }

    pub fn attribute_descriptions() -> [vk::VertexInputAttributeDescription; 3] {
    let pos = vk::VertexInputAttributeDescription::builder()
    .binding(0)
    .location(0)
    .format(vk::Format::R32G32B32_SFLOAT)
    .offset(0)
    .build();
    let color = vk::VertexInputAttributeDescription::builder()
    .binding(0)
    .location(1)
    .format(vk::Format::R32G32B32_SFLOAT)
    .offset(size_of::<Vec3>() as u32)
    .build();
    let tex_coord = vk::VertexInputAttributeDescription::builder()
    .binding(0)
    .location(2)
    .format(vk::Format::R32G32_SFLOAT)
    .offset((size_of::<Vec3>() + size_of::<Vec3>()) as u32)
    .build();
    [pos, color, tex_coord]
    }
}


pub(crate) unsafe fn create_vertex_buffer(
    instance: &Instance,
    device: &Device,
    data: &mut AppData,
) -> Result<()> {
    let size = (size_of::<Vertex>() * data.vertices.len()/*VERTICES.len()*/) as u64;

    let (staging_buffer, staging_buffer_memory) = create_buffer(
        instance,
        device,
        data,
        size,
        vk::BufferUsageFlags::TRANSFER_SRC,
        vk::MemoryPropertyFlags::HOST_COHERENT | vk::MemoryPropertyFlags::HOST_VISIBLE,
    )?;

    let memory = device.map_memory(
        staging_buffer_memory,
        0,
        size,
        vk::MemoryMapFlags::empty(),
    )?;

    memcpy(data.vertices.as_ptr()/*VERTICES.as_ptr()*/, memory.cast(), data.vertices.len());

    device.unmap_memory(staging_buffer_memory);

    let (vertex_buffer, vertex_buffer_memory) = create_buffer(
        instance,
        device,
        data,
        size,
        vk::BufferUsageFlags::TRANSFER_DST | vk::BufferUsageFlags::VERTEX_BUFFER,
        vk::MemoryPropertyFlags::DEVICE_LOCAL,
    )?;
    copy_buffer(device, data, staging_buffer, vertex_buffer, size)?;
    device.destroy_buffer(staging_buffer, None);
    device.free_memory(staging_buffer_memory, None);


    data.vertex_buffer = vertex_buffer;
    data.vertex_buffer_memory = vertex_buffer_memory;

    Ok(())
}

pub unsafe fn create_index_buffer(
    instance: &Instance,
    device: &Device,
    data: &mut AppData,
) -> Result<()> {
    let size = (size_of::<u32>() * data.indices.len()/*INDICES.len()*/) as u64;

    let (staging_buffer, staging_buffer_memory) = create_buffer(
        instance,
        device,
        data,
        size,
        vk::BufferUsageFlags::TRANSFER_SRC,
        vk::MemoryPropertyFlags::HOST_COHERENT | vk::MemoryPropertyFlags::HOST_VISIBLE,
    )?;

    let memory = device.map_memory(
        staging_buffer_memory,
        0,
        size,
        vk::MemoryMapFlags::empty(),
    )?;

    memcpy(data.indices.as_ptr(), memory.cast(), data.indices.len());

    device.unmap_memory(staging_buffer_memory);

    let (index_buffer, index_buffer_memory) = create_buffer(
        instance,
        device,
        data,
        size,
        vk::BufferUsageFlags::TRANSFER_DST | vk::BufferUsageFlags::INDEX_BUFFER,
        vk::MemoryPropertyFlags::DEVICE_LOCAL,
    )?;

    data.index_buffer = index_buffer;
    data.index_buffer_memory = index_buffer_memory;

    copy_buffer(device, data, staging_buffer, index_buffer, size)?;

    device.destroy_buffer(staging_buffer, None);
    device.free_memory(staging_buffer_memory, None);

    Ok(())
}








impl PartialEq for Vertex {
    fn eq(&self, other: &Self) -> bool {
        self.pos == other.pos
            && self.color == other.color
            && self.tex_coord == other.tex_coord
    }
}

impl Eq for Vertex {}

impl Hash for Vertex {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.pos[0].to_bits().hash(state);
        self.pos[1].to_bits().hash(state);
        self.pos[2].to_bits().hash(state);
        self.color[0].to_bits().hash(state);
        self.color[1].to_bits().hash(state);
        self.color[2].to_bits().hash(state);
        self.tex_coord[0].to_bits().hash(state);
        self.tex_coord[1].to_bits().hash(state);
    }
}


