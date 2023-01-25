#version 330

in vec3 pos;
in vec4 colour;
in mat4 inst_pos;

out vec4 ocolour;

uniform mat4 vp;

void main() {
    ocolour = colour;
    gl_Position = vp * inst_pos * vec4(pos, 1.0);
}
