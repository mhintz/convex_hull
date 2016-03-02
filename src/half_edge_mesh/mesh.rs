use std::collections::HashMap;

use defs::*;

use half_edge_mesh::components::{Edge, Vert, Face};
use half_edge_mesh::ptr::{Ptr, EdgeRc, VertRc, FaceRc, EdgePtr, VertPtr, FacePtr};
use half_edge_mesh::iterators::ToPtrVec;

/// Half-Edge Mesh data structure
/// While it's possible to create non-triangular faces, this code assumes
/// triangular faces in several locations
pub struct HalfEdgeMesh {
  pub edges: HashMap<u32, EdgeRc>,
  pub vertices: HashMap<u32, VertRc>,
  pub faces: HashMap<u32, FaceRc>,
}

// Takes three Rc<RefCell<Vert>>,
// creates three edges and one face, and connects them as well as it can
// Note: since this creates a lone triangle, edge.pair links are
// still empty after this function
pub fn make_triangle(p1: & VertRc, p2: & VertRc, p3: & VertRc) -> (FaceRc, EdgeRc, EdgeRc, EdgeRc) {
  // Create triangle edges
  let e1 = Ptr::new_rc(Edge::with_origin(Ptr::new(& p1)));
  let e2 = Ptr::new_rc(Edge::with_origin(Ptr::new(& p2)));
  let e3 = Ptr::new_rc(Edge::with_origin(Ptr::new(& p3)));

  // Be sure to set up vertex connectivity with the new edges
  // It doesn't matter which edge a vertex points to,
  // so long as it points back to the vertex
  p1.borrow_mut().take_edge(Ptr::new(& e1));
  p2.borrow_mut().take_edge(Ptr::new(& e2));
  p3.borrow_mut().take_edge(Ptr::new(& e3));

  // Set up edge cycle
  e1.borrow_mut().take_next(Ptr::new(& e2));
  e2.borrow_mut().take_next(Ptr::new(& e3));
  e3.borrow_mut().take_next(Ptr::new(& e1));

  // Create triangle face
  let f1 = Ptr::new_rc(Face::with_edge(Ptr::new(& e1)));

  // Set up face links
  e1.borrow_mut().take_face(Ptr::new(& f1));
  e2.borrow_mut().take_face(Ptr::new(& f1));
  e3.borrow_mut().take_face(Ptr::new(& f1));

  // Now is the right time to run this, since vertices and edges are connected
  f1.borrow_mut().compute_attrs();

  (f1, e1, e2, e3)
}

fn merge_tuple_opt<A, B>(o: (Option<A>, Option<B>)) -> Option<(A, B)> {
  match o {
    (Some(a), Some(b)) => Some((a, b)),
    _ => None
  }
}

fn vert_ab_key(e: & EdgeRc) -> Option<(u32, u32)> {
  let id_origin = e.borrow().origin.upgrade().map(|o| o.borrow().id);
  let id_next_origin = e.borrow().next.upgrade().and_then(|n| n.borrow().origin.upgrade()).map(|o| o.borrow().id);
  merge_tuple_opt((id_origin, id_next_origin))
}

fn vert_ba_key(e: & EdgeRc) -> Option<(u32, u32)> { vert_ab_key(e).map(|tuple| (tuple.1, tuple.0)) }

