#version 330

in vec2 tex_coords;
out vec4 color;

uniform mat4 color_modulation;
uniform bool invert_color;
uniform bool flip_vertically;
uniform bool flip_horizontally;

uniform sampler2D object;

void main() {
    vec2 coords = tex_coords;
    if (flip_vertically) {
        coords.y = 1 - coords.y;
    }

    if (flip_horizontally) {
        coords.x = 1 - coords.x;
    }

    color = color_modulation * texture(object, coords);

    if (invert_color) {
        color.xzy = 1 - color.xyz;
    }

    if (color.a == 0.0) {
        discard;
    }
}