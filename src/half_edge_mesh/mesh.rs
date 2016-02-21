use std::rc::Rc;
use std::cell::RefCell;

use defs::*;

use half_edge_mesh::components::Edge;
use half_edge_mesh::components::Vert;
use half_edge_mesh::components::Face;

pub struct Mesh {
  pub edges: Vec<Rc<RefCell<Edge>>>,
  pub vertices: Vec<Rc<RefCell<Vert>>>,
  pub faces: Vec<Rc<RefCell<Face>>>,
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