// Takes what is assumed to be a fully-connected mesh, with no
// pair links, and establishes pair links between adjacent edges
pub fn connect_pairs(mesh: &mut HalfEdgeMesh) -> Result<(), &'static str> {
  // Two-stage algorithm: first collect all edge A -> B relationships,
  // Then go through and look for edges that are B -> A
  let mut edge_hash: HashMap<(u32, u32), & EdgeRc> = HashMap::new();

  for ref edge in mesh.edges.values() {
    // The types returned by match arms must be the same,
    // hence the braces and semicolon used in the first branch
    match vert_ab_key(edge) {
      Some(key) => { edge_hash.insert(key, edge); },
      // This happens if one of the mesh edges doesn't have a valid .origin or .next.origin pointer
      None => { return Err("Could not hash all mesh edges"); }
    }
  }

  for ref edge in mesh.edges.values() {
    // This if statement should skip half the edges, because two
    // edge pairs are set each time it's true
    if !edge.borrow().pair.is_valid() {
      if let Some(key) = vert_ba_key(edge) {
        match edge_hash.get(& key) {
          Some(pair_edge) => {
            // if one edge A -> B matches another edge B -> A, the edges are adjacent
            edge.borrow_mut().take_pair(Ptr::new(pair_edge));
            pair_edge.borrow_mut().take_pair(Ptr::new(edge));
          },
          None => { /* Happens when mesh is not closed */
            return Err("Could not find pair edge");
          }
        }
      } else {
        // Theoretically this shouldn't ever happen
        // because of the early return in the previous match block
        return Err("Could not find reverse hash for mesh edge");
      }
    }
  }

  return Ok(());
}

// Checks if edge pair connections are all valid
pub fn are_edge_pairs_valid(mesh: & HalfEdgeMesh) -> Result<(), &'static str> {
  unimplemented!();
}

fn report_connect_err(res: Result<(), &str>) {
  match res {
    Err(e) => println!("Error connecting mesh pairs! Mesh is not valid! {}", e),
    _ => {},
  }
}

impl HalfEdgeMesh {
  pub fn empty() -> HalfEdgeMesh {
    HalfEdgeMesh { edges: HashMap::new(), vertices: HashMap::new(), faces: HashMap::new(), }
  }

  // A half-edge mesh requires at least a tetrahedron to be valid
  // p1: apex, p2: bottom left front, p3: bottom right front, p4: bottom rear
  pub fn from_tetrahedron_pts(p1: Pt, p2: Pt, p3: Pt, p4: Pt) -> HalfEdgeMesh {
    // In progress
    let mut mesh = HalfEdgeMesh::empty();

    let v1 = Ptr::new_rc(Vert::empty(p1));
    let v2 = Ptr::new_rc(Vert::empty(p2));
    let v3 = Ptr::new_rc(Vert::empty(p3));
    let v4 = Ptr::new_rc(Vert::empty(p4));

    mesh.add_triangle(make_triangle(& v1, & v2, & v3));
    mesh.add_triangle(make_triangle(& v2, & v1, & v4));
    mesh.add_triangle(make_triangle(& v3, & v4, & v1));
    mesh.add_triangle(make_triangle(& v4, & v3, & v2));

    mesh.move_verts(vec![v1, v2, v3, v4]);

    report_connect_err(connect_pairs(&mut mesh));

    return mesh;
  }

  // p1: top apex, p2: mid left front, p3: mid right front, p4: mid left back, p5: mid right back, p6: bottom apex
  pub fn from_octahedron_pts(p1: Pt, p2: Pt, p3: Pt, p4: Pt, p5: Pt, p6: Pt) -> HalfEdgeMesh {
    let mut mesh = HalfEdgeMesh::empty();

    let v1 = Ptr::new_rc(Vert::empty(p1));
    let v2 = Ptr::new_rc(Vert::empty(p2));
    let v3 = Ptr::new_rc(Vert::empty(p3));
    let v4 = Ptr::new_rc(Vert::empty(p4));
    let v5 = Ptr::new_rc(Vert::empty(p5));
    let v6 = Ptr::new_rc(Vert::empty(p6));

    mesh.add_triangle(make_triangle(& v1, & v2, & v3));
    mesh.add_triangle(make_triangle(& v1, & v4, & v2));
    mesh.add_triangle(make_triangle(& v1, & v3, & v5));
    mesh.add_triangle(make_triangle(& v1, & v5, & v4));
    mesh.add_triangle(make_triangle(& v6, & v3, & v2));
    mesh.add_triangle(make_triangle(& v6, & v2, & v4));
    mesh.add_triangle(make_triangle(& v6, & v5, & v3));
    mesh.add_triangle(make_triangle(& v6, & v4, & v5));

    mesh.move_verts(vec![v1, v2, v3, v4, v5, v6]);

    report_connect_err(connect_pairs(&mut mesh));

    return mesh;
  }

