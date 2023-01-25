use miniquad::*;

pub struct Mesh {
    pub pipeline: Pipeline,
    pub bindings: Bindings,
}

impl Mesh {
    pub fn new(
        ctx: &mut Context,
        vertices: Vec<glam::Vec3>,
        colours: Vec<glam::Vec4>,
        indices: Vec<u32>,
    ) -> Self {
        let shader = Shader::new(
            ctx,
            include_str!("mesh.vert"),
            include_str!("mesh.frag"),
            ShaderMeta {
                uniforms: UniformBlockLayout {
                    uniforms: vec![UniformDesc::new("vp", UniformType::Mat4)],
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
                    BufferLayout::default(),
                    BufferLayout {
                        step_func: VertexStep::PerInstance,
                        ..Default::default()
                    },
                ],
                &[
                    VertexAttribute::with_buffer("pos", VertexFormat::Float3, 0),
                    VertexAttribute::with_buffer("colour", VertexFormat::Float3, 1),
                    VertexAttribute::with_buffer("inst_pos", VertexFormat::Mat4, 2),
                ],
                shader,
                PipelineParams {
                    depth_test: Comparison::LessOrEqual,
                    depth_write: true,
                    primitive_type: PrimitiveType::Triangles,
                    ..Default::default()
                },
            ),
            bindings: Bindings {
                vertex_buffers: vec![
                    Buffer::immutable(ctx, BufferType::VertexBuffer, &vertices),
                    Buffer::immutable(ctx, BufferType::VertexBuffer, &colours),
                    Buffer::stream(
                        ctx,
                        BufferType::VertexBuffer,
                        512 * 1024 * std::mem::size_of::<glam::Vec3>(),
                    ),
                ],
                index_buffer: Buffer::immutable(ctx, BufferType::IndexBuffer, &indices),
                images: vec![],
            },
        }
    }
}
