use miniquad::*;
pub struct Cube {
    pub pipeline: Pipeline,
    pub bindings: Bindings,
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
                        UniformDesc::new("vp", UniformType::Mat4),
                        UniformDesc::new("uselected", UniformType::Int2),
                    ],
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
                    VertexAttribute::with_buffer("inst_pos", VertexFormat::Mat4, 1),
                ],
                shader,
                PipelineParams {
                    depth_test: Comparison::LessOrEqual,
                    depth_write: true,
                    primitive_type: PrimitiveType::Lines,
                    ..Default::default()
                },
            ),
            bindings: Bindings {
                vertex_buffers: vec![
                    Buffer::immutable(
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
                    ),
                    Buffer::stream(
                        ctx,
                        BufferType::VertexBuffer,
                        512 * 1024 * std::mem::size_of::<glam::Vec3>(),
                    ),
                ],
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
}