  pub fn from_face_vertex_mesh(vertices: & Vec<Pt>, indices: & Vec<Tri>) {
    unimplemented!();
  }

  pub fn push_edge(&mut self, edge: EdgeRc) {
    let key = edge.borrow().id;
    self.edges.insert(key, edge);
  }

  pub fn extend_edges(&mut self, edges: & [EdgeRc]) {
    for edge in edges {
      let key = edge.borrow().id;
      self.edges.insert(key, edge.clone());
    }
  }

  pub fn move_edges(&mut self, edges: Vec<EdgeRc>) {
    for edge in edges {
      let key = edge.borrow().id;
      self.edges.insert(key, edge);
    }
  }

  pub fn push_vert(&mut self, vert: VertRc) {
    let key = vert.borrow().id;
    self.vertices.insert(key, vert);
  }

  pub fn extend_verts(&mut self, verts: & [VertRc]) {
    for vert in verts {
      let key = vert.borrow().id;
      self.vertices.insert(key, vert.clone());
    }
  }

  pub fn move_verts(&mut self, verts: Vec<VertRc>) {
    for vert in verts {
      let key = vert.borrow().id;
      self.vertices.insert(key, vert);
    }
  }

  pub fn push_face(&mut self, face: FaceRc) {
    let key = face.borrow().id;
    self.faces.insert(key, face);
  }

  pub fn extend_faces(&mut self, faces: & [FaceRc]) {
    for face in faces {
      let key = face.borrow().id;
      self.faces.insert(key, face.clone());
    }
  }

  pub fn move_faces(&mut self, faces: Vec<FaceRc>) {
    for face in faces {
      let key = face.borrow().id;
      self.faces.insert(key, face);
    }
  }

  pub fn add_triangle(&mut self, triangle: (FaceRc, EdgeRc, EdgeRc, EdgeRc)) {
    let mut key: u32;

    key = triangle.0.borrow().id;
    self.faces.insert(key, triangle.0);

    key = triangle.1.borrow().id;
    self.edges.insert(key, triangle.1);

    key = triangle.2.borrow().id;
    self.edges.insert(key, triangle.2);

    key = triangle.3.borrow().id;
    self.edges.insert(key, triangle.3);
  }

  // Checks if two faces are adjacent by looking for a shared edge
  pub fn are_faces_adjacent(& self, face_l: & FaceRc, face_r: & FaceRc) -> bool {
    unimplemented!();
  }

  pub fn are_face_ptrs_adjacent(& self, face_l: & FacePtr, face_r: & FacePtr) -> bool {
    match Ptr::merge_upgrade(face_l, face_r) {
      Some((l_rc, r_rc)) => self.are_faces_adjacent(& l_rc, & r_rc),
      None => false,
    }
  }

