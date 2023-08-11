@group(0)
@binding(0)
var<uniform> vp: mat4x4f;

struct Output {
    @location(0) orig: vec3f,
    @builtin(position) position: vec4f
}

@vertex
fn vert(
    @location(0) pos: vec3f,
) -> Output {
    return Output(pos, vp * vec4(pos, 1.0));
}

@fragment
fn frag(in: Output) -> @location(0) vec4f {
    return vec4(abs(in.orig), 1.0);
}