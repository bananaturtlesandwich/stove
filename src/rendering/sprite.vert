#version 330
uniform vec2 top_left;
uniform mat4 model;
uniform mat4 view;
out vec2 uv;

const vec3 verts[] = vec3[](vec3(-0.5f, -0.5f, 0), vec3(0.5f, -0.5f, 0), vec3(0.5f, 0.5f, 0), vec3(-0.5f, -0.5f, 0), vec3(0.5f, 0.5f, 0), vec3(-0.5f, 0.5f, 0));

vec2 map(vec2 value, vec2 min1, vec2 max1, vec2 min2, vec2 max2) {
    return min2 + (value - min1) * (max2 - min2) / (max1 - min1);
}

void main() {
    vec3 pos = verts[gl_VertexID];
    uv = map(pos.xy, vec2(-0.5, 0.5), vec2(0.5, -0.5), top_left, top_left + vec2(0.25, 0.25));
    gl_Position = view * model * vec4(pos, 1);
}