#version 330
in vec2 position;

out vec2 tex_coords;
out vec2 source_size;

uniform vec2 target_dimensions;
uniform vec2 source_texture_dimensions;
uniform vec2 source_position;

uniform uvec2 source_texture_offset;
uniform uvec2 source_dimensions;

uniform bool flip_vertically;
uniform bool flip_horizontally;

uniform uvec2 source_scale;
uniform float depth;

void main() {
    vec2 tex_position = position;
    if (flip_vertically) {
        tex_position.y = 1 - tex_position.y;
    }

    if (flip_horizontally) {
        tex_position.x = 1 - tex_position.x;
    }

    tex_coords = vec2(source_texture_offset + source_dimensions * tex_position) / source_texture_dimensions;
    
    vec2 target_pos = (position * vec2(source_scale * source_dimensions) + source_position) / target_dimensions;

    target_pos = target_pos * 2.0 - 1.0;

    gl_Position = vec4(target_pos, depth, 1.0);
}