#version 100

precision lowp float;

uniform vec3 tint;

void main() {
    gl_FragColor = vec4(tint, 1);
}