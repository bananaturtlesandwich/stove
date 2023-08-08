#version 330

in vec3 pos;
in vec2 texcoord;

out vec2 uv;

uniform mat4 transform;

void main() {
    uv = texcoord;
    gl_Position = transform * vec4(pos, 1.0);
}
