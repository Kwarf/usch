use std::{borrow::Cow, time::Instant};

use futures::executor::block_on;
use winit::{
    dpi::PhysicalSize,
    event_loop::{ControlFlow, EventLoop},
    window::{Window, WindowBuilder},
};

pub use builders::DemoBuilder;
#[cfg(feature = "hot-reload")]
use source_watcher::SourceWatcher;

mod buffertypes;
mod builders;
mod glsl;
mod raymarching;
#[cfg(feature = "hot-reload")]
mod source_watcher;
pub mod time;
#[cfg(feature = "ui")]
pub mod ui;

pub struct Demo {
    event_loop: EventLoop<()>,
    window: Window,
    device: wgpu::Device,
    queue: wgpu::Queue,
    surface: wgpu::Surface,
    adapter: wgpu::Adapter,
    scenes: Vec<Scene>,
    #[cfg(feature = "ui")]
    ui: Option<ui::Ui>,
}

impl Demo {
    pub fn run(mut self) {
        let size = self.window.inner_size();
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: self.surface.get_preferred_format(&self.adapter).unwrap(),
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Mailbox,
        };
        self.surface.configure(&self.device, &config);

        let start_time = Instant::now();
        self.event_loop.run(move |event, _, control_flow| {
            *control_flow = ControlFlow::Poll;

            #[cfg(feature = "ui")]
            self.ui
                .as_mut()
                .map(|x| x.handle_event(&start_time.elapsed(), &event));

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
                    let active_scene = self.scenes.first_mut().unwrap();

                    #[cfg(feature = "hot-reload")]
                    active_scene.reload_shaders_if_requested(
                        &self.device,
                        self.surface.get_preferred_format(&self.adapter).unwrap(),
                    );

                    active_scene.update(&self.queue);

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

                        active_scene.draw(&mut rpass);
                    }

                    #[cfg(feature = "ui")]
                    self.ui.as_mut().map(|x| {
                        x.draw(&self.window, &self.device, &self.queue, &mut encoder, &view)
                    });

                    self.queue.submit(Some(encoder.finish()));
                    frame.present();
                }
                winit::event::Event::MainEventsCleared => {
                    self.window.request_redraw();
                }
                _ => (),
            }
        });
    }

    pub fn get_preferred_format(&self) -> wgpu::TextureFormat {
        self.surface.get_preferred_format(&self.adapter).unwrap()
    }
}

pub struct Scene {
    pipeline: raymarching::Pipeline,
    #[cfg(feature = "hot-reload")]
    fragment_source_watcher: Option<SourceWatcher>,
    uniforms: Box<dyn Fn() -> Vec<u8>>,
}

impl Scene {
    pub fn update(&self, queue: &wgpu::Queue) {
        queue.write_buffer(&self.pipeline.uniform_buffer, 0, &(self.uniforms)());
    }

    pub fn draw<'a>(&'a self, pass: &mut wgpu::RenderPass<'a>) {
        pass.set_pipeline(&self.pipeline.render_pipeline);
        pass.set_vertex_buffer(0, self.pipeline.vertex_buffer.slice(..));
        pass.set_bind_group(0, &self.pipeline.uniform_bind_group, &[]);
        pass.draw(0..4, 0..1);
    }

    #[cfg(feature = "hot-reload")]
    pub fn reload_shaders_if_requested(
        &mut self,
        device: &wgpu::Device,
        format: wgpu::TextureFormat,
    ) {
        match &self.fragment_source_watcher {
            Some(rx) => match rx.get_new_content() {
                Some(content) => {
                    self.pipeline = raymarching::build_pipeline(
                        device,
                        format,
                        wgpu::ShaderSource::SpirV(Cow::Owned(glsl::compile_fragment(&content))),
                        &(self.uniforms)(),
                    )
                }
                None => (),
            },
            None => (),
        }
    }
}
