@group(0)
@binding(0)
var<uniform> mvp: mat4x4f;

@vertex
fn vert(
    @location(0) pos: vec3f,
) -> @builtin(position) vec4f {
    return mvp * vec4(pos, 1.0);
}

@fragment
fn frag() -> @location(0) vec4f {
    return vec4(0.2, 0.5, 1.0, 1.0);
}