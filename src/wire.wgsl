#import bevy_pbr::forward_io::VertexOutput

@fragment
fn fragment(
    mesh: VertexOutput,
) -> @location(0) vec4<f32> {
#ifdef SELECTED
    return vec4(1.0, 1.0, 0.0, 1.0);
#else
    return vec4(0.0, 1.0, 0.0, 1.0);
#endif
}