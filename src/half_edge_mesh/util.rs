use std::collections::HashMap;

use half_edge_mesh::components::{Edge, /*Vert,*/ Face};
use half_edge_mesh::ptr::{Ptr, EdgeRc, VertRc, FaceRc};
use half_edge_mesh::half_edge_mesh::HalfEdgeMesh;

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
    match vert_ba_key(edge) {
      Some(key) => {
        match edge_hash.get(& key) {
          Some(ref pair) => {
            if (edge.borrow().pair.upgrade().as_ref() != Some(pair)) ||
               (pair.borrow().pair.upgrade().as_ref() != Some(edge)) {
                return Err("Pairs don't match");
            }
          },
          None => { return Err("Could not find a pair edge"); }
        }
      },
      None => { return Err("Could not find reverse hash for mesh edge"); }
    }
  }

  return Ok(());
}