  // Replace a face with three faces, each connected to the new point
  // And one of the face's previous vertices
  pub fn triangulate_face(&mut self, point: Pt, face: & FaceRc) {
    // get face edges
    let face_edges = face.borrow().adjacent_edges().to_ptr_vec();
    // get face vertexes, assumed to be counter-clockwise
    let face_vertices = face.borrow().adjacent_verts().to_ptr_vec();
    let vertices_len = face_vertices.len();

    debug_assert_eq!(vertices_len, 3); // should be 3, or else your faces aren't triangles

    let apex_vert = Ptr::new_rc(Vert::empty(point));

    // Add the three new faces - one attached to each of the original face's edges,
    // plus two new edges attached to the point
    let mut new_lead_edges: Vec<EdgeRc> = Vec::new();
    let mut new_trail_edges: Vec<EdgeRc> = Vec::new();
    for (i, base_edge) in face_edges.iter().enumerate() {
      // Might not be necessary
      base_edge.borrow_mut().take_origin(Ptr::new(& face_vertices[i]));
      base_edge.borrow().origin.upgrade().map(|o| o.borrow_mut().take_edge(Ptr::new(base_edge)));

      let new_face = Ptr::new_rc(Face::with_edge(Ptr::new(base_edge)));
      let leading_edge = Ptr::new_rc(Edge::with_origin(Ptr::new(& face_vertices[(i + 1) % vertices_len])));
      let trailing_edge = Ptr::new_rc(Edge::with_origin(Ptr::new(& apex_vert)));

      base_edge.borrow_mut().take_face(Ptr::new(& new_face));
      leading_edge.borrow_mut().take_face(Ptr::new(& new_face));
      trailing_edge.borrow_mut().take_face(Ptr::new(& new_face));

      base_edge.borrow_mut().take_next(Ptr::new(& leading_edge));
      leading_edge.borrow_mut().take_next(Ptr::new(& trailing_edge));
      trailing_edge.borrow_mut().take_next(Ptr::new(base_edge));

      apex_vert.borrow_mut().take_edge(Ptr::new(& trailing_edge));

      new_lead_edges.push(leading_edge.clone());
      new_trail_edges.push(trailing_edge.clone());

      self.push_edge(leading_edge);
      self.push_edge(trailing_edge);
      self.push_face(new_face);
    }

    let trail_edge_len = new_trail_edges.len();

    // Should be 3, or else the faces are not triangular, or not enough edges were created
    debug_assert_eq!(trail_edge_len, 3);
    debug_assert_eq!(new_lead_edges.len(), 3);

    // Connect pairs
    for (i, leading_edge) in new_lead_edges.iter().enumerate() {
      let trailing_edge = & new_trail_edges[(i + 1) % trail_edge_len];
      leading_edge.borrow_mut().take_pair(Ptr::new(& trailing_edge));
      trailing_edge.borrow_mut().take_pair(Ptr::new(& leading_edge));
    }

    // Remove the face and the edges from the mesh.
    // When the local pointer to this falls out of scope, it should be deallocated
    self.faces.remove(& face.borrow().id);
  }

  pub fn triangulate_face_ptr(&mut self, point: Pt, face: & FacePtr) {
    match face.upgrade() {
      Some(face_rc) => self.triangulate_face(point, & face_rc),
      None => (),
    }
  }

  // Attach a point to a mesh, replacing many faces (used for the convex hull algorithm)
  // The faces should be a continuously connected group, each adjacent pair of vertices
  // in the border of this group are connected to the point in a new triangular face.
  pub fn attach_point_for_faces(&mut self, point: Pt, faces: & Vec<FaceRc>) {
    unimplemented!();
  }

  pub fn attach_point_for_face_ptrs(&mut self, point: Pt, faces: & Vec<FacePtr>) {
    let face_ptrs = faces.iter().filter_map(|f| f.upgrade()).collect::<Vec<FaceRc>>();
    self.attach_point_for_faces(point, & face_ptrs);
  }

  // This function should only work if the vertex has exactly three adjacent edges.
  // Therefore, it has three adjacent faces.
  // The vertices connected to those edges form a new face, and the faces and edges connected
  // to the removed vertex are also removed
  pub fn remove_point(&mut self, point: & VertRc) {
    unimplemented!();
  }

  pub fn remove_point_ptr(&mut self, point: & VertPtr) {
    match point.upgrade() {
      Some(point_rc) => self.remove_point(& point_rc),
      None => (),
    }
  }

  // flips an edge between two faces so that the faces are each split by
  // the other diagonal of the parallelogram they form.
  pub fn flip_edge(&mut self, edge: & EdgeRc) {
    unimplemented!();
  }

  pub fn flip_edge_ptr(&mut self, edge: & EdgePtr) {
    match edge.upgrade() {
      Some(edge_rc) => self.flip_edge(& edge_rc),
      None => (),
    }
  }

  // Inserts a vertex at the position, specified by tval, along edge.origin -> edge.next.origin
  // The edge's two neighboring faces are each split into two faces.
  // All four new faces include the new vertex
  pub fn split_edge(&mut self, edge: & EdgeRc, tval: f32) {
    unimplemented!();
  }

  pub fn split_edge_rc(&mut self, edge: & EdgePtr, tval: f32) {
    match edge.upgrade() {
      Some(edge_rc) => self.split_edge(& edge_rc, tval),
      None => (),
    }
  }
}
