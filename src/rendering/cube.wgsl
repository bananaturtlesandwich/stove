@group(0)
@binding(0)
var<uniform> vp: mat4x4<f32>;

struct Output {
    @location(0) selected: f32,
    @builtin(position) position: vec4<f32>,
};

@vertex
fn vert(
    @location(0) pos: vec3<f32>,
    @location(1) instx: vec4<f32>,
    @location(2) insty: vec4<f32>,
    @location(3) instz: vec4<f32>,
    @location(4) instw: vec4<f32>,
    @location(5) selected: f32
) -> Output {
    var out: Output;
    let inst = mat4x4(instx, insty, instz, instw);
    out.selected = selected;
    out.position = vp * inst * vec4(pos.x, pos.y, pos.z, 1.0);
    return out;
}

@fragment
fn frag(vert: Output) -> @location(0) vec4<f32> {
    return vec4<f32>(vert.selected, 1.0, 0.5, 1.0);
}