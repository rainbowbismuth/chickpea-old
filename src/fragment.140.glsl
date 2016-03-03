#version 140

uniform sampler2D tex;
in vec2 v_tex_coords;
in vec3 v_color;
out vec4 f_color;

void main() {
    f_color = clamp(vec4(v_color, 1.0) * texture(tex, v_tex_coords) * 2, 0.0, 1.0);
}
