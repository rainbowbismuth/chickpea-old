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

uniform sampler2D tex;
in vec2 v_tex_coords;
in vec3 v_color;
out vec4 f_color;

void main() {
    f_color = clamp(vec4(v_color, 1.0) * texture(tex, v_tex_coords) * 2, 0.0, 1.0);
}
