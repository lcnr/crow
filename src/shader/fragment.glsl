#version 140

in vec2 tex_coords;
out vec4 color;

uniform sampler2D tex;

void main() {
    color = texture(tex, tex_coords);
}