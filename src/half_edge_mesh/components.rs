use std;
// TODO: make these classes generic over point type, vector type, and vertex data type
use defs::*;
use cgmath::Point; // Needed for Pt::origin()
use cgmath::EuclideanVector;
// Import the pointer types
use half_edge_mesh::ptr::{EdgePtr, /*EdgeRc,*/ VertPtr, VertRc, FacePtr, /*FaceRc*/};
// Import the iterator structs
use half_edge_mesh::iterators::*;

// Please, don't make more than 2^32-1 edges, vertices, or faces
// TODO: better ids (mesh-specific?)
// Maybe use this: https://crates.io/crates/snowflake
static mut edge_id: u32 = 0;
static mut vert_id: u32 = 0;
static mut face_id: u32 = 0;

fn get_edge_id() -> u32 { unsafe { edge_id += 1; edge_id } }
fn get_vert_id() -> u32 { unsafe { vert_id += 1; vert_id } }
fn get_face_id() -> u32 { unsafe { face_id += 1; face_id } }

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

  pub fn take_pair(&mut self, pair: EdgePtr) { self.pair = pair; }

  pub fn set_pair(&mut self, pair: & EdgePtr) { self.pair = pair.clone(); }

  pub fn take_origin(&mut self, origin: VertPtr) { self.origin = origin; }

  pub fn set_origin(&mut self, origin: & VertPtr) { self.origin = origin.clone(); }

  pub fn set_face(&mut self, face: & FacePtr) { self.face = face.clone(); }

  pub fn take_face(&mut self, face: FacePtr) { self.face = face; }

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

pub struct Face {
  pub edge: EdgePtr,
  pub normal: Vec3,
  pub center: Pt,
  pub id: u32,
}

impl Face {
  pub fn empty() -> Face {
    Face {
      id: get_face_id(),
      edge: EdgePtr::empty(),
      // Are these sensible defaults?
      // Are these values even necessary?
      normal: Vec3::unit_z(),
      center: Pt::origin(),
    }
  }

  // Face connected to an existing edge
  pub fn with_edge(edge: EdgePtr) -> Face {
    Face {
      id: get_face_id(),
      edge: edge,
      normal: Vec3::unit_z(),
      center: Pt::origin(),
    }
  }

  pub fn take_edge(&mut self, edge: EdgePtr) { self.edge = edge; }

  pub fn set_edge(&mut self, edge: & EdgePtr) { self.edge = edge.clone(); }

  pub fn is_valid(& self) -> bool { self.edge.is_valid() }

  pub fn num_vertices(& self) -> usize { self.adjacent_verts().count() }

  // Note: this only works when edges and verts are properly connected
  // So wait for the right time during initialization to run this
  pub fn compute_attrs(&mut self) {
    let mut center = Pt::origin();
    let mut count: f32 = 0.0;

    let vert_list: Vec<VertRc> = self.adjacent_verts().to_ptr_vec();

    for vert in & vert_list {
      let pos = vert.borrow().get_pos();
      center.x += pos.x;
      center.y += pos.y;
      center.z += pos.z;
      count += 1.0;
    }

    self.center = center / count;

    let vert_a = vert_list[0].borrow().get_pos();
    let s1 = vert_list[1].borrow().get_pos() - vert_a;
    let s2 = vert_list[2].borrow().get_pos() - vert_a;
    self.normal = s1.cross(s2).normalize();
  }

  pub fn adjacent_verts(& self) -> FaceAdjacentVertIterator {
    FaceAdjacentVertIterator::new(self.edge.clone())
  }

  pub fn adjacent_edges(& self) -> FaceAdjacentEdgeIterator {
    FaceAdjacentEdgeIterator::new(self.edge.clone())
  }

  pub fn adjacent_faces(& self) -> FaceAdjacentFaceIterator {
    FaceAdjacentFaceIterator::new(self.edge.clone())
  }
}

impl PartialEq<Face> for Face {
  fn eq(& self, other: & Face) -> bool { self.id == other.id }
}

impl Eq for Face {}

impl std::hash::Hash for Face {
  fn hash<H>(& self, state: &mut H) where H: std::hash::Hasher {
    state.write_u32(self.id);
    state.finish();
  }
}

impl std::fmt::Debug for Face {
  fn fmt(& self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
    write!(fmt, "Face {{ id: {} }}", self.id)
  }
}
