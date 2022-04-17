use lazy_static::lazy_static;
use std::time::Instant;

use usch::DemoBuilder;

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct Uniforms {
    resolution: [f32; 3],
    time: f32,
}

lazy_static! {
    static ref CLOCK: Instant = Instant::now();
}

fn main() {
    DemoBuilder::new((1920, 1080), "01 Raymarch")
        .scene(|builder| {
            builder
                .with_uniforms(&|| {
                    bytemuck::bytes_of(&Uniforms {
                        resolution: [1920f32, 1080f32, 0f32],
                        time: CLOCK.elapsed().as_secs_f32(),
                    })
                    .to_vec()
                })
                .set_fragment_source(include_str!("shader.frag"))
                .build()
        })
        .build()
        .run();
}
