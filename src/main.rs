#[macro_use]
extern crate glium;
extern crate cgmath;
extern crate itertools;

mod defs;
mod mesh;
mod bufferset;
mod half_edge_mesh;

use std::io::prelude::*;
use std::fs::File;

use glium::glutin;
use glium::{DisplayBuild, Surface};

use cgmath::{Vector3, EuclideanVector, Rotation3, Rad, Angle, Matrix, SquareMatrix};

use defs::*;
use mesh::*;
use bufferset::*;

const WINDOW_WIDTH: u32 = 800;
const WINDOW_HEIGHT: u32 = 800;
const ASPECT_RATIO: f32 = (WINDOW_WIDTH as f32) / (WINDOW_HEIGHT as f32);
const AR_SCALE: f32 = 1.3;
const NEAR_PLANE_Z: f32 = 0.5;
const FAR_PLANE_Z: f32 = 1000.0;

fn mat4_uniform(mat: & Mat4) -> [[f32; 4]; 4] {
  return mat.clone().into();
}

fn main() {
  let window = glutin::WindowBuilder::new()
    .with_depth_buffer(24)
    .with_dimensions(WINDOW_WIDTH, WINDOW_HEIGHT)
    .with_title("Convex Hull".to_string())
    .build_glium().unwrap();

  // Geometry Data
  let cube_geom = get_cube();
  let tet_geom = get_tetrahedron();

  let cube_buffer = BufferSet::from_mesh(& window, & cube_geom);
  let tet_buffer = BufferSet::from_mesh(& window, & tet_geom);

  // Vertex Shader
  let mut vert_shader_file = File::open("src/shader/base.vs").unwrap();
  let mut vert_shader = String::new();
  vert_shader_file.read_to_string(&mut vert_shader).unwrap();

  // Fragment Shader
  let mut frag_shader_file = File::open("src/shader/base.fs").unwrap();
  let mut frag_shader = String::new();
  frag_shader_file.read_to_string(&mut frag_shader).unwrap();

  // Shader Program
  let basic_program = glium::Program::from_source(& window, & vert_shader, & frag_shader, None).unwrap();

  // Matrices
  let mut model_rotation = 0.0;
  let per_frame_rotation = 0.002;
  let model_position = Mat4::from_translation(Vec3::new(0.0, 0.0, -5.0));

  let world_cam = Mat4::from_translation(Vec3::new(0.0, 0.0, 0.0));

  // let projection: Mat4 = cgmath::ortho(-ASPECT_RATIO * AR_SCALE, ASPECT_RATIO * AR_SCALE, -AR_SCALE, AR_SCALE, NEAR_PLANE_Z, FAR_PLANE_Z);
  let projection: Mat4 = cgmath::perspective(cgmath::Deg::new(40.0), ASPECT_RATIO, NEAR_PLANE_Z, FAR_PLANE_Z);

  // Draw Parameters
  let draw_params = glium::draw_parameters::DrawParameters {
    backface_culling: glium::draw_parameters::BackfaceCullingMode::CullClockwise,
    depth: glium::Depth {
      test: glium::DepthTest::IfLess,
      write: true,
      ..Default::default()
    },
    .. Default::default()
  };

  // Main Loop
  loop {
    let mut target = window.draw();

    target.clear_color_and_depth((0.0, 0.0, 0.0, 1.0), 1.0);

    model_rotation += per_frame_rotation;
    let rotation_quat = Quat::from_axis_angle(Vector3::new(1.0, 1.0, 0.0).normalize(), Rad::new(model_rotation));

    let model_matrix = model_position * Mat4::from(rotation_quat);

    let normal_world = (model_matrix).invert().unwrap().transpose();

    let normal_cam = (world_cam * model_matrix).invert().unwrap().transpose();

    let basic_uniforms = uniform! {
      u_model_world: mat4_uniform(& model_matrix),
      u_world_cam: mat4_uniform(& world_cam),
      u_projection: mat4_uniform(& projection),
      u_normal_world: mat4_uniform(& normal_world),
      u_normal_cam: mat4_uniform(& normal_cam)
    };

    target.draw(& tet_buffer.vertices, & tet_buffer.indices, & basic_program, & basic_uniforms, & draw_params).unwrap();

    target.draw(& cube_buffer.vertices, & cube_buffer.indices, & basic_program, & basic_uniforms, & draw_params).unwrap();

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
