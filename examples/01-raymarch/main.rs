use usch::{
    DemoBuilder,
};

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct Uniforms {
    resolution: [f32; 3],
    time: f32,
}

fn main() {
    let mut tracker = usch::sync::Tracker::new(140
        , Some(std::path::Path::new("examples/01-raymarch/sync.json"))
        , &["foo"]
    );

    tracker.set_value("foo", 4, 4.5);

    DemoBuilder::new((1920, 1080), false, "01 Raymarch")
        .with_tracker(tracker)
        .with_ogg_music(include_bytes!("music.ogg"), Some(743006))
        .scene(|builder| {
            builder
                .with_uniforms(|time| {
                    bytemuck::bytes_of(&Uniforms {
                        resolution: [1920f32, 1080f32, 0f32],
                        time: time.elapsed().as_secs_f32(),
                    })
                    .to_vec()
                })
                .set_fragment_source(include_str!("shader.frag"))
                .watch_fragment_source(std::path::Path::new("examples/01-raymarch/shader.frag"))
                .build()
        })
        .build()
        .run();
}
