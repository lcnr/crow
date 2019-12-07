#version 140

in vec2 tex_coords;
out vec4 color;

uniform sampler2D object;

void main() {
    color = texture(object, tex_coords);
}