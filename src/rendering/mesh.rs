use miniquad::*;

pub struct Mesh {
    pipeline: Pipeline,
    bindings: Vec<Bindings>,
}

impl Mesh {
    pub fn new(
        ctx: &mut Context,
        vertices: Vec<glam::Vec3>,
        indices: Vec<u32>,
        uvs: Vec<Vec<(f32, f32)>>,
        mats: Vec<(u32, u32, Vec<u8>)>,
        mat_data: Vec<(u32, u32)>,
    ) -> Self {
        let mut mat_indices = Vec::with_capacity(mat_data.capacity());
        let mut mat_data = mat_data.into_iter().peekable();
        while let Some((mat, first)) = mat_data.next() {
            mat_indices.push((
                mat as usize,
                first as usize,
                mat_data
                    .peek()
                    .map_or(indices.len(), |(_, first)| *first as usize),
            ))
        }
        let shader = Shader::new(
            ctx,
            include_str!("mesh.vert"),
            include_str!("mesh.frag"),
            ShaderMeta {
                uniforms: UniformBlockLayout {
                    uniforms: vec![UniformDesc::new("transform", UniformType::Mat4)],
                },
                images: vec!["tex".to_string()],
            },
        )
        .unwrap();
        Self {
            pipeline: Pipeline::with_params(
                ctx,
                &[BufferLayout::default(), BufferLayout::default()],
                &[
                    VertexAttribute::with_buffer("pos", VertexFormat::Float3, 0),
                    VertexAttribute::with_buffer("texcoord", VertexFormat::Float2, 1),
                ],
                shader,
                PipelineParams {
                    depth_test: Comparison::LessOrEqual,
                    depth_write: true,
                    primitive_type: PrimitiveType::Triangles,
                    ..Default::default()
                },
            ),
            bindings: mat_indices
                .into_iter()
                .map(|(i, start, end)| Bindings {
                    vertex_buffers: vec![
                        Buffer::immutable(ctx, BufferType::VertexBuffer, &vertices),
                        Buffer::immutable(ctx, BufferType::VertexBuffer, &uvs[i]),
                    ],
                    index_buffer: Buffer::immutable(
                        ctx,
                        BufferType::IndexBuffer,
                        &indices[start..end],
                    ),
                    images: vec![Texture::new(
                        ctx,
                        TextureAccess::Static,
                        Some(mats[i].2.as_slice()),
                        TextureParams {
                            format: TextureFormat::RGBA8,
                            wrap: TextureWrap::Repeat,
                            filter: FilterMode::Linear,
                            width: mats[i].0,
                            height: mats[i].1,
                        },
                    )],
                })
                .collect(),
        }
    }

    pub fn draw(&self, ctx: &mut Context, vp: glam::Mat4) {
        for binding in self.bindings.iter() {
            ctx.apply_pipeline(&self.pipeline);
            ctx.apply_bindings(binding);
            ctx.apply_uniforms(&vp);
            ctx.draw(
                0,
                (binding.index_buffer.size() / std::mem::size_of::<usize>()) as i32,
                1,
            )
        }
    }
}
