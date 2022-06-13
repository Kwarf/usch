use crate::{Fullscreen, Music, Scene};

pub struct Demo<'a> {
    pub name: &'static str,
    pub resolution: (u32, u32),
    pub fullscreen: Fullscreen,
    pub music: Option<Music>,
    pub scenes: Vec<Scene<'a>>,
}
