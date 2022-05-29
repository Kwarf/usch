pub use engine::run;
pub use music::Music;
pub use time::Time;

mod engine;
pub mod music;
pub mod time;

#[derive(PartialEq)]
pub enum Fullscreen {
    No,
    Borderless,
    Exclusive,
}

pub struct Demo {
    pub name: &'static str,
    pub resolution: (u32, u32),
    pub fullscreen: Fullscreen,
    pub music: Option<Music>,
    pub scenes: Vec<Scene>,
}

pub struct Scene {
    duration: Time,
}
