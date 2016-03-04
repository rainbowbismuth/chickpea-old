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

#[macro_use]
extern crate glium;
extern crate image;
extern crate time;

use std::io::Cursor;

use glium::glutin;
use glium::Surface;
use glium::draw_parameters::DrawParameters;
use glium::index::PrimitiveType;
use std::time::Duration;
use std::thread;

#[derive(Copy, Clone)]
struct Vertex {
    position: [f32; 2],
    color: [f32; 3],
    tex_coords: [f32; 2],
}

implement_vertex!(Vertex, position, color, tex_coords);

#[derive(Copy, Clone)]
struct Attr {
    world_pos: [f32; 2],
}

implement_vertex!(Attr, world_pos);

fn main() {
    use glium::DisplayBuild;

    // building the display, ie. the main object
    let display = glutin::WindowBuilder::new()
        .build_glium()
        .unwrap();

    let image = image::load(Cursor::new(&include_bytes!("river-stone.jpg")[..]),
                            image::JPEG)
                    .expect("JPEG loading failed")
                    .to_rgba();

    let image_dimensions = image.dimensions();
    let image = glium::texture::RawImage2d::from_raw_rgba_reversed(image.into_raw(),
                                                                   image_dimensions);
    let texture = glium::texture::CompressedSrgbTexture2d::new(&display, image)
        .expect("texture creation failed");

    // building the vertex buffer, which contains all the vertices that we will draw
    let vertex_buffer = {
        glium::VertexBuffer::new(&display,
                                 &[Vertex {
                                       position: [-0.5, -0.5],
                                       color: [1.0, 0.0, 0.0],
                                       tex_coords: [0.0, 0.0],
                                   },
                                   Vertex {
                                       position: [0.0, 0.5],
                                       color: [0.0, 1.0, 0.0],
                                       tex_coords: [0.0, 1.0],
                                   },
                                   Vertex {
                                       position: [0.5, -0.5],
                                       color: [0.0, 0.0, 1.0],
                                       tex_coords: [1.0, 0.0],
                                   }])
            .expect("vertex_buffer creation failed")
    };

    // building the index buffer
    let index_buffer = glium::IndexBuffer::new(&display,
                                               PrimitiveType::TrianglesList,
                                               &[0u16, 1, 2])
                           .expect("index_buffer creation failed");

    let triangles = [Attr { world_pos: [0.0, 0.5] },
                     Attr { world_pos: [-0.5, -0.5] },
                     Attr { world_pos: [0.5, -0.5] }];

    let attr_buffer = glium::VertexBuffer::new(&display, &triangles)
        .expect("attr_buffer creation failed");

    // compiling shaders and linking them together
    let program = program!(&display,
        140 => {
            vertex: include_str!("vertex.140.glsl"),
            fragment: include_str!("fragment.140.glsl")
        }
    )
                      .expect("shader program creation failed");

    let draw_params: DrawParameters = Default::default();

    let mut angle = 0.0f32;

    const FIXED_TIME_STAMP: u64 = 16_666_667;
    let mut previous_clock = time::precise_time_ns();

    // the main loop
    'mainloop: loop {

        // building the uniforms
        let uniforms = uniform! {
            matrix: [
                [ angle.cos(), 0.0, angle.sin(), 0.0],
                [         0.0, 1.0,         0.0, 0.0],
                [-angle.sin(), 0.0, angle.cos(), 0.0],
                [         0.0, 0.0,         0.0, 1.0f32]
            ],
            tex: &texture,
        };

        // drawing a frame
        let mut target = display.draw();
        target.clear_color(0.0, 0.0, 0.0, 0.0);
        let per_instance = attr_buffer.per_instance().expect("per_instance() failed");
        target.draw((&vertex_buffer, per_instance),
                    &index_buffer,
                    &program,
                    &uniforms,
                    &draw_params)
              .expect("draw call error");
        target.finish().expect("frame end error");

        // polling and handling the events received by the window
        for event in display.poll_events() {
            match event {
                glutin::Event::Closed => break 'mainloop,
                _ => (),
            }
        }

        let now = time::precise_time_ns();
        let diff = now - previous_clock;
        previous_clock = now;
        if diff < FIXED_TIME_STAMP {
            let ms = ((FIXED_TIME_STAMP - diff) / 1_000_000) as u64;
            thread::sleep(Duration::from_millis(ms));
        }
        angle += 1.0 * (diff as f32 / 1_000_000_000.0);
    }
}
