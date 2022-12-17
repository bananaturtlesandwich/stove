#version 100

attribute vec3 pos;

uniform mat4 model;
uniform mat4 view;

// this isn't going to change so might as well just make it a constant
const mat4 PROJECTION = mat4(
    1, 0, 0, 0,
    0, 1.8, 0, 0,
    0, 0, 1, 1,
    0, 0, -1, 0
);

void main() {
    gl_Position = PROJECTION * view * model * vec4(pos, 1.0);
}
