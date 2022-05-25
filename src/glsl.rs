use std::{path::{PathBuf, Path}, fs};

use shaderc::{CompileOptions, ResolvedInclude};

pub fn compile_fragment(src: &str, includes: &Option<Vec<PathBuf>>) -> Result<Vec<u32>, shaderc::Error> {
    compile(src, includes, shaderc::ShaderKind::Fragment)
}

fn compile(src: &str, includes: &Option<Vec<PathBuf>>, shader_kind: shaderc::ShaderKind) -> Result<Vec<u32>, shaderc::Error> {
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

    Ok(shaderc::Compiler::new()
        .unwrap()
        .compile_into_spirv(src,
            shader_kind,
            "shader.glsl",
            "main",
            Some(&options)
        )?
        .as_binary()
        .to_vec())
}
