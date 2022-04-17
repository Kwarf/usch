use std::time::Duration;

use egui::FontDefinitions;
use egui_wgpu_backend::{RenderPass, ScreenDescriptor};
use egui_winit_platform::{Platform, PlatformDescriptor};
use winit::window::Window;

use crate::DemoBuilder;

impl DemoBuilder {
    pub fn use_debug_ui(mut self, init: impl Fn(&mut egui::Ui) + 'static) -> DemoBuilder {
        self.demo.ui = Some(Ui::new(&self, init));
        self
    }
}

pub struct Ui {
    init: Box<dyn Fn(&mut egui::Ui)>,
    platform: egui_winit_platform::Platform,
    pass: egui_wgpu_backend::RenderPass,
}

impl Ui {
    pub fn new(demo_builder: &DemoBuilder, init: impl Fn(&mut egui::Ui) + 'static) -> Ui {
        let window = &demo_builder.demo.window;
        let size = window.inner_size();

        let platform = Platform::new(PlatformDescriptor {
            physical_width: size.width as u32,
            physical_height: size.height as u32,
            scale_factor: window.scale_factor(),
            font_definitions: FontDefinitions::default(),
            style: Default::default(),
        });

        let pass = RenderPass::new(
            &demo_builder.demo.device,
            demo_builder.demo.get_preferred_format(),
            1,
        );

        Ui {
            init: Box::new(init),
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
            egui::Window::new("usch toolkit").show(ctx, |ui| {
                self.init.as_mut()(ui);
            });
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

    use egui::{Slider, Ui};

    use crate::time::{SeekableTimeSource, TimeSource};

    pub fn time_seeker(ui: &mut Ui, time_source: &mut SeekableTimeSource) {
        let mut time = time_source.elapsed().as_secs_f32();
        ui.add(Slider::new(&mut time, 0.0..=100.0).text("Time"));
        if time != time_source.elapsed().as_secs_f32() {
            time_source.seek(Duration::from_secs_f32(time));
        }
    }
}
