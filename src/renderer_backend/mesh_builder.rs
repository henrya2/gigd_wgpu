extern crate glam;
extern crate wgpu;

use std::slice;
use wgpu::util::DeviceExt;

#[repr(C)]
pub struct Vertex {
    position: glam::Vec3,
    color: glam::Vec3,
}

impl Vertex {
    pub fn get_layout() -> wgpu::VertexBufferLayout<'static> {
        const ATTRIBUTES: [wgpu::VertexAttribute; 2] = wgpu::vertex_attr_array![0 => Float32x3, 1 => Float32x3];

        wgpu::VertexBufferLayout {
            array_stride: size_of::<Self>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &ATTRIBUTES,
        }
    }
}

unsafe fn any_as_u8_slice<T: Sized>(p: &T) -> &[u8] {
    slice::from_raw_parts(
        (p as *const T) as *const u8,
        size_of::<T>(),
    )
}

pub fn make_triangle(device: &wgpu::Device) -> wgpu::Buffer {
    let vertices = [
        Vertex { position: glam::Vec3::new(-0.75, -0.75, 0.0) , color: glam::Vec3::new(1.0, 0.0, 0.0) },
        Vertex { position: glam::Vec3::new( 0.75, -0.75, 0.0) , color: glam::Vec3::new(0.0, 1.0, 0.0) },
        Vertex { position: glam::Vec3::new(  0.0,  0.75, 0.0) , color: glam::Vec3::new(0.0, 0.0, 1.0) },
    ];
    let bytes: &[u8] = unsafe { any_as_u8_slice(&vertices) };

    let buffer_descriptor = wgpu::util::BufferInitDescriptor {
        label: Some("Triangle Vertex Buffer"),
        contents: bytes,
        usage: wgpu::BufferUsages::VERTEX,
    };

    device.create_buffer_init(&buffer_descriptor)
}