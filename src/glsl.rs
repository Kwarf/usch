use std::{path::{PathBuf, Path}, fs};

use lazy_static::lazy_static;
use shaderc::{CompileOptions, ResolvedInclude};

lazy_static! {
    #[rustfmt::skip]
    static ref VERT_PASSTHROUGH: Vec<u32> = compile_vertex(r"#version 420
layout(location = 0) in vec2 in_position;
void main()
{
    gl_Position = vec4(in_position.xy, 0.0, 1.0);
}");
}

pub fn vertex_passthrough() -> &'static [u32] {
    &VERT_PASSTHROUGH
}

pub fn compile_vertex(src: &str) -> Vec<u32> {
    compile(src, &None, shaderc::ShaderKind::Vertex)
}

pub fn compile_fragment(src: &str, includes: &Option<Vec<PathBuf>>) -> Vec<u32> {
    compile(src, includes, shaderc::ShaderKind::Fragment)
}

fn compile(src: &str, includes: &Option<Vec<PathBuf>>, shader_kind: shaderc::ShaderKind) -> Vec<u32> {
    let mut options = CompileOptions::new().unwrap();
    options.set_include_callback(|name, _, _, _| {
            let path = Path::new(name);
            match includes {
                Some(includes) => {
                    for include in includes {
                        let full_path = include.join(path);
                        if full_path.exists() {
                            return Ok(ResolvedInclude
                            {
                                resolved_name: full_path.to_str().unwrap().to_string(),
                                content: fs::read_to_string(full_path).unwrap(),
                            });
                        }
                    }
                    return Err(String::new());
                },
                None => Err(String::new()),
            }
        });

    shaderc::Compiler::new()
        .unwrap()
        .compile_into_spirv(src,
            shader_kind,
            "shader.glsl",
            "main",
            Some(&options)
        )
        .unwrap()
        .as_binary()
        .to_vec()
}
