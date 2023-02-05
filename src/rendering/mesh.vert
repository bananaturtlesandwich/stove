#version 330

in vec3 pos;

uniform mat4 vp;

void main() {
    gl_Position = vp * vec4(pos, 1.0);
}
