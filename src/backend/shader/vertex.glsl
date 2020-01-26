#version 330
in vec2 position;

out vec2 tex_coords;
out vec2 object_size;

uniform uvec2 target_dimensions;

uniform uvec2 object_texture_dimensions;
uniform uvec2 object_texture_offset;
uniform uvec2 object_dimensions;

uniform bool flip_vertically;
uniform bool flip_horizontally;

uniform ivec2 object_position;
uniform uvec2 object_scale;
uniform float depth;

void main() {
    vec2 tex_position = position;
    if (flip_vertically) {
        tex_position.y = 1 - tex_position.y;
    }

    if (flip_horizontally) {
        tex_position.x = 1 - tex_position.x;
    }

    tex_coords = vec2(
        (float(object_texture_offset.x) + float(object_dimensions.x) * tex_position.x)
            / float(object_texture_dimensions.x),
        (float(object_texture_offset.y) + float(object_dimensions.y) * tex_position.y)
            / float(object_texture_dimensions.y)
    );
    
    vec2 target_pos = vec2(
        (position.x * float(object_scale.x) * float(object_dimensions.x)
            + float(object_position.x)) / float(target_dimensions.x),
        (position.y * float(object_scale.y) * float(object_dimensions.y)
            + float(object_position.y)) / float(target_dimensions.y)
    );

    target_pos = target_pos * 2.0 - 1.0;

    gl_Position = vec4(target_pos, depth, 1.0);
}