#version 330
layout (location = 0) in vec2 position;

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
uniform mat2 source_rotation;
uniform float depth;

void main() {
    // Texture coordinates
    vec2 tex_position = position;
    if (flip_vertically) {
        tex_position.y = 1 - tex_position.y;
    }

    if (flip_horizontally) {
        tex_position.x = 1 - tex_position.x;
    }

    tex_coords = vec2(source_texture_offset + source_dimensions * tex_position) / source_texture_dimensions;

    // Position
    // Everything starts with a 1x1 square
    // All sprites don't have such dimension
    vec2 scale = vec2(source_scale * source_dimensions);
    vec2 pos = position * scale;
    // [0.5, 0.5] is currently the center of the square
    // It has to be [0.0, 0.0] for a correct rotation
    // Relatively to current dimension, "re-center" the square
    vec2 trick = vec2(pos.x - scale.x / 2.0, pos.y - scale.y / 2.0);
    // Then rotate it
    trick *= source_rotation;
    // Then "re-re-center" it
    trick = vec2(trick.x + scale.x / 2.0, trick.y + scale.y / 2.0);
    
    vec2 target_pos = (trick + source_position) / target_dimensions;

    target_pos = target_pos * 2.0 - 1.0;

    gl_Position = vec4(target_pos, depth, 1.0);
}