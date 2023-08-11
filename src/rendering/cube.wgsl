@group(0)
@binding(0)
var<uniform> vp: mat4x4f;

struct Output {
    @location(0) selected: f32,
    @builtin(position) position: vec4f,
};

@vertex
fn vert(
    @location(0) pos: vec3f,
    @location(1) instx: vec4f,
    @location(2) insty: vec4f,
    @location(3) instz: vec4f,
    @location(4) instw: vec4f,
    @location(5) selected: f32
) -> Output {
    let inst = mat4x4(instx, insty, instz, instw);
    return Output(selected, vp * inst * vec4(pos, 1.0));
}

@fragment
fn frag(vert: Output) -> @location(0) vec4f {
    return vec4(vert.selected, 1.0, 0.5, 1.0);
}