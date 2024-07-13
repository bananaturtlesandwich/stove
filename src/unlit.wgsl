#import bevy_pbr::forward_io::VertexOutput

@group(2) @binding(0) var texture: texture_2d<f32>;
@group(2) @binding(1) var tsampler: sampler;

@fragment
fn fragment(
    mesh: VertexOutput,
) -> @location(0) vec4<f32> {
    return textureSample(texture, tsampler, mesh.uv);
}