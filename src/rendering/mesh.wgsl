@group(0)
@binding(0)
var<uniform> vp: mat4x4f;

@vertex
fn vert(
    @location(0) pos: vec3f,
    @location(1) instx: vec4f,
    @location(2) insty: vec4f,
    @location(3) instz: vec4f,
    @location(4) instw: vec4f,
) -> @builtin(position) vec4f {
    let inst = mat4x4(instx, insty, instz, instw);
    return vp * inst * vec4(pos, 1.0);
}

@fragment
fn solid() -> @location(0) vec4f {
    return vec4(0.2, 0.5, 1.0, 1.0);
}

@fragment
fn wire() -> @location(0) vec4f {
    return vec4(0.0, 0.0, 0.0, 1.0);
}