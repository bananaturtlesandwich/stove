#version 330

uniform vec3 tint;

void main() {
    gl_FragColor = vec4(tint, 1);
}