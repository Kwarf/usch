use std::borrow::Cow;

use encase::UniformBuffer;
use wgpu::{Queue, RenderPass};

use crate::{
    pipeline::{self, Pipeline},
    Shader, Time,
};

pub struct UniformsContext {
    pub time: Time,
}

pub struct Scene<'a> {
    pub duration: Time,
    pub content: Content<'a>,
    pub uniforms: Box<dyn Fn(&UniformsContext, &mut UniformBuffer<Vec<u8>>)>,
}

impl<'a> Scene<'a> {
    pub(crate) fn compile(
        self,
        device: &wgpu::Device,
        format: wgpu::TextureFormat,
    ) -> CompiledScene<'a> {
        match &self.content {
            Content::SingleFragmentShader(shader) => {
                let fragment_shader = wgpu::ShaderSource::Wgsl(Cow::Owned(shader.into_wgsl()));
                CompiledScene {
                    pipeline: pipeline::build_fragment(
                        device,
                        format,
                        fragment_shader,
                        &self.uniforms(&UniformsContext {
                            time: Time::from_frame(0),
                        }),
                    ),
                    scene: self,
                }
            }
        }
    }

    fn uniforms(&self, ctx: &UniformsContext) -> Vec<u8> {
        let mut buffer = UniformBuffer::new(Vec::new());
        (self.uniforms)(ctx, &mut buffer);
        buffer.into_inner()
    }
}

pub enum Content<'a> {
    SingleFragmentShader(Shader<'a>),
}

pub(crate) struct CompiledScene<'a> {
    pub(crate) pipeline: Pipeline,
    pub(crate) scene: Scene<'a>,
}

impl<'a> CompiledScene<'a> {
    pub(crate) fn render(
        &'a self,
        ctx: &UniformsContext,
        queue: &mut Queue,
        pass: &mut RenderPass<'a>,
    ) {
        queue.write_buffer(&self.pipeline.uniform_buffer, 0, &self.scene.uniforms(ctx));

        pass.set_pipeline(&self.pipeline.render_pipeline);
        pass.set_vertex_buffer(0, self.pipeline.vertex_buffer.slice(..));
        pass.set_bind_group(0, &self.pipeline.uniform_bind_group, &[]);
        pass.draw(0..4, 0..1);
    }
}
