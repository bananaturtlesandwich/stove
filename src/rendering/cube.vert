#version 330

in vec3 pos;
in mat4 inst_pos;

flat out ivec2 selected;
flat out int id;

uniform mat4 vp;
uniform ivec2 uselected;

void main() {
    id = gl_InstanceID;
    selected = uselected;
    gl_Position = vp * inst_pos * vec4(pos, 1.0);
}
