use std::rc::{Rc, Weak};
use std::cell::RefCell;

// TODO: make these classes generic over point type, vector type, and vertex data type
use defs::*;
use half_edge_mesh::iterators::*;

pub type EdgePtr = Weak<RefCell<Edge>>;
pub type EdgeRcPtr = Rc<RefCell<Edge>>;
pub type VertPtr = Weak<RefCell<Vert>>;
pub type VertRcPtr = Rc<RefCell<Vert>>;
pub type FacePtr = Weak<RefCell<Face>>;
pub type FaceRcPtr = Rc<RefCell<Face>>;

pub struct Edge {
  pub next: EdgePtr,
  pub pair: EdgePtr,
  pub origin: VertPtr,
  pub face: FacePtr,
  pub id: u32,
}

impl Edge {
  pub fn adjacent_verts<'a> (&'a self) -> EdgeAdjacentVertIterator<'a> {
    EdgeAdjacentVertIterator::new(self)
  }

  pub fn adjacent_edges(& self) -> EdgeAdjacentEdgeIterator {
    EdgeAdjacentEdgeIterator::new(self)
  }

  pub fn adjacent_faces<'a>(&'a self) -> EdgeAdjacentFaceIterator<'a> {
    EdgeAdjacentFaceIterator::new(self)
  }
}

impl PartialEq<Edge> for Edge {
  fn eq(& self, other: & Edge) -> bool { self.id == other.id }
}

pub struct Vert {
  pub edge: EdgePtr,
  pub pos: Pt,
  pub id: u32,
}

impl Vert {
  pub fn adjacent_verts(& self) -> VertAdjacentVertIterator {
    VertAdjacentVertIterator::new(self.edge.clone())
  }

  pub fn adjacent_edges(& self) -> VertAdjacentEdgeIterator {
    VertAdjacentEdgeIterator::new(self.edge.clone())
  }

  pub fn adjacent_faces(& self) -> VertAdjacentFaceIterator {
    VertAdjacentFaceIterator::new(self.edge.clone())
  }
}

impl PartialEq<Vert> for Vert {
  fn eq(& self, other: & Vert) -> bool { self.id == other.id }
}

pub struct Face {
  pub edge: EdgePtr,
  pub normal: Vec3,
  pub center: Pt,
  pub id: u32,
}

impl Face {
  pub fn adjacent_verts(& self) -> FaceAdjacentVertIterator {
    FaceAdjacentVertIterator::new(self.edge.clone())
  }

  pub fn adjacent_edges(& self) -> FaceAdjacentEdgeIterator {
    FaceAdjacentEdgeIterator::new(self.edge.clone())
  }

  // pub fn adjacent_faces(& self) -> FaceAdjacentFaceIterator {
  //   FaceAdjacentFaceIterator::new()
  // }
}

impl PartialEq<Face> for Face {
  fn eq(& self, other: & Face) -> bool { self.id == other.id }
}
