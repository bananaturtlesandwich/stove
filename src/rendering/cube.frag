#version 330

uniform vec3 tint;
out vec4 colour;

void main() {
    colour = vec4(tint, 1);
}