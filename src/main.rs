use glfw::{Action, Key, fail_on_errors, ClientApiHint};

mod renderer_backend;
mod model;

use renderer_backend::{pipeline, bind_group_layout, material::Material, mesh_builder, ubo::UBO};

use model::game_objects::Object;

struct World {
    quads: Vec<Object>,
    tris: Vec<Object>,
}

impl World {
    const ROTATION_SPEED: f32 = 24.0;
    fn new() -> Self {
        World { quads: Vec::new(), tris: Vec::new() }
    }

    fn update(&mut self, dt: f32) {

        let update_obj = |obj: &mut Object| {
            obj.angle = obj.angle + Self::ROTATION_SPEED * dt;
            if obj.angle > 360.0 {
                obj.angle -= 360.0;
            }

            if !obj.velocity.cmpeq(glam::Vec3::ZERO).all() {
                obj.position = obj.position + obj.velocity * dt;

                if obj.position.cmpge(glam::Vec3::ONE).any() ||
                    obj.position.cmple(glam::Vec3::NEG_ONE).any() {
                    obj.velocity = -obj.velocity;
                }
            }
        };

        for tri in &mut self.tris {
            update_obj(tri);
        }

        for quad in &mut self.quads {
            update_obj(quad);
        }
    }
}

struct State<'a> {
    instance: wgpu::Instance,
    surface: wgpu::Surface<'a>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    size: (i32, i32),
    window: &'a mut glfw::Window,
    render_pipeline: wgpu::RenderPipeline,
    triangle_mesh: wgpu::Buffer,
    quad_mesh: mesh_builder::Mesh,
    triangle_material: Material,
    quad_material: Material,
    ubo: Option<UBO>,
}

impl<'a> State<'a> {
    async fn new(window: &'a mut glfw::Window) -> Self {

        let size = window.get_framebuffer_size();

        let instance_descriptor = wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(), ..Default::default()
        };

        let instance = wgpu::Instance::new(&instance_descriptor);
        let surface = instance.create_surface(window.render_context()).unwrap();

        let adapter_descriptor = wgpu::RequestAdapterOptionsBase {
            power_preference: wgpu::PowerPreference::default(),
            compatible_surface: Some(&surface),
            force_fallback_adapter: false,
        };
        let adapter = instance.request_adapter(&adapter_descriptor)
            .await.unwrap();

        #[cfg(debug_assertions)]
        {
            let backend = adapter.get_info().backend;
            println!("Using backend: {:?}", backend);
        }

        let device_descriptor = wgpu::DeviceDescriptor {
            required_features: wgpu::Features::empty(),
            required_limits: wgpu::Limits::default(),
            label: Some("Device"),
            ..Default::default()
        };
        let (device, queue) = adapter
            .request_device(&device_descriptor)
            .await.unwrap();


