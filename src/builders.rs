use super::*;

pub struct DemoBuilder {
    demo: Demo,
}

impl DemoBuilder {
    pub fn new((width, height): (u32, u32), title: &'static str) -> DemoBuilder {
        let event_loop = EventLoop::new();
        let window = WindowBuilder::new()
            .with_inner_size(PhysicalSize { width, height })
            .with_title(title)
            .build(&event_loop)
            .unwrap();

        let instance = wgpu::Instance::new(wgpu::Backends::PRIMARY);
        let surface = unsafe { instance.create_surface(&window) };

        let adapter = block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            force_fallback_adapter: false,
            compatible_surface: Some(&surface),
        }))
        .unwrap();

        let (device, queue) = block_on(adapter.request_device(
            &wgpu::DeviceDescriptor {
                label: None,
                features: wgpu::Features::empty(),
                limits: wgpu::Limits::default(),
            },
            None,
        ))
        .unwrap();

        DemoBuilder {
            demo: Demo {
                event_loop,
                window,
                device,
                queue,
                surface,
                adapter,
                scenes: vec![],
            },
        }
    }

    pub fn scene(mut self, builder: impl Fn(SceneBuilder) -> Scene) -> DemoBuilder {
        self.demo.scenes.push(builder(SceneBuilder {
            demo_builder: &self,
            fragment_source: "",
            uniforms: &|| vec![],
        }));
        self
    }

    pub fn build(self) -> Demo {
        self.demo
    }
}

pub struct SceneBuilder<'a> {
    demo_builder: &'a DemoBuilder,
    fragment_source: &'static str,
    uniforms: &'static dyn Fn() -> Vec<u8>,
}

impl<'a> SceneBuilder<'a> {
    pub fn with_uniforms(mut self, uniforms: &'static dyn Fn() -> Vec<u8>) -> SceneBuilder<'a> {
        self.uniforms = uniforms;
        self
    }

    pub fn set_fragment_source(mut self, src: &'static str) -> SceneBuilder<'a> {
        self.fragment_source = src;
        self
    }

    pub fn build(self) -> Scene {
        let device = &self.demo_builder.demo.device;

        let vert_shader = device.create_shader_module(&wgpu::ShaderModuleDescriptor {
            label: None,
            source: wgpu::ShaderSource::SpirV(Cow::Borrowed(glsl::vertex_passthrough())),
        });

        let frag_shader = device.create_shader_module(&wgpu::ShaderModuleDescriptor {
            label: None,
            source: wgpu::ShaderSource::SpirV(Cow::Owned(glsl::compile_fragment(
                self.fragment_source,
            ))),
        });

        let uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            contents: &(self.uniforms)(),
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::UNIFORM,
        });

        let uniform_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: None,
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
            });

        let uniform_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &uniform_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: uniform_buffer.as_entire_binding(),
            }],
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[&uniform_bind_group_layout],
            push_constant_ranges: &[],
        });

        let swapchain_format = self
            .demo_builder
            .demo
            .surface
            .get_preferred_format(&self.demo_builder.demo.adapter)
            .unwrap();

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: None,
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &vert_shader,
                entry_point: "main",
                buffers: &[buffertypes::Vertex2D::desc()],
            },
            fragment: Some(wgpu::FragmentState {
                module: &frag_shader,
                entry_point: "main",
                targets: &[swapchain_format.into()],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleStrip,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                ..Default::default()
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
        });

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(&[
                buffertypes::Vertex2D::new(-1f32, -1f32),
                buffertypes::Vertex2D::new(1f32, -1f32),
                buffertypes::Vertex2D::new(-1f32, 1f32),
                buffertypes::Vertex2D::new(1f32, 1f32),
            ]),
            usage: wgpu::BufferUsages::VERTEX,
        });

        Scene {
            vertex_buffer,
            uniform_bind_group,
            uniform_buffer,
            render_pipeline,
            uniforms: self.uniforms,
        }
    }
}
