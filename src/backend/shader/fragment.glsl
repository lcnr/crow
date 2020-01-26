#version 330

in vec2 tex_coords;
out vec4 color;

uniform mat4 color_modulation;
uniform bool invert_color;

uniform sampler2D object;

void main() {
    color = color_modulation * texture(object, tex_coords);
    if (invert_color) {
        color.rgb = 1 - color.rgb;
    }

    if (color.a == 0.0) {
        discard;
    }
}