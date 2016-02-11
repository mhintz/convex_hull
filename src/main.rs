#[macro_use]
extern crate glium;
extern crate cgmath;
extern crate itertools;

mod defs;
mod mesh;
mod bufferset;

use glium::glutin;
use glium::{DisplayBuild, Surface};
use glium::vertex;
use glium::index;

use defs::*;
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

  let draw_params: glium::DrawParameters = Default::default();

  loop {
    let mut target = window.draw();

    target.clear_color_and_depth((0.0, 0.0, 0.0, 1.0), 1.0);

    // target.draw(& attribute_buf, & index_buf, & program, & uniforms, & draw_params);

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
