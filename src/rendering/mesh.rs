use miniquad::*;

pub struct Mesh {
    pub pipeline: Pipeline,
    pub bindings: Bindings,
    len: i32,
}

impl Mesh {
    pub fn new(ctx: &mut Context, vertices: Vec<glam::Vec3>, indices: Vec<u32>) -> Self {
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
        let len = vertices.len() as i32;
        Self {
            pipeline: Pipeline::with_params(
                ctx,
                &[BufferLayout::default()],
                &[VertexAttribute::with_buffer("pos", VertexFormat::Float3, 0)],
                shader,
                PipelineParams {
                    depth_test: Comparison::LessOrEqual,
                    depth_write: true,
                    primitive_type: PrimitiveType::Triangles,
                    ..Default::default()
                },
            ),
            bindings: Bindings {
                vertex_buffers: vec![Buffer::immutable(ctx, BufferType::VertexBuffer, &vertices)],
                index_buffer: Buffer::immutable(ctx, BufferType::IndexBuffer, &indices),
                images: vec![],
            },
            len,
        }
    }

    pub fn draw(&self, ctx: &mut Context, uniform: &glam::Mat4) {
        unsafe {
            gl::glEnable(gl::GL_PROGRAM_POINT_SIZE);
        }
        ctx.apply_pipeline(&self.pipeline);
        ctx.apply_bindings(&self.bindings);
        ctx.apply_uniforms(uniform);
        ctx.draw(0, self.len, 1);
    }
}
