use miniquad::*;

pub struct Sprite {
    block: Pipeline,
    bindings: Bindings,
}

impl Sprite {
    pub fn new(ctx: &mut Context) -> Self {
        let shader = Shader::new(
            ctx,
            include_str!("sprite.vert"),
            include_str!("sprite.frag"),
            ShaderMeta {
                uniforms: UniformBlockLayout {
                    uniforms: vec![
                        UniformDesc::new("model", UniformType::Mat4),
                        UniformDesc::new("view", UniformType::Mat4),
                        UniformDesc::new("top_left", UniformType::Float2),
                    ],
                },
                images: vec!["tex".to_string()],
            },
        )
        .unwrap();
        Self {
            block: Pipeline::with_params(
                ctx,
                &[BufferLayout::default()],
                &[],
                shader,
                PipelineParams {
                    depth_test: Comparison::LessOrEqual,
                    depth_write: true,
                    ..Default::default()
                },
            ),
            bindings: Bindings {
                vertex_buffers: vec![],
                index_buffer: Buffer::immutable(ctx, BufferType::IndexBuffer, &[0, 1, 2, 3, 4, 5]),
                images: vec![Texture::new(
                    ctx,
                    TextureAccess::Static,
                    Some(include_bytes!("../../assets/icons.rgba")),
                    TextureParams {
                        width: 256,
                        height: 256,
                        ..Default::default()
                    },
                )],
            },
        }
    }
    pub fn draw(&self, ctx: &mut Context, model: glam::Mat4, view: glam::Mat4, uv: glam::Vec2) {
        ctx.apply_pipeline(&self.block);
        ctx.apply_bindings(&self.bindings);
        ctx.apply_uniforms(&Uniforms { model, view, uv });
        ctx.draw(0, 6, 1);
    }
}

#[repr(C)]
struct Uniforms {
    model: glam::Mat4,
    view: glam::Mat4,
    uv: glam::Vec2,
}
