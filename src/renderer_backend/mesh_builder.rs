extern crate glam;
extern crate wgpu;
extern crate bytemuck;

use wgpu::util::DeviceExt;

pub struct Mesh {
    pub buffer: wgpu::Buffer,
    pub offset: u64,
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
    let indices: [u16; _] = [0, 1, 2, 2, 3, 0];
    
    let bytes_1 = bytemuck::cast_slice(&vertices);
    let bytes_2 = bytemuck::cast_slice(&indices);
    let bytes_merged = &[bytes_1, bytes_2].concat();

    let buffer_descriptor = wgpu::util::BufferInitDescriptor {
        label: Some("Quad vertex & index Buffer"),
        contents: bytes_merged,
        usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::INDEX,
    };
    let buffer = device.create_buffer_init(&buffer_descriptor);
    let offset = bytes_1.len().try_into().unwrap();

    Mesh {
        buffer,
        offset,
    }
}