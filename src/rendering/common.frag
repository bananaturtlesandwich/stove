#version 330

in vec3 tint;

void main() {
    gl_FragColor = vec4(tint, 1);
}