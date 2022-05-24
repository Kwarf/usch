use std::{time::Duration, sync::{Arc, Mutex}};

use egui::FontDefinitions;
use egui_wgpu_backend::{RenderPass, ScreenDescriptor};
use egui_winit_platform::{Platform, PlatformDescriptor};
use winit::window::Window;

use crate::{DemoBuilder, sync, music::Music};

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
        music: &mut Option<Arc<Mutex<Music>>>,
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
                            widgets::tracker_view(tracker, music, ui);
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
    use std::{sync::{Arc, Mutex}};

    use egui::{Ui, Grid, Key, Event, Color32, RichText};

    use crate::{sync, music::Music};

    pub fn tracker_view(tracker: &mut sync::Tracker,
        music: &mut Option<Arc<Mutex<Music>>>,
        ui: &mut Ui
    ) {
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
                        tracker.time.set_paused(!tracker.time.is_paused());

                        match music {
                            Some(music) => {
                                let mut music = music.as_ref().lock().unwrap();
                                music.paused = tracker.time.is_paused();
                                if !music.paused {
                                    music.seek(&tracker.get_time_from_row(row as u32));
                                }
                            },
                            None => (),
                        };
                    }
                    Event::Key {
                        key: Key::ArrowUp,
                        pressed: true,
                        modifiers,
                    } => {
                        row = std::cmp::max(0, row - if modifiers.shift { 4 } else { 1 });
                        tracker.time.seek(tracker.get_time_from_row(row as u32));
                        tracker.time.set_paused(true);
                    }
                    Event::Key {
                        key: Key::ArrowDown,
                        pressed: true,
                        modifiers,
                    } => {
                        row += if modifiers.shift { 4 } else { 1 };
                        tracker.time.seek(tracker.get_time_from_row(row as u32));
                        tracker.time.set_paused(true);
                    }
                    _ => ()
                }
            }
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
                    if n < 0 {
                        ui.end_row();
                        continue;
                    }

                    let label = if n == row { format!("{:04} >", n) } else { format!("{:04}", n) };
                    if n % 4 == 0 {
                        ui.colored_label(Color32::RED, label);
                    } else {
                        ui.label(label);
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
