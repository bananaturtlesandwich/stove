use miniquad::*;

pub struct Axes {
    pipeline: Pipeline,
    x: Bindings,
    y: Bindings,
    z: Bindings,
}

fn axis(ctx: &mut Context, axis: glam::Vec3) -> Bindings {
    Bindings {
        vertex_buffers: vec![
            Buffer::immutable(ctx, BufferType::VertexBuffer, &[-axis, axis]),
            Buffer::immutable(ctx, BufferType::VertexBuffer, &[axis]),
        ],
        index_buffer: Buffer::immutable(ctx, BufferType::IndexBuffer, &[0, 1]),
        images: vec![],
    }
}

impl Axes {
    pub fn new(ctx: &mut Context) -> Self {
        let shader = Shader::new(
            ctx,
            include_str!("common.vert"),
            include_str!("common.frag"),
            ShaderMeta {
                uniforms: UniformBlockLayout {
                    uniforms: vec![UniformDesc::new("transform", UniformType::Mat4)],
                },
                images: vec![],
            },
        )
        .unwrap();
        Self {
            pipeline: Pipeline::with_params(
                ctx,
                &[
                    BufferLayout::default(),
                    BufferLayout {
                        step_func: VertexStep::PerInstance,
                        ..Default::default()
                    },
                ],
                &[
                    VertexAttribute::with_buffer("pos", VertexFormat::Float3, 0),
                    VertexAttribute::with_buffer("colour", VertexFormat::Float3, 1),
                ],
                shader,
                PipelineParams {
                    depth_test: Comparison::LessOrEqual,
                    depth_write: true,
                    primitive_type: PrimitiveType::Lines,
                    ..Default::default()
                },
            ),
            x: axis(ctx, glam::Vec3::X),
            y: axis(ctx, glam::Vec3::Y),
            z: axis(ctx, glam::Vec3::Z),
        }
    }

    pub fn draw(&self, ctx: &mut Context, filter: &glam::Vec3, mvp: glam::Mat4) {
        macro_rules! draw {
            ($axis: ident) => {
                if filter.$axis == 1.0 {
                    ctx.apply_pipeline(&self.pipeline);
                    ctx.apply_bindings(&self.$axis);
                    ctx.apply_uniforms(&mvp);
                    ctx.draw(0, 2, 1);
                }
            };
        }
        draw!(x);
        draw!(y);
        draw!(z);
    }
}
