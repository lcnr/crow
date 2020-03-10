#version 330

// select (x1, y1, x2, y2)
layout (location = 0) in vec4 position;

// (x1, y1, x2, y2)
uniform vec4 start_end;

void main() {
    gl_Position = vec4(
        start_end.x * position.x + start_end.z * position.z,
        start_end.y * position.y + start_end.w * position.w,
        0.0, 1.0
    );
}