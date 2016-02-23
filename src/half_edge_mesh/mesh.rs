use std::rc::Rc;
use std::cell::RefCell;

use defs::*;

use half_edge_mesh::components::{
  Edge, EdgeRcPtr,
  Vert, VertRcPtr,
  Face, FaceRcPtr,
};

pub struct Mesh {
  pub edges: Vec<EdgeRcPtr>,
  pub vertices: Vec<VertRcPtr>,
  pub faces: Vec<FaceRcPtr>,
  pub name: String,
  visited: i32,
}

impl Mesh {
  // A half-edge mesh requires at least a tetrahedron to be valid
  pub fn from_tetrahedron_pts(p1: Pt, p2: Pt, p3: Pt, p4: Pt) -> Mesh {
    unimplemented!();
  }

  pub fn from_face_vertex(vertices: & Vec<Pt>, indices: & Vec<Tri>) {
    unimplemented!();
  }
}
