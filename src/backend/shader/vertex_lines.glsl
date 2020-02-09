#version 330

in vec2 position;

uniform vec4 start_end;

void main() {
    gl_Position = vec4(start_end.xy * position.x + start_end.zw * position.y, 0.0, 1.0);
}