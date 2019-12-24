#version 140

in vec2 position;

out vec2 tex_coords;

uniform uvec2 target_dimensions;
uniform uvec2 object_dimensions;
uniform ivec2 object_position;
uniform uvec2 object_scale;
uniform float depth;

void main() {
    tex_coords = position;
    
    vec2 target_pos = vec2(
        (position.x * float(object_scale.x) / float(target_dimensions.x) * float(object_dimensions.x) + float(object_position.x) / float(target_dimensions.x)) * 2.0 - 1.0,
        (position.y * float(object_scale.y) / float(target_dimensions.y) * float(object_dimensions.y) + float(object_position.y) / float(target_dimensions.y)) * 2.0 - 1.0
    );


    gl_Position = vec4(target_pos, depth, 1.0);
}