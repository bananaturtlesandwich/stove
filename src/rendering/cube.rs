use miniquad::*;
pub struct Cube {
    block: Pipeline,
    bindings: Bindings,
}

impl Cube {
    pub fn new(ctx: &mut Context) -> Self {
        let shader = Shader::new(
            ctx,
            include_str!("cube.vert"),
            include_str!("cube.frag"),
            ShaderMeta {
                uniforms: UniformBlockLayout {
                    uniforms: vec![
                        UniformDesc::new("model", UniformType::Mat4),
                        UniformDesc::new("view", UniformType::Mat4),
                    ],
                },
                images: vec![],
            },
        )
        .unwrap();
        Self {
            block: Pipeline::new(ctx, &[BufferLayout::default()], &[], shader),
            bindings: Bindings {
                vertex_buffers: vec![],
                index_buffer: Buffer::immutable(
                    ctx,
                    BufferType::IndexBuffer,
                    &(0..36).collect::<Vec<_>>(),
                ),
                images: vec![],
            },
        }
    }
    pub fn draw(&self, ctx: &mut Context, model: glam::Mat4, view: glam::Mat4) {
        ctx.apply_pipeline(&self.block);
        ctx.apply_bindings(&self.bindings);
        ctx.apply_uniforms(&(Uniforms { model, view }));
        ctx.draw(0, 36, 1);
    }
}

#[repr(C)]
struct Uniforms {
    model: glam::Mat4,
    view: glam::Mat4,
}
