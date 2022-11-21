#version 330

uniform mat4 model;
uniform mat4 view;

// this isn't going to change so might as well just make it a constant
const mat4 PROJECTION = mat4(
    1, 0, 0, 0,
    0, 1.8, 0, 0,
    0, 0, 1, 1,
    0, 0, -1, 0
);

const vec3 VERTICES[] = vec3[](
    // front verts
    vec3(-0.5, -0.5, -0.5),
    vec3(-0.5,  0.5, -0.5),
    vec3( 0.5, -0.5, -0.5),
    vec3( 0.5,  0.5, -0.5),
    // back verts
    vec3(-0.5, -0.5,  0.5),
    vec3(-0.5,  0.5,  0.5),
    vec3( 0.5, -0.5,  0.5),
    vec3( 0.5,  0.5,  0.5)
);

void main() {
    gl_Position = PROJECTION * view * model * vec4(VERTICES[gl_VertexID], 1.0);
}
