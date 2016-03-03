#version 140

uniform mat4 matrix;

in vec2 world_pos;
in vec2 position;
in vec3 color;
in vec2 tex_coords;

out vec2 v_tex_coords;
out vec3 v_color;

void main() {
    gl_Position = vec4(position + world_pos, 0.0, 1.0) * matrix;
    v_tex_coords = tex_coords;
    v_color = color;
}
