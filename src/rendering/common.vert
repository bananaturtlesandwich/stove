#version 330

in vec3 pos;
in vec3 colour;

out vec3 tint;

uniform mat4 transform;

void main() {
    tint = colour;
    gl_Position = transform * vec4(pos, 1.0);
}
