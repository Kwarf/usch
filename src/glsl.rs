use lazy_static::lazy_static;

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
    compile(src, shaderc::ShaderKind::Vertex)
}

pub fn compile_fragment(src: &str) -> Vec<u32> {
    compile(src, shaderc::ShaderKind::Fragment)
}

fn compile(src: &str, shader_kind: shaderc::ShaderKind) -> Vec<u32> {
    shaderc::Compiler::new()
        .unwrap()
        .compile_into_spirv(src, shader_kind, "shader.glsl", "main", None)
        .unwrap()
        .as_binary()
        .to_vec()
}
