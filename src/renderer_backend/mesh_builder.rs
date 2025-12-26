extern crate glam;
extern crate wgpu;
extern crate bytemuck;

use wgpu::util::DeviceExt;

pub struct Mesh {
    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: wgpu::Buffer,
}

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
    position: glam::Vec3,
    color: glam::Vec3,
}

impl<'a> Vertex {
    pub fn get_layout() -> wgpu::VertexBufferLayout<'a> {
        const ATTRIBUTES: [wgpu::VertexAttribute; 2] = wgpu::vertex_attr_array![0 => Float32x3, 1 => Float32x3];

        wgpu::VertexBufferLayout {
            array_stride: size_of::<Self>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &ATTRIBUTES,
        }
    }
}

pub fn make_triangle(device: &wgpu::Device) -> wgpu::Buffer {
    let vertices = [
        Vertex { position: glam::Vec3::new(-0.75, -0.75, 0.0) , color: glam::Vec3::new(1.0, 0.0, 0.0) },
        Vertex { position: glam::Vec3::new( 0.75, -0.75, 0.0) , color: glam::Vec3::new(0.0, 1.0, 0.0) },
        Vertex { position: glam::Vec3::new(  0.0,  0.75, 0.0) , color: glam::Vec3::new(0.0, 0.0, 1.0) },
    ];

    let buffer_descriptor = wgpu::util::BufferInitDescriptor {
        label: Some("Triangle Vertex Buffer"),
        contents: bytemuck::cast_slice(&vertices),
        usage: wgpu::BufferUsages::VERTEX,
    };

    device.create_buffer_init(&buffer_descriptor)
}

pub fn make_quad(device: &wgpu::Device) -> Mesh {
    let vertices = [
        Vertex { position: glam::Vec3::new(-0.75, -0.75, 0.0) , color: glam::Vec3::new(1.0, 0.0, 0.0) },
        Vertex { position: glam::Vec3::new( 0.75, -0.75, 0.0) , color: glam::Vec3::new(0.0, 1.0, 0.0) },
        Vertex { position: glam::Vec3::new( 0.75,  0.75, 0.0) , color: glam::Vec3::new(0.0, 0.0, 1.0) },
        Vertex { position: glam::Vec3::new(-0.75,  0.75, 0.0) , color: glam::Vec3::new(0.0, 1.0, 1.0) },
        ];

    let mut buffer_descriptor = wgpu::util::BufferInitDescriptor {
        label: Some("Quad Vertex Buffer"),
        contents: bytemuck::cast_slice(&vertices),
        usage: wgpu::BufferUsages::VERTEX,
    };
    let vertex_buffer = device.create_buffer_init(&buffer_descriptor);

    let indices: [u16; _] = [0, 1, 2, 2, 3, 0];

    buffer_descriptor = wgpu::util::BufferInitDescriptor {
        label: Some("Quad Index Buffer"),
        contents: bytemuck::cast_slice(&indices),
        usage: wgpu::BufferUsages::INDEX,
    };
    let index_buffer = device.create_buffer_init(&buffer_descriptor);

    Mesh {
        vertex_buffer,
        index_buffer,
    }
}