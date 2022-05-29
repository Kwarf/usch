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

pub struct Demo<'a> {
    pub name: &'static str,
    pub resolution: (u32, u32),
    pub fullscreen: Fullscreen,
    pub music: Option<Music>,
    pub scenes: Vec<Scene<'a>>,
}

pub struct Scene<'a> {
    pub duration: Time,
    pub content: SceneContent<'a>,
}

pub enum SceneContent<'a> {
    SingleFragmentShader(Shader<'a>),
}

pub enum Shader<'a> {
    Wgsl(&'a str),
    #[cfg(feature = "glsl")]
    Glsl(&'a str),
}
