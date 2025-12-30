use super::bind_group;

pub struct UBO {
    pub buffer: wgpu::Buffer,
    pub bind_groups: Vec<wgpu::BindGroup>,
    alignment: u64,
}

impl UBO {

    pub fn new(device: &wgpu::Device, object_count: usize, layout: wgpu::BindGroupLayout) -> Self {

        let alignment = core::cmp::max(
            device.limits().min_storage_buffer_offset_alignment as u32,
            std::mem::size_of::<glam::Mat4>() as u32) as u64;

        let buffer_descriptor = wgpu::BufferDescriptor {
            label: Some("UBO"),
            size: object_count as u64 * alignment,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        };
        let buffer = device.create_buffer(&buffer_descriptor);

        // build bind groups
        let mut bind_groups: Vec<wgpu::BindGroup> = Vec::new();
        for i in 0..object_count {
            let mut builder = bind_group::Builder::new(device);
            builder.set_layout(&layout);
            builder.add_buffer(&buffer, i as u64 * alignment);
            bind_groups.push(builder.build("Matrix"));
        }

        Self { buffer, bind_groups, alignment }
    }

    pub fn upload(&mut self, i: u64, matrix: &glam::Mat4, queue: &wgpu::Queue) {
        let offset = i * self.alignment;
        let data = bytemuck::cast_slice(matrix.as_ref());
        queue.write_buffer(&self.buffer, offset, data);

    }
}