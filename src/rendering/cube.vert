#version 330

in vec3 pos;
in mat4 inst_pos;

out vec3 tint;

uniform mat4 vp;
uniform ivec2 selected;

void main() {
    // if selected is some then x is 1
    tint = vec3(selected.x == 1 && selected.y == gl_InstanceID ? 1 : 0, 1, 0.5);
    gl_Position = vp * inst_pos * vec4(pos, 1.0);
}