        let surface_capabilities = surface.get_capabilities(&adapter);
        let surface_format = surface_capabilities
            .formats
            .iter()
            .copied()
            .filter(|f | f.is_srgb())
            .next()
            .unwrap_or(surface_capabilities.formats[0]);
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.0 as u32,
            height: size.1 as u32,
            present_mode: surface_capabilities.present_modes[0],
            alpha_mode: surface_capabilities.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2
        };
        surface.configure(&device, &config);
        
        let triangle_buffer = mesh_builder::make_triangle(&device);

        let quad_mesh = mesh_builder::make_quad(&device);

        let material_bind_group_layout = {
            let mut builder = bind_group_layout::Builder::new(&device);
            builder.add_material();
            builder.build("Material Bind Group Layout")
        };

        let ubo_bind_group_layout = {
            let mut builder = bind_group_layout::Builder::new(&device);
            builder.add_ubo();
            builder.build("UBO Bind Group Layout")
        };

        let render_pipeline = {
            let mut builder = pipeline::Builder::new(&device);
            builder.set_shader_module("shaders/shader.wgsl", "vs_main", "fs_main")
            .set_pixel_format(config.format)
            .add_vertex_buffer_layout(mesh_builder::Vertex::get_layout())
            .add_bind_group_layout(&material_bind_group_layout)
            .add_bind_group_layout(&ubo_bind_group_layout);
            builder.build("Render Pipeline")
        };
        let triangle_materail = Material::new("../img/winry.jpg", &device, &queue, "Triangle Material", &material_bind_group_layout);
        let quad_materail = Material::new("../img/satin.jpg", &device, &queue, "Quad Material", &material_bind_group_layout);

        Self {
            instance,
            window,
            surface,
            device,
            queue,
            config,
            size,
            render_pipeline,
            triangle_mesh: triangle_buffer,
            quad_mesh,
            triangle_material: triangle_materail,
            quad_material: quad_materail,
            ubo: None,
        }
    }

    fn render(&mut self, quads: &Vec<Object>, tris: &Vec<Object>) -> Result<(), wgpu::SurfaceError> {

        //self.device.poll(wgpu::Maintain::Wait);

        // Upload
        let mut offset: u64 = 0;
        for (i, value) in quads.iter().enumerate() {
            // Be careful here, glam uses column major matrix， ABv, B applies first, then A. So rotation first , then translation.
            let matrix = glam::Mat4::from_translation(value.position) *
                glam::Mat4::from_axis_angle(glam::Vec3::new(0.0, 0.0, 1.0), value.angle.to_radians());
            self.ubo.as_mut().unwrap().upload(offset + i as u64, &matrix, &self.queue);
        }

        offset = quads.len() as u64;
        for (i, value) in tris.iter().enumerate() {
            // Be careful here, glam uses column major matrix， ABv, B applies first, then A. So rotation first , then translation.
            let matrix = glam::Mat4::from_translation(value.position) *
                glam::Mat4::from_axis_angle(glam::Vec3::new(0.0, 0.0, 1.0), value.angle.to_radians());
            self.ubo.as_mut().unwrap().upload(offset + i as u64, &matrix, &self.queue);
        }

        let drawable = self.surface.get_current_texture()?;
        let image_view_descriptor = wgpu::TextureViewDescriptor::default();
        let image_view = drawable.texture.create_view(&image_view_descriptor);

        let command_encoder_descriptor = wgpu::CommandEncoderDescriptor {
            label: Some("Render Encoder"),
        };
        let mut command_encoder = self.device.create_command_encoder(&command_encoder_descriptor);

        let color_attachment = wgpu::RenderPassColorAttachment {
            view: &image_view,
            depth_slice: None,
            resolve_target: None,
            ops: wgpu::Operations {
                load: wgpu::LoadOp::Clear(wgpu::Color {
                    r: 0.75,
                    g: 0.5,
                    b: 0.25,
                    a: 1.0
                }),
                store: wgpu::StoreOp::Store,
            },
        };

        let render_pass_descriptor = wgpu::RenderPassDescriptor {
            label: Some("Renderpass"),
            color_attachments: &[Some(color_attachment)],
            depth_stencil_attachment: None,
            occlusion_query_set: None,
            timestamp_writes: None,
            ..Default::default()
        };

        {
            let mut renderpass = command_encoder.begin_render_pass(&render_pass_descriptor);
            renderpass.set_pipeline(&self.render_pipeline);

            renderpass.set_bind_group(0, &self.quad_material.bind_group, &[]);
            renderpass.set_vertex_buffer(0, self.quad_mesh.buffer.slice(..self.quad_mesh.offset));
            renderpass.set_index_buffer(self.quad_mesh.buffer.slice(self.quad_mesh.offset..), wgpu::IndexFormat::Uint16);

            let mut offset:usize = 0;
            for i in 0..quads.len() {
                renderpass.set_bind_group(
                    1,
                    &(self.ubo.as_ref().unwrap()).bind_groups[offset + i],
                    &[]
                );
                renderpass.draw_indexed(0..6, 0, 0..1);
            }

            {
                renderpass.set_bind_group(0, &self.triangle_material.bind_group, &[]);
                renderpass.set_vertex_buffer(0, self.triangle_mesh.slice(..));
                offset = quads.len();
                for i in 0..tris.len() {
                    renderpass.set_bind_group(
                        1,
                        &(self.ubo.as_ref().unwrap()).bind_groups[offset + i],
                        &[]
                    );
                    renderpass.draw(0..3, 0..1);
                }
            }
        }

        self.queue.submit(std::iter::once(command_encoder.finish()));

        drawable.present();

        Ok(())

    }

    fn resize(&mut self, new_size: (i32, i32)) {
        if new_size.0 > 0 && new_size.1 > 0 {
            self.size = new_size;
            self.config.width = new_size.0 as u32;
            self.config.height = new_size.1 as u32;
            self.surface.configure(&self.device, &self.config);
        }
    }

    fn update_surface(&mut self) {
        self.surface = self.instance.create_surface(self.window.render_context()).unwrap();
    }

    pub fn build_ubos_for_objects(&mut self, object_count: usize) {
        let ubo_bind_group_layout = {
            let mut builder = bind_group_layout::Builder::new(&self.device);
            builder.add_ubo();
            builder.build("UBO Bind Group Layout")
        };
        self.ubo = Some(UBO::new(&self.device, object_count, ubo_bind_group_layout));
    }
}

