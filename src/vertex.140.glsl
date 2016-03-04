// chickpea, A small tile-based game project
// Copyright (C) 2016 Emily A. Bellows
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <http://www.gnu.org/licenses/>.

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
