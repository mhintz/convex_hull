use std;

use half_edge_mesh::ptr::{Ptr, EdgePtr, VertPtr, FacePtr, EdgeRc, VertRc, FaceRc};
use half_edge_mesh::iterators::*;

// Please, don't make more than 2^32-1 edges, vertices, or faces
// TODO: better ids (mesh-specific?)
// Maybe use this: https://crates.io/crates/snowflake
static mut edge_id: u32 = 0;

fn get_edge_id() -> u32 { unsafe { edge_id += 1; edge_id } }

pub struct Edge {
  pub next: EdgePtr,
  pub pair: EdgePtr,
  pub origin: VertPtr,
  pub face: FacePtr,
  pub id: u32,
}

impl Edge {
  pub fn empty() -> Edge {
    Edge {
      id: get_edge_id(),
      next: EdgePtr::empty(),
      pair: EdgePtr::empty(),
      origin: VertPtr::empty(),
      face: FacePtr::empty(),
    }
  }

  pub fn with_origin(origin: VertPtr) -> Edge {
    Edge {
      id: get_edge_id(),
      next: EdgePtr::empty(),
      pair: EdgePtr::empty(),
      origin: origin,
      face: FacePtr::empty(),
    }
  }

  pub fn take_next(&mut self, next: EdgePtr) { self.next = next; }

  pub fn set_next(&mut self, next: & EdgePtr) { self.next = next.clone(); }

  pub fn set_next_rc(&mut self, next: & EdgeRc) { self.next = Ptr::new(next); }

  pub fn take_pair(&mut self, pair: EdgePtr) { self.pair = pair; }

  pub fn set_pair(&mut self, pair: & EdgePtr) { self.pair = pair.clone(); }

  pub fn set_pair_rc(&mut self, pair: & EdgeRc) { self.pair = Ptr::new(pair); }

  pub fn take_origin(&mut self, origin: VertPtr) { self.origin = origin; }

  pub fn set_origin(&mut self, origin: & VertPtr) { self.origin = origin.clone(); }

  pub fn set_origin_rc(&mut self, origin: & VertRc) { self.origin = Ptr::new(origin); }

  pub fn set_face(&mut self, face: & FacePtr) { self.face = face.clone(); }

  pub fn take_face(&mut self, face: FacePtr) { self.face = face; }

  pub fn set_face_rc(&mut self, face: & FaceRc) { self.face = Ptr::new(face); }

  // The tests in this function are in order of "subjective likeliness of being invalid"
  pub fn is_valid(& self) -> bool { self.pair.is_valid() && self.face.is_valid() && self.origin.is_valid() && self.next.is_valid() }

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

impl Eq for Edge {}

impl std::hash::Hash for Edge {
  fn hash<H>(& self, state: &mut H) where H: std::hash::Hasher {
    state.write_u32(self.id);
    state.finish();
  }
}

impl std::fmt::Debug for Edge {
  fn fmt(& self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
    write!(fmt, "Edge {{ id: {} }}", self.id)
  }
}