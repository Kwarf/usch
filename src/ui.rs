use std::time::Duration;

use egui::FontDefinitions;
use egui_wgpu_backend::{RenderPass, ScreenDescriptor};
use egui_winit_platform::{Platform, PlatformDescriptor};
use winit::window::Window;

use crate::{DemoBuilder, sync};

impl DemoBuilder {
    pub fn with_tracker(mut self, tracker: sync::Tracker) -> DemoBuilder {
        self.demo.tracker = Some(tracker);
        self
    }
}

pub struct Ui {
    platform: egui_winit_platform::Platform,
    pass: egui_wgpu_backend::RenderPass,
}

impl Ui {
    pub fn new(window: &Window
        , device: &wgpu::Device
        , format: wgpu::TextureFormat
    ) -> Ui {
        let size = window.inner_size();

        let platform = Platform::new(PlatformDescriptor {
            physical_width: size.width as u32,
            physical_height: size.height as u32,
            scale_factor: window.scale_factor(),
            font_definitions: FontDefinitions::default(),
            style: Default::default(),
        });

        let pass = RenderPass::new(
            device,
            format,
            1,
        );

        Ui {
            platform,
            pass,
        }
    }

    pub fn draw(
        &mut self,
        window: &Window,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        encoder: &mut wgpu::CommandEncoder,
        view: &wgpu::TextureView,
        tracker: &mut Option<sync::Tracker>,
    ) {
        let size = window.inner_size();
        let screen_descriptor = ScreenDescriptor {
            physical_width: size.width,
            physical_height: size.height,
            scale_factor: window.scale_factor() as f32,
        };

        self.platform.begin_frame();
        {
            let ctx = &self.platform.context();

            match tracker {
                Some(ref mut tracker) => {
                    egui::Window::new("Tracker")
                        .show(ctx, |ui| {
                            widgets::tracker_view(tracker, ui);
                        });
                },
                None => (),
            }
        }
        let output = self.platform.end_frame(None);

        let paint_jobs = self.platform.context().tessellate(output.shapes);

        self.pass
            .add_textures(device, queue, &output.textures_delta)
            .unwrap();

        self.pass
            .update_buffers(device, queue, &paint_jobs, &screen_descriptor);

        self.pass
            .execute(encoder, view, &paint_jobs, &screen_descriptor, None)
            .unwrap();
    }

    pub fn handle_event(&mut self, elapsed: &Duration, event: &winit::event::Event<()>) {
        self.platform.update_time(elapsed.as_secs_f64());
        self.platform.handle_event(event);
    }
}

pub mod widgets {
    use std::time::Duration;

    use egui::{Slider, Ui, Grid, Key, Event, Color32, RichText, Label};

    use crate::{time::{SeekableTimeSource, TimeSource}, sync};

    pub fn time_seeker(ui: &mut Ui, time_source: &mut SeekableTimeSource) {
        let mut time = time_source.elapsed().as_secs_f32();
        ui.add(Slider::new(&mut time, 0.0..=100.0).text("Time"));
        if time != time_source.elapsed().as_secs_f32() {
            time_source.seek(Duration::from_secs_f32(time));
        }
    }

    pub fn tracker_view(tracker: &mut sync::Tracker, ui: &mut Ui) {
        let mut row = tracker.current_row() as i32;
        {
            let events = &ui.input().events;
            for event in events {
                match event {
                    Event::Key {
                        key: Key::Space,
                        pressed: true,
                        modifiers: _,
                    } => {
                        tracker.playing = !tracker.playing;
                    }
                    Event::Key {
                        key: Key::ArrowUp,
                        pressed: true,
                        modifiers,
                    } => {
                        row = std::cmp::max(1, row - if modifiers.shift { 4 } else { 1 });
                    }
                    Event::Key {
                        key: Key::ArrowDown,
                        pressed: true,
                        modifiers,
                    } => {
                        row += if modifiers.shift { 4 } else { 1 };
                    }
                    _ => ()
                }
            }
        }

        if !tracker.playing {
            tracker.time.seek(tracker.get_time_from_row(row as u32));
        }
        let tracks = tracker.tracks();

        Grid::new("tracker_view")
            .num_columns(tracks.len() + 1)
            .striped(true)
            .show(ui, |ui| {
                // Headings
                ui.label(RichText::new("Beat").strong());
                for track in tracks {
                    ui.label(RichText::new(track.name()).strong());
                }
                ui.end_row();

                // Values
                for n in (row - 20)..(row + 20) {
                    if n <= 0 {
                        ui.end_row();
                        continue;
                    }

                    if (n - 1) % 4 == 0 {
                        ui.colored_label(Color32::RED, format!("{:04}", n));
                    } else {
                        ui.label(format!("{:04}", n));
                    }

                    for track in tracks {
                        if n == row {
                            match track.get_value(n as u32) {
                                Some(value) => ui.label(RichText::new(format!("{}", value)).background_color(Color32::GRAY)),
                                None => ui.label(RichText::new("...").background_color(Color32::LIGHT_GRAY)),
                            };
                        } else {
                            match track.get_value(n as u32) {
                                Some(value) => ui.label(format!("{}", value)),
                                None => ui.label("..."),
                            };
                        }
                    }

                    ui.end_row();
                }
            });
    }
}
