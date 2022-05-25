use std::{borrow::Cow, time::Instant, sync::{Arc, Mutex}, path::PathBuf};

use cpal::{traits::{DeviceTrait, HostTrait, StreamTrait}, SampleFormat, Stream, SupportedBufferSize, BufferSize};
use futures::executor::block_on;
use time::{SeekableTimeSource, TimeSource};
use winit::{
    dpi::PhysicalSize,
    event_loop::{ControlFlow, EventLoop},
    window::{Window, WindowBuilder},
};

pub use builders::DemoBuilder;
#[cfg(feature = "editor")]
use source_watcher::SourceWatcher;

mod buffertypes;
mod builders;
mod glsl;
pub mod music;
mod raymarching;
#[cfg(feature = "editor")]
mod source_watcher;
pub mod sync;
mod time;
#[cfg(feature = "editor")]
pub mod ui;

pub struct Demo {
    event_loop: EventLoop<()>,
    window: Window,
    device: wgpu::Device,
    queue: wgpu::Queue,
    surface: wgpu::Surface,
    adapter: wgpu::Adapter,
    music: Option<Arc<Mutex<music::Music>>>,
    scenes: Vec<Scene>,
    time: SeekableTimeSource,
    #[cfg(feature = "editor")]
    tracker: Option<sync::Tracker>,
    #[cfg(feature = "editor")]
    ui: ui::Ui,
}

impl Demo {
    pub fn run(mut self) {
        let _stream = match self.music {
            Some(_) => {
                let stream = self.init_music().unwrap();
                stream.play().unwrap();
                Some(stream)
            }
            None => None,
        };

        let size = self.window.inner_size();
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: self.surface.get_preferred_format(&self.adapter).unwrap(),
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Mailbox,
        };
        self.surface.configure(&self.device, &config);

        #[cfg(feature=  "editor")]
        let start_time = Instant::now();
        self.time = SeekableTimeSource::now();

        self.event_loop.run(move |event, _, control_flow| {
            *control_flow = ControlFlow::Poll;

            #[cfg(feature = "editor")]
            self.ui.handle_event(&start_time.elapsed(), &event);

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

                    #[cfg(feature = "editor")]
                    active_scene.reload_shaders_if_requested(
                        &self.device,
                        &self.time,
                        self.surface.get_preferred_format(&self.adapter).unwrap(),
                    );

                    active_scene.update(&self.queue, &self.time);

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

                    #[cfg(feature = "editor")]
                    {
                        self.tracker.as_mut().unwrap().time = self.time.clone();
                        self.ui.draw(&self.window
                            , &self.device
                            , &self.queue
                            , &mut encoder
                            , &view
                            , &mut self.tracker
                            , &mut self.music
                        );
                        self.time = self.tracker.as_ref().unwrap().time.clone();
                    }

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

    fn init_music(&self) -> Option<Stream> {
        match &self.music {
            None => None,
            Some(music) => {
                let music = music.as_ref().lock().unwrap();
                let host = cpal::default_host();
                let device = host.default_output_device().unwrap();
                let supported_config = device
                    .supported_output_configs()
                    .unwrap()
                    .find(|x| x.channels() == 2
                        && x.min_sample_rate().0 <= music.sample_rate
                        && x.max_sample_rate().0 >= music.sample_rate
                        && x.sample_format() == SampleFormat::F32
                    )
                    .expect(&format!("No audio output device supporting {} sample rate found", music.sample_rate))
                    .with_sample_rate(cpal::SampleRate(music.sample_rate));
                let mut config = supported_config.config();

                // Use the smallest supported buffer size during editing for consistent scrubbing
                #[cfg(feature = "editor")]
                match supported_config.buffer_size() {
                    SupportedBufferSize::Range
                    {
                        min,
                        max: _
                    } => config.buffer_size = BufferSize::Fixed(*min),
                    SupportedBufferSize::Unknown => (),
                }

                device.build_output_stream(&config,
                    {
                        let music = self.music.as_ref().unwrap().clone();
                        move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
                            match music.as_ref().lock() {
                                Ok(mut music) => {
                                    let samples = music.read(data.len());
                                    for i in data.iter_mut().enumerate() {
                                        *i.1 = cpal::Sample::from(&samples[i.0]);
                                    }
                                },
                                Err(_) => panic!(),
                            }
                        }
                    },
                    move |_err| {
                        panic!()
                    },
                ).ok()
            }
        }
    }
}

pub struct Scene {
    pipeline: raymarching::Pipeline,
    #[cfg(feature = "editor")]
    fragment_source_watcher: Option<SourceWatcher>,
    #[cfg(feature = "editor")]
    glsl_include_paths: Option<Vec<PathBuf>>,
    uniforms: Box<dyn Fn(&dyn TimeSource) -> Vec<u8>>,
}

impl Scene {
    pub fn update(&self, queue: &wgpu::Queue, time: &dyn TimeSource) {
        queue.write_buffer(&self.pipeline.uniform_buffer, 0, &(self.uniforms)(time));
    }

    pub fn draw<'a>(&'a self, pass: &mut wgpu::RenderPass<'a>) {
        pass.set_pipeline(&self.pipeline.render_pipeline);
        pass.set_vertex_buffer(0, self.pipeline.vertex_buffer.slice(..));
        pass.set_bind_group(0, &self.pipeline.uniform_bind_group, &[]);
        pass.draw(0..4, 0..1);
    }

    #[cfg(feature = "editor")]
    pub fn reload_shaders_if_requested(
        &mut self,
        device: &wgpu::Device,
        time: &dyn TimeSource,
        format: wgpu::TextureFormat,
    ) {
        match &self.fragment_source_watcher {
            Some(rx) => match rx.get_new_content() {
                Some(content) => {
                    let shader = glsl::compile_fragment(&content, &self.glsl_include_paths);
                    if shader.is_err() {
                        println!("Failed to editor shader:\n{}", shader.err().unwrap());
                        return;
                    }

                    self.pipeline = raymarching::build_pipeline(
                        device,
                        format,
                        wgpu::ShaderSource::SpirV(Cow::Owned(shader.unwrap())),
                        &(self.uniforms)(time),
                    )
                }
                None => (),
            },
            None => (),
        }
    }
}

mod binary {
    use std::{io::{Read, Write}, mem::size_of};

    use bytemuck::{from_bytes, bytes_of};

    pub fn read<T: bytemuck::Pod>(mut reader: impl Read) -> T {
        let mut buf: Vec<u8> = Vec::with_capacity(size_of::<T>());
        reader.read_exact(&mut buf).unwrap();
        *from_bytes::<T>(&buf)
    }
    
    pub fn write<T: bytemuck::Pod>(mut writer: impl Write, value: &T) {
        write_bytes(writer, bytes_of(value));
    }

    pub fn write_bytes(mut writer: impl Write, bytes: &[u8]) {
        writer.write_all(bytes).unwrap();
    }
}
