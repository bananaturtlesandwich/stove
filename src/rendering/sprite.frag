#version 330

uniform sampler2D tex;
uniform vec4 tint;
in vec2 uv;
out vec4 colour;

void main() {
    colour = tint * texture(tex, uv);
}