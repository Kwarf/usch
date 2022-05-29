use std::time::Duration;

use usch::{Demo, Fullscreen, Music, Scene, SceneContent, Shader};

fn main() {
    let demo = Demo {
        name: "01 Raymarch",
        resolution: (1920, 1080),
        fullscreen: Fullscreen::No,
        music: Some(Music::Ogg(include_bytes!("music.ogg"))),
        scenes: vec![Scene {
            duration: Duration::from_secs(8).into(),
            content: SceneContent::SingleFragmentShader(Shader::Glsl(include_str!("shader.frag"))),
        }],
    };

    usch::run(demo);
}
