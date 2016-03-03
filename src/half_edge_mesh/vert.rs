use std;

use defs::*;

use half_edge_mesh::ptr::EdgePtr;
use half_edge_mesh::iterators::*;

static mut vert_id: u32 = 0;

fn get_vert_id() -> u32 { unsafe { vert_id += 1; vert_id } }

pub struct Vert {
  pub edge: EdgePtr,
  pub pos: Pt,
  pub id: u32,
}

impl Vert {
  // All structure of the mesh revolves around vertex positions and their connectivity.
  // (Faces are just an abstraction). All vertices must therefore have a concrete position.
  pub fn empty(pos: Pt) -> Vert {
    Vert {
      id: get_vert_id(),
      edge: EdgePtr::empty(),
      pos: pos,
    }
  }

  // Vertex connected to an existing edge
  pub fn with_edge(pos: Pt, edge: EdgePtr) -> Vert {
    Vert {
      id: get_vert_id(),
      edge: edge,
      pos: pos,
    }
  }

  pub fn take_edge(&mut self, edge: EdgePtr) { self.edge = edge; }

  pub fn set_edge(&mut self, edge: & EdgePtr) { self.edge = edge.clone(); }

  pub fn move_to(&mut self, pos: Pt) { self.pos = pos; }

  pub fn get_pos(& self) -> Pt { self.pos }

  pub fn is_valid(& self) -> bool { self.edge.is_valid() }

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

impl Eq for Vert {}

impl std::hash::Hash for Vert {
  fn hash<H>(& self, state: &mut H) where H: std::hash::Hasher {
    state.write_u32(self.id);
    state.finish();
  }
}

impl std::fmt::Debug for Vert {
  fn fmt(& self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
    write!(fmt, "Vert {{ id: {} }}", self.id)
  }
}
