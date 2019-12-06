#version 140

in vec2 position;

out vec2 tex_coords;

uniform uvec2 target_size;
uniform uvec2 object_size;
uniform ivec2 object_position;
uniform uvec2 object_scale;

void main() {
    tex_coords = position;
    
    vec2 target_pos = vec2(
        (position.x * float(object_scale.x) / float(target_size.x) * float(object_size.x) + float(object_position.x) / float(target_size.x)) * 2.0 - 1.0,
        (position.y * float(object_scale.x) / float(target_size.y) * float(object_size.y) + float(object_position.y) / float(target_size.y)) * 2.0 - 1.0
    );


    gl_Position = vec4(target_pos, 0.0, 1.0);
}