use pollster::FutureExt;
use wgpu::{
    Backends, Color, CommandEncoderDescriptor, DeviceDescriptor, Features, Instance, Limits,
    LoadOp, Operations, PowerPreference, PresentMode, RenderPassColorAttachment,
    RenderPassDescriptor, RequestAdapterOptions, SurfaceConfiguration, TextureUsages,
    TextureViewDescriptor,
};
use winit::{
    dpi::PhysicalSize,
    event::{Event, KeyboardInput, VirtualKeyCode, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::{Window, WindowBuilder},
};

use crate::{Demo, Fullscreen, Time};

pub fn run(demo: Demo) {
    let duration: Time = demo
        .scenes
        .iter()
        .fold(Time::default(), |acc, x| acc + x.duration);

    let music = demo.music.as_ref().map(|x| x.decode());

    let (event_loop, window) = create_window(&demo);
    let instance = Instance::new(Backends::PRIMARY);
    let surface = unsafe { instance.create_surface(&window) };

    let adapter = instance
        .request_adapter(&RequestAdapterOptions {
            power_preference: PowerPreference::HighPerformance,
            force_fallback_adapter: false,
            compatible_surface: Some(&surface),
        })
        .block_on()
        .unwrap();

    let (device, queue) = adapter
        .request_device(
            &DeviceDescriptor {
                label: None,
                features: Features::empty(),
                limits: Limits::default(),
            },
            None,
        )
        .block_on()
        .unwrap();

    let size = window.inner_size();
    let config = SurfaceConfiguration {
        usage: TextureUsages::RENDER_ATTACHMENT,
        format: surface.get_preferred_format(&adapter).unwrap(),
        width: size.width,
        height: size.height,
        present_mode: PresentMode::Fifo,
    };
    surface.configure(&device, &config);

    let mut t = std::time::Instant::now();
    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Poll;

        match event {
            Event::WindowEvent {
                event:
                    WindowEvent::CloseRequested
                    | WindowEvent::KeyboardInput {
                        input:
                            KeyboardInput {
                                virtual_keycode: Some(VirtualKeyCode::Escape),
                                ..
                            },
                        ..
                    },
                ..
            } => *control_flow = ControlFlow::Exit,
            Event::MainEventsCleared => window.request_redraw(),
            Event::RedrawRequested(_) => {
                let frame = surface.get_current_texture().unwrap();
                let view = frame.texture.create_view(&TextureViewDescriptor::default());
                let mut encoder =
                    device.create_command_encoder(&CommandEncoderDescriptor::default());
                {
                    let mut rpass = encoder.begin_render_pass(&RenderPassDescriptor {
                        label: None,
                        color_attachments: &[RenderPassColorAttachment {
                            view: &view,
                            resolve_target: None,
                            ops: Operations {
                                load: LoadOp::Clear(Color::BLACK),
                                store: true,
                            },
                        }],
                        depth_stencil_attachment: None,
                    });
                }

                queue.submit(Some(encoder.finish()));
                frame.present();

                println!("{:?}", t.elapsed());
                t = std::time::Instant::now();
            }
            _ => (),
        }
    });
}

fn create_window(demo: &Demo) -> (EventLoop<()>, Window) {
    let event_loop = EventLoop::new();
    let resolution = PhysicalSize::new(demo.resolution.0, demo.resolution.1);
    let window = WindowBuilder::new()
        .with_title(demo.name)
        .with_inner_size(resolution)
        .with_fullscreen(fullscreen_mode(&event_loop, demo))
        .build(&event_loop)
        .unwrap();

    match demo.fullscreen {
        Fullscreen::Borderless | Fullscreen::Exclusive => window.set_cursor_visible(false),
        _ => (),
    };

    (event_loop, window)
}

fn fullscreen_mode(event_loop: &EventLoop<()>, demo: &Demo) -> Option<winit::window::Fullscreen> {
    match demo.fullscreen {
        Fullscreen::No => None,
        Fullscreen::Borderless => Some(winit::window::Fullscreen::Borderless(None)),
        Fullscreen::Exclusive => {
            let resolution = PhysicalSize::new(demo.resolution.0, demo.resolution.1);
            let video_mode = event_loop
                .primary_monitor()
                .unwrap()
                .video_modes()
                .find(|x| x.refresh_rate() == 60 && x.size() == resolution)
                .expect(&format!(
                    "Could not find a {}x{} @ 60Hz fullscreen video mode",
                    resolution.width, resolution.height
                ));
            Some(winit::window::Fullscreen::Exclusive(video_mode))
        }
    }
}
