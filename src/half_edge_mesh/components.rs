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

  pub fn set_next(&mut self, next: & EdgePtr) { self.next = next.clone(); }

  pub fn set_pair(&mut self, pair: & EdgePtr) { self.pair = pair.clone(); }

  pub fn set_origin(&mut self, origin: & VertPtr) { self.origin = origin.clone(); }

  pub fn set_face(&mut self, face: & FacePtr) { self.face = face.clone(); }

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

  pub fn set_edge(&mut self, edge: & EdgePtr) { self.edge = edge.clone(); }

  pub fn is_valid(& self) -> bool { self.edge.is_valid() }

  pub fn num_vertices(& self) -> usize { self.adjacent_verts().count() }

  pub fn compute_attrs(&mut self) {
    let mut center = Pt::origin();
    let mut count: f32 = 0.0;

    let vert_list = self.adjacent_verts()
      .filter_map(|v| v.upgrade())
      .collect::<Vec<VertRc>>();

    for vert in & vert_list {
      let pos = vert.borrow().get_pos();
      center.x += pos.x;
      center.y += pos.y;
      center.z += pos.z;
      count += 1.0;
    }

    self.center = center / count;

    let vertA = vert_list[0].borrow().get_pos();
    let s1 = vert_list[1].borrow().get_pos() - vertA;
    let s2 = vert_list[2].borrow().get_pos() - vertA;
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
