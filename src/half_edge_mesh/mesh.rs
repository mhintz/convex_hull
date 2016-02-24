use defs::*;

#[allow(unused_imports)]
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
}

impl Mesh {
  // A half-edge mesh requires at least a tetrahedron to be valid
  #[allow(unused_variables)]
  pub fn from_tetrahedron_pts(p1: Pt, p2: Pt, p3: Pt, p4: Pt) -> Mesh {
    unimplemented!();
  }

  #[allow(unused_variables)]
  pub fn from_face_vertex(vertices: & Vec<Pt>, indices: & Vec<Tri>) {
    unimplemented!();
  }
}
