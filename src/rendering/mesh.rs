use miniquad::*;

pub struct Mesh {
    len: i32,
    solid_pipeline: Pipeline,
    solid_bindings: Bindings,
    wire_pipeline: Pipeline,
    wire_bindings: Bindings,
}

impl Mesh {
    pub fn new(ctx: &mut Context, vertices: Vec<glam::Vec3>, indices: Vec<u32>) -> Self {
        let solid = Shader::new(
            ctx,
            include_str!("mesh.vert"),
            include_str!("solid.frag"),
            ShaderMeta {
                uniforms: UniformBlockLayout {
                    uniforms: vec![
                        UniformDesc::new("vp", UniformType::Mat4),
                        UniformDesc::new("tint", UniformType::Float3),
                    ],
                },
                images: vec![],
            },
        )
        .unwrap();
        let wire = Shader::new(
            ctx,
            include_str!("mesh.vert"),
            include_str!("wire.frag"),
            ShaderMeta {
                uniforms: UniformBlockLayout {
                    uniforms: vec![UniformDesc::new("vp", UniformType::Mat4)],
                },
                images: vec![],
            },
        )
        .unwrap();
        let len = vertices.len() as i32;
        Self {
            len,
            solid_pipeline: Pipeline::with_params(
                ctx,
                &[BufferLayout::default()],
                &[VertexAttribute::with_buffer("pos", VertexFormat::Float3, 0)],
                solid,
                PipelineParams {
                    depth_test: Comparison::LessOrEqual,
                    depth_write: true,
                    primitive_type: PrimitiveType::Triangles,
                    ..Default::default()
                },
            ),
            solid_bindings: Bindings {
                vertex_buffers: vec![Buffer::immutable(ctx, BufferType::VertexBuffer, &vertices)],
                index_buffer: Buffer::immutable(ctx, BufferType::IndexBuffer, &indices),
                images: vec![],
            },
            wire_pipeline: Pipeline::with_params(
                ctx,
                &[BufferLayout::default()],
                &[VertexAttribute::with_buffer("pos", VertexFormat::Float3, 0)],
                wire,
                PipelineParams {
                    depth_test: Comparison::LessOrEqual,
                    depth_write: true,
                    primitive_type: PrimitiveType::Lines,
                    ..Default::default()
                },
            ),
            wire_bindings: Bindings {
                vertex_buffers: vec![Buffer::immutable(ctx, BufferType::VertexBuffer, &vertices)],
                index_buffer: Buffer::immutable(
                    ctx,
                    BufferType::IndexBuffer,
                    &indices
                        .chunks_exact(3)
                        .flat_map(|i| [i[0], i[1], i[0], i[2], i[1], i[2]])
                        .collect::<Vec<_>>(),
                ),
                images: vec![],
            },
        }
    }

    pub fn draw(&self, ctx: &mut Context, uniform: glam::Mat4, colour: [f32; 3]) {
        ctx.apply_pipeline(&self.solid_pipeline);
        ctx.apply_bindings(&self.solid_bindings);
        ctx.apply_uniforms(&(uniform, colour));
        ctx.draw(0, self.len, 1);
        ctx.apply_pipeline(&self.wire_pipeline);
        ctx.apply_bindings(&self.wire_bindings);
        ctx.apply_uniforms(&uniform);
        ctx.draw(0, self.len, 1);
    }
}
