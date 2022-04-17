use std::{cell::RefCell, rc::Rc};

use usch::{
    time::{SeekableTimeSource, TimeSource},
    DemoBuilder,
};

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct Uniforms {
    resolution: [f32; 3],
    time: f32,
}

fn main() {
    let clock = Rc::new(RefCell::new(SeekableTimeSource::now()));

    DemoBuilder::new((1920, 1080), "01 Raymarch")
        .use_debug_ui({
            let clock = clock.clone();
            move |ui| {
                usch::ui::widgets::time_seeker(ui, &mut clock.borrow_mut());
            }
        })
        .scene(|builder| {
            builder
                .with_uniforms({
                    let clock = clock.clone();
                    move || {
                        bytemuck::bytes_of(&Uniforms {
                            resolution: [1920f32, 1080f32, 0f32],
                            time: clock.borrow().elapsed().as_secs_f32(),
                        })
                        .to_vec()
                    }
                })
                .set_fragment_source(include_str!("shader.frag"))
                .watch_fragment_source(std::path::Path::new("examples/01-raymarch/shader.frag"))
                .build()
        })
        .build()
        .run();
}
