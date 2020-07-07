#version 450

layout(set = 1, binding = 0) uniform sampler2D ui_texture;

layout(location = 0) in vec4 frag_color;
layout(location = 1) in vec2 frag_tex_coords;

layout(location = 0) out vec4 out_color;

void main() {
	out_color = texture(ui_texture, frag_tex_coords) * frag_color;
}