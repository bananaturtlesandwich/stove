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
                        UniformDesc::new("tint", UniformType::Float3),
                    ],
                },
                images: vec![],
            },
        )
        .unwrap();
        Self {
            block: Pipeline::with_params(
                ctx,
                &[BufferLayout::default()],
                &[VertexAttribute::new("pos", VertexFormat::Float3)],
                shader,
                PipelineParams {
                    depth_test: Comparison::LessOrEqual,
                    depth_write: true,
                    primitive_type: PrimitiveType::Lines,
                    ..Default::default()
                },
            ),
            bindings: Bindings {
                vertex_buffers: vec![Buffer::immutable(
                    ctx,
                    BufferType::VertexBuffer,
                    &[
                        // front verts
                        glam::vec3(-0.5, -0.5, -0.5),
                        glam::vec3(-0.5, 0.5, -0.5),
                        glam::vec3(0.5, -0.5, -0.5),
                        glam::vec3(0.5, 0.5, -0.5),
                        // back verts
                        glam::vec3(-0.5, -0.5, 0.5),
                        glam::vec3(-0.5, 0.5, 0.5),
                        glam::vec3(0.5, -0.5, 0.5),
                        glam::vec3(0.5, 0.5, 0.5),
                    ],
                )],
                index_buffer: Buffer::immutable(
                    ctx,
                    BufferType::IndexBuffer,
                    &[
                        0, 1, 0, 2, 1, 3, 2, 3, 4, 5, 4, 6, 5, 7, 6, 7, 4, 0, 5, 1, 6, 2, 7, 3,
                    ],
                ),
                images: vec![],
            },
        }
    }
    pub fn apply(&self, ctx: &mut Context) {
        ctx.apply_pipeline(&self.block);
        ctx.apply_bindings(&self.bindings);
    }
    pub fn draw(&self, ctx: &mut Context, model: glam::Mat4, view: glam::Mat4, tint: glam::Vec3) {
        ctx.apply_uniforms(&Uniforms { model, view, tint });
        ctx.draw(0, 24, 1);
    }
}

#[repr(C)]
struct Uniforms {
    model: glam::Mat4,
    view: glam::Mat4,
    tint: glam::Vec3,
}
