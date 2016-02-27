use defs::*;

use half_edge_mesh::components::{Edge, Vert, Face};
use half_edge_mesh::ptr::{Ptr, EdgeRc, VertRc, FaceRc};

pub struct Mesh {
  pub edges: Vec<EdgeRc>,
  pub vertices: Vec<VertRc>,
  pub faces: Vec<FaceRc>,
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
