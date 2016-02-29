use std::collections::HashMap;

use defs::*;

use half_edge_mesh::components::{Edge, Vert, Face};
use half_edge_mesh::ptr::{Ptr, EdgeRc, VertRc, FaceRc};

pub struct HalfEdgeMesh {
  pub edges: Vec<EdgeRc>,
  pub vertices: Vec<VertRc>,
  pub faces: Vec<FaceRc>,
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

  for ref edge in mesh.edges.iter() {
    // The types returned by match arms must be the same,
    // hence the braces and semicolon used in the first branch
    match vert_ab_key(edge) {
      Some(key) => { edge_hash.insert(key, edge); },
      // This happens if one of the mesh edges doesn't have a valid .origin or .next.origin pointer
      None => { return Err("Could not hash all mesh edges"); }
    }
  }

  for ref edge in mesh.edges.iter() {
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

fn report_connect_err(res: Result<(), &str>) {
  match res {
    Err(e) => println!("Error connecting mesh pairs! Mesh is not valid! {}", e),
    _ => {},
  }
}

impl HalfEdgeMesh {
  pub fn empty() -> HalfEdgeMesh {
    HalfEdgeMesh { edges: vec![], vertices: vec![], faces: vec![], }
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

  pub fn push_edge(&mut self, edge: EdgeRc) { self.edges.push(edge); }

  pub fn extend_edges(&mut self, edges: & [EdgeRc]) { for edge in edges { self.edges.push(edge.clone()); } }

  pub fn move_edges(&mut self, edges: Vec<EdgeRc>) { for edge in edges { self.edges.push(edge); } }

  pub fn push_vert(&mut self, vert: VertRc) { self.vertices.push(vert); }

  pub fn extend_verts(&mut self, verts: & [VertRc]) { for vert in verts { self.vertices.push(vert.clone()); } }

  pub fn move_verts(&mut self, verts: Vec<VertRc>) { for vert in verts { self.vertices.push(vert); } }

  pub fn push_face(&mut self, face: FaceRc) { self.faces.push(face); }

  pub fn extend_faces(&mut self, faces: & [FaceRc]) { for face in faces { self.faces.push(face.clone()); } }

  pub fn move_faces(&mut self, faces: Vec<FaceRc>) { for face in faces { self.faces.push(face); } }

  pub fn add_triangle(&mut self, triangle: (FaceRc, EdgeRc, EdgeRc, EdgeRc)) {
    self.faces.push(triangle.0);
    self.edges.push(triangle.1);
    self.edges.push(triangle.2);
    self.edges.push(triangle.3);
  }
}
