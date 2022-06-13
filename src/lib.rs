pub use demo::Demo;
pub use engine::run;
pub use music::Music;

pub use scene::Scene;
pub use time::Time;

pub mod demo;
mod engine;
pub mod music;
mod pipeline;
pub mod scene;
pub mod time;

#[derive(PartialEq)]
pub enum Fullscreen {
    No,
    Borderless,
    Exclusive,
}

pub enum Shader<'a> {
    Wgsl(&'a str),
    #[cfg(feature = "glsl")]
    Glsl(&'a str),
}

impl<'a> Shader<'a> {
    pub(crate) fn into_wgsl(&self) -> String {
        match self {
            Shader::Wgsl(source) => source.to_string(),
            #[cfg(feature = "glsl")]
            Shader::Glsl(source) => {
                use naga::back::wgsl::WriterFlags;
                use naga::front::glsl::{Options, Parser};
                use naga::valid::{Capabilities, ValidationFlags, Validator};
                use naga::ShaderStage;

                let mut parser = Parser::default();
                let options = Options::from(ShaderStage::Fragment);
                let module = parser.parse(&options, source).unwrap();
                let mut validator = Validator::new(ValidationFlags::all(), Capabilities::empty());
                let module_info = validator.validate(&module).unwrap();

                naga::back::wgsl::write_string(&module, &module_info, WriterFlags::empty()).unwrap()
            }
        }
    }
}
