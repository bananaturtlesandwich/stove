#version 100

precision lowp float;

uniform bool selected;

void main() {
    gl_FragColor = vec4(selected ? 1 : 0, 1, 0.5, 1);
}