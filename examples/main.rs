#[macro_use]
extern crate glium;
extern crate cgmath;
extern crate convex_hull;
extern crate rand;
extern crate time;

use std::fs::File;
use std::io::Read;

use glium::glutin;
use glium::{DisplayBuild, Surface};

use cgmath::prelude::*;
use cgmath::{Vector3, Rad};

use rand::distributions::Sample;

use convex_hull::defs::*;
use convex_hull::bufferset::*;

const WINDOW_WIDTH: u32 = 800;
const WINDOW_HEIGHT: u32 = 800;
const ASPECT_RATIO: f32 = (WINDOW_WIDTH as f32) / (WINDOW_HEIGHT as f32);
// const AR_SCALE: f32 = 1.3;
const NEAR_PLANE_Z: f32 = 0.5;
const FAR_PLANE_Z: f32 = 1000.0;
const NUM_POINTS: usize = 500;

fn rand_points_in_cube<R: rand::Rng>(num_gen: &mut R, num: usize, side: f32) -> Vec<Pt> {
  let mut rand_range = rand::distributions::Range::new(-side * 0.5, side * 0.5);
  let mut points = Vec::with_capacity(num);
  for idx in 0..num {
    points.insert(idx, Pt::new(rand_range.sample(num_gen), rand_range.sample(num_gen), rand_range.sample(num_gen)));
  }
  deduplicate_points_list(&mut points);
  return points;
}

fn rand_points_in_sphere<R: rand::Rng>(num_gen: &mut R, num: usize, radius: f32) -> Vec<Pt> {
  let mut vec_range = rand::distributions::Range::new(-1.0, 1.0);
  let mut radius_range = rand::distributions::Range::new(0.0, radius);
  let mut points = Vec::with_capacity(num);
  for idx in 0..num {
    let point_vec = Vec3::new(vec_range.sample(num_gen), vec_range.sample(num_gen), vec_range.sample(num_gen));
    let radius = radius_range.sample(num_gen);
    let rand_point = Pt::from_vec(point_vec.normalize() * radius);
    points.insert(idx, rand_point);
  }
  deduplicate_points_list(&mut points);
  return points;
}

fn rand_points_on_sphere<R: rand::Rng>(num_gen: &mut R, num: usize, radius: f32) -> Vec<Pt> {
  let mut rand_range = rand::distributions::Range::new(-1.0, 1.0);
  let mut points = Vec::with_capacity(num);
  for idx in 0..num {
    let point_vec = Vec3::new(rand_range.sample(num_gen), rand_range.sample(num_gen), rand_range.sample(num_gen));
    let rand_point = Pt::from_vec(point_vec.normalize() * radius);
    points.insert(idx, rand_point);
  }
  deduplicate_points_list(&mut points);
  return points;
}

fn same_point(num: usize) -> Vec<Pt> {
  let mut points = Vec::with_capacity(num);
  for idx in 0..num {
    points.insert(idx, Pt::new(1.0, 1.0, 1.0));
  }
  return points;
}

// sorts and deduplicates a list of input points
fn deduplicate_points_list(list: &mut Vec<Pt>) {
  use std::cmp::Ordering::{Less, Equal};

  list.sort_by(|a, b| {
    match a.x.partial_cmp(& b.x).unwrap_or(Less) {
      Equal => {
        match a.y.partial_cmp(& b.y).unwrap_or(Less) {
          Equal => a.z.partial_cmp(& b.z).unwrap_or(Less),
          y => y,
        }
      },
      x => x,
    }
  });

  list.dedup();
}

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
  let mut rand_rng = rand::thread_rng();
  // let random_points = rand_points_in_cube(&mut rand_rng, NUM_POINTS, 1.0);
  let random_points = rand_points_in_sphere(&mut rand_rng, NUM_POINTS, 1.0);
  // let random_points = rand_points_on_sphere(&mut rand_rng, NUM_POINTS, 1.0);
  // let random_points = same_point(NUM_POINTS);

  let start = time::get_time();
  let hull_mesh = convex_hull::get_convex_hull(random_points);
  let duration = time::get_time() - start;
  println!("convex hull computation took: {} seconds", duration);

  let hull_mesh_buffer = BufferSet::from_half_edge_mesh_flat_faces(& window, & hull_mesh);

  // Vertex Shader
  let mut vert_shader_file = File::open("examples/shader/base.vs").unwrap();
  let mut vert_shader = String::new();
  vert_shader_file.read_to_string(&mut vert_shader).unwrap();

  // Fragment Shader
  let mut frag_shader_file = File::open("examples/shader/base.fs").unwrap();
  let mut frag_shader = String::new();
  frag_shader_file.read_to_string(&mut frag_shader).unwrap();

  // Shader Program
  let basic_program = glium::Program::from_source(& window, & vert_shader, & frag_shader, None).unwrap();

  // Matrices
  let mut model_rotation = 0.0;
  let per_frame_rotation = 0.002;
  let model_position = Mat4::from_translation(Vec3::new(0.0, 0.0, -5.0));

  let world_cam = Mat4::from_translation(Vec3::new(0.0, 0.0, 0.0));

  // let ortho_projection: Mat4 = cgmath::ortho(-ASPECT_RATIO * AR_SCALE, ASPECT_RATIO * AR_SCALE, -AR_SCALE, AR_SCALE, NEAR_PLANE_Z, FAR_PLANE_Z);
  let perspective_projection: Mat4 = cgmath::perspective(cgmath::Deg::new(40.0), ASPECT_RATIO, NEAR_PLANE_Z, FAR_PLANE_Z);

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
      u_projection: mat4_uniform(& perspective_projection),
      u_normal_world: mat4_uniform(& normal_world),
      u_normal_cam: mat4_uniform(& normal_cam)
    };

    // target.draw(& tet_he_mesh_buffer.vertices, & tet_he_mesh_buffer.indices, & basic_program, & basic_uniforms, & draw_params).unwrap();

    // target.draw(& oct_he_mesh_buffer.vertices, & oct_he_mesh_buffer.indices, & basic_program, & basic_uniforms, & draw_params).unwrap();

    target.draw(& hull_mesh_buffer.vertices, & hull_mesh_buffer.indices, & basic_program, & basic_uniforms, & draw_params).unwrap();

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
