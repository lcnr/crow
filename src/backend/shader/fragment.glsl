#version 330

in vec2 tex_coords;
out vec4 color;

uniform mat4 color_modulation;
uniform sampler2D object;

void main() {
    color = color_modulation * texture(object, tex_coords);
    if (color.a == 0.0) {
        discard;
    }
}