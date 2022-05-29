use usch::{Demo, Fullscreen, Music};

fn main() {
    let demo = Demo {
        name: "01 Raymarch",
        resolution: (1920, 1080),
        fullscreen: Fullscreen::No,
        music: Some(Music::Ogg(include_bytes!("music.ogg"))),
        scenes: vec![],
    };

    usch::run(demo);
}
