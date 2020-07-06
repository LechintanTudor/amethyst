#version 450

const vec2[4] POSITIONS = vec2[](
    vec2(0.5, -0.5),  // Right bottom
    vec2(-0.5, -0.5), // Left bottom
    vec2(0.5, 0.5),   // Right top
    vec2(-0.5, 0.5)   // Left top
);

layout(std140, set = 0, binding = 0) uniform UiViewArgs {
	vec2 inverse_window_size;
};

layout(location = 0) in vec2 position;
layout(location = 1) in vec2 dimensions;
layout(location = 2) in vec4 color;

layout(location = 0) out vec4 frag_color;

void main() {
	vec2 center = vec2(position.x * inverse_window_size.x, -position.y * inverse_window_size.y);
	vec2 final_position = center + dimensions * POSITIONS[gl_VertexIndex] * inverse_window_size;

	gl_Position = vec4(final_position, 0.0, 1.0);
    frag_color = color;
}