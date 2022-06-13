use std::time::Duration;

use usch::{scene::Content, Demo, Fullscreen, Music, Scene, Shader};

#[derive(encase::ShaderType)]
struct Uniforms {
    resolution: mint::Vector3<f32>,
    time: f32,
}

const RESOLUTION: (u32, u32) = (1920, 1080);

fn main() {
    let demo = Demo {
        name: "01 Raymarch",
        resolution: RESOLUTION,
        fullscreen: Fullscreen::No,
        music: Some(Music::Ogg(include_bytes!("music.ogg"))),
        scenes: vec![Scene {
            duration: Duration::from_secs(8).into(),
            content: Content::SingleFragmentShader(Shader::Glsl(include_str!("shader.frag"))),
            uniforms: Box::new(|ctx, buffer| {
                buffer
                    .write(&Uniforms {
                        resolution: mint::Vector3 {
                            x: RESOLUTION.0 as f32,
                            y: RESOLUTION.1 as f32,
                            z: 0f32,
                        },
                        time: Duration::from(ctx.time).as_secs_f32(),
                    })
                    .unwrap();
            }),
        }],
    };

    usch::run(demo);
}
