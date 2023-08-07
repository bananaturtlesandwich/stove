#version 330

in vec3 pos;
in mat4 inst_pos;
in float selected;

out vec3 tint;

uniform mat4 vp;

void main() {
    tint = vec3(selected, 1, 0.5);
    gl_Position = vp * inst_pos * vec4(pos, 1.0);
}
