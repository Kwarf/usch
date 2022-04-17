use std::borrow::Cow;

use futures::executor::block_on;
use wgpu::util::DeviceExt;
use winit::{
    dpi::PhysicalSize,
    event_loop::{ControlFlow, EventLoop},
    window::{Window, WindowBuilder},
};

pub use builders::DemoBuilder;

mod buffertypes;
mod builders;
mod glsl;

pub struct Demo {
    event_loop: EventLoop<()>,
    window: Window,
    device: wgpu::Device,
    queue: wgpu::Queue,
    surface: wgpu::Surface,
    adapter: wgpu::Adapter,
    scenes: Vec<Scene>,
}

impl Demo {
    pub fn run(self) {
        let size = self.window.inner_size();
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: self.surface.get_preferred_format(&self.adapter).unwrap(),
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Mailbox,
        };
        self.surface.configure(&self.device, &config);

        self.event_loop.run(move |event, _, control_flow| {
            *control_flow = ControlFlow::Wait;

            match event {
                winit::event::Event::WindowEvent {
                    event:
                        winit::event::WindowEvent::CloseRequested
                        | winit::event::WindowEvent::KeyboardInput {
                            input:
                                winit::event::KeyboardInput {
                                    virtual_keycode: Some(winit::event::VirtualKeyCode::Escape),
                                    ..
                                },
                            ..
                        },
                    window_id,
                } if window_id == self.window.id() => *control_flow = ControlFlow::Exit,
                winit::event::Event::RedrawRequested(_) => {
                    let frame = self.surface.get_current_texture().unwrap();
                    let view = frame
                        .texture
                        .create_view(&wgpu::TextureViewDescriptor::default());
                    let mut encoder = self
                        .device
                        .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
                    {
                        let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                            label: None,
                            color_attachments: &[wgpu::RenderPassColorAttachment {
                                view: &view,
                                resolve_target: None,
                                ops: wgpu::Operations {
                                    load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                                    store: true,
                                },
                            }],
                            depth_stencil_attachment: None,
                        });
                        self.scenes.first().unwrap().draw(&self.queue, &mut rpass);
                    }

                    self.queue.submit(Some(encoder.finish()));
                    frame.present();
                    self.window.request_redraw();
                }
                _ => (),
            }
        });
    }
}

pub struct Scene {
    vertex_buffer: wgpu::Buffer,
    uniform_bind_group: wgpu::BindGroup,
    uniform_buffer: wgpu::Buffer,
    render_pipeline: wgpu::RenderPipeline,
    uniforms: &'static dyn Fn() -> Vec<u8>,
}

impl Scene {
    pub fn draw<'a>(&'a self, queue: &wgpu::Queue, pass: &mut wgpu::RenderPass<'a>) {
        queue.write_buffer(&self.uniform_buffer, 0, &(self.uniforms)());

        pass.set_pipeline(&self.render_pipeline);
        pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        pass.set_bind_group(0, &self.uniform_bind_group, &[]);
        pass.draw(0..4, 0..1);
    }
}
