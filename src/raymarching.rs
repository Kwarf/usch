use std::borrow::Cow;

use wgpu::util::DeviceExt;

use crate::{buffertypes, glsl};

pub struct Pipeline {
    pub vertex_buffer: wgpu::Buffer,
    pub uniform_bind_group: wgpu::BindGroup,
    pub uniform_buffer: wgpu::Buffer,
    pub render_pipeline: wgpu::RenderPipeline,
}

pub fn build_pipeline(
    device: &wgpu::Device,
    format: wgpu::TextureFormat,
    frag: wgpu::ShaderSource,
    uniforms: &[u8],
) -> Pipeline {
    let vert_shader = device.create_shader_module(&wgpu::ShaderModuleDescriptor {
        label: None,
        source: wgpu::ShaderSource::SpirV(Cow::Borrowed(glsl::vertex_passthrough())),
    });

    let frag_shader = device.create_shader_module(&wgpu::ShaderModuleDescriptor {
        label: None,
        source: frag,
    });

    let uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: None,
        contents: uniforms,
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

    Pipeline {
        vertex_buffer: device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(&[
                buffertypes::Vertex2D::new(-1f32, -1f32),
                buffertypes::Vertex2D::new(1f32, -1f32),
                buffertypes::Vertex2D::new(-1f32, 1f32),
                buffertypes::Vertex2D::new(1f32, 1f32),
            ]),
            usage: wgpu::BufferUsages::VERTEX,
        }),
        uniform_bind_group,
        uniform_buffer,
        render_pipeline: device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
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
                targets: &[format.into()],
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
        }),
    }
}