async fn run() {
    let mut glfw = glfw::init(fail_on_errors!()).unwrap();

    glfw.window_hint(glfw::WindowHint::ClientApi(ClientApiHint::NoApi));
    glfw.window_hint(glfw::WindowHint::Resizable(true));

    let (mut window, events) = glfw.create_window(800, 600, "It's WGPU time", glfw::WindowMode::Windowed).unwrap();

    let mut state = State::new(&mut window).await;

    state.window.set_framebuffer_size_polling(true);
    state.window.set_key_polling(true);
    state.window.set_mouse_button_polling(true);
    state.window.set_pos_polling(true);


    // Build world
    let mut world = World::new();
    world.tris.push(Object {
        position: glam::Vec3::new(-0.5, 0.0, 0.0),
        angle: 0.0,
        velocity: glam::vec3(0.1, 0.06, 0.0)
    });
    world.quads.push(Object {
        position: glam::Vec3::new(0.9, 0.0, 0.0),
        angle: 0.0,
        velocity: glam::vec3(0.0, 0.0, 0.0),
    });
    state.build_ubos_for_objects(world.tris.len() + world.quads.len());

    let mut delta_time;
    let mut last_time = glfw.get_time();

    while !state.window.should_close() {
        let current_time = glfw.get_time();
        delta_time = current_time - last_time;
        last_time = current_time;

        world.update(delta_time as f32);

        glfw.poll_events();
        for event in glfw::flush_messages(&events) {
            handle_window_event(&mut state, event);
        }
        match state.render(&world.quads, &world.tris) {
            Ok(_) => {}
            Err(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated) => {
                state.update_surface();
                state.resize(state.size);
            }
            Err(e) => {
                eprintln!("Error: {:?}", e);
            }
        }
    }
}

fn handle_window_event(state: &mut State, (_time, event): (f64, glfw::WindowEvent)) {
    match event {
        glfw::WindowEvent::Key(Key::Escape, _, Action::Press, _) => {
            state.window.set_should_close(true);
        }

        glfw::WindowEvent::Pos(..) => {
            state.update_surface();
            let new_size = state.window.get_framebuffer_size();
            state.resize(new_size);
        }

        glfw::WindowEvent::FramebufferSize(_width, _height) => {
            state.update_surface();
            let new_size = state.window.get_framebuffer_size();
            state.resize(new_size);
        }
        _ => {}
    }
}

fn main() {
    pollster::block_on(run());
}