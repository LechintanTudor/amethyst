#version 450

layout(set = 1, binding = 0) uniform sampler2D ui_texture;

layout(location = 0) in vec2 tex_coords;
layout(location = 1) in vec4 color;
layout(location = 2) in vec4 color_bias;

layout(location = 0) out vec4 out_color;

void main() {
	out_color = (texture(ui_texture, tex_coords) + color_bias) * color;
}