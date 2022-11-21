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

const int INDICES[] = int[](
    // front face
    1, 2, 0,
    1, 2, 3,
    // back face
    5, 6, 4,
    5, 6, 7,
    // top face
    3, 5, 1,
    3, 5, 7,
    // bottom face
    2, 4, 0,
    2, 4, 6,
    // left face
    0, 5, 4,
    0, 5, 1,
    // right face
    2, 7, 6,
    2, 7, 3
);

void main() {
    gl_Position = PROJECTION * view * model * vec4(VERTICES[INDICES[gl_VertexID]], 1.0);
}
