#[macro_use]
extern crate glium;
extern crate cgmath;
extern crate itertools;

mod defs;
mod mesh;
mod bufferset;

use std::io::prelude::*;
use std::fs::File;

use glium::glutin;
use glium::{DisplayBuild, Surface};

use mesh::*;
use bufferset::*;

fn main() {
  let window = glutin::WindowBuilder::new()
    .with_depth_buffer(24)
    .with_dimensions(800, 600)
    .with_title("Convex Hull".to_string())
    .build_glium().unwrap();

  let geom = get_cube();

  let buffers = BufferSet::from_mesh(& window, & geom);

  let mut vert_shader_file = File::open("src/shader/base.vs").unwrap();
  let mut vert_shader = String::new();
  vert_shader_file.read_to_string(&mut vert_shader).unwrap();

  let mut frag_shader_file = File::open("src/shader/base.fs").unwrap();
  let mut frag_shader = String::new();
  frag_shader_file.read_to_string(&mut frag_shader).unwrap();

  let basic_program = glium::Program::from_source(& window, & vert_shader, & frag_shader, None).unwrap();

  // let basic_uniforms: glium::

  let draw_params: glium::DrawParameters = Default::default();

  loop {
    let mut target = window.draw();

    target.clear_color_and_depth((0.0, 0.0, 0.0, 1.0), 1.0);

    // target.draw(& buffers.vertices, & buffers.indices, & basic_program, & basic_uniforms, & draw_params);

    target.finish().unwrap();

    for event in window.poll_events() {
      match event {
        glutin::Event::Closed => return,
        glutin::Event::KeyboardInput(glutin::ElementState::Pressed, _, Some(glutin::VirtualKeyCode::Escape)) => return,
        _ => (),
      }
    }
  }


}
