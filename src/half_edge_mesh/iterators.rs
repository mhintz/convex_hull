use std::rc::{Rc, Weak};
use std::cell::RefCell;

use defs::*;

use half_edge_mesh::components::{
  Edge, EdgePtr, EdgeRcPtr,
  Vert, VertPtr, VertRcPtr,
  Face, FacePtr, FaceRcPtr,
};

fn merge_upgrade<T>(weak_a: & Weak<T>, weak_b: & Weak<T>) -> Option<(Rc<T>, Rc<T>)> {
  match (weak_a.upgrade(), weak_b.upgrade()) {
    (Some(strong_a), Some(strong_b)) => Some((strong_a, strong_b)),
    _ => None
  }
}

fn merge_options<T>(opt_a: Option<T>, opt_b: Option<T>) -> Option<(T, T)> {
  match (opt_a, opt_b) {
    (Some(val_a), Some(val_b)) => Some((val_a, val_b)),
    _ => None
  }
}

// EdgeIterators

pub struct EdgeAdjacentVertIterator<'a> {
  count: u8,
  start: &'a Edge,
}

impl<'a> EdgeAdjacentVertIterator<'a> {
  pub fn new(target: &'a Edge) -> EdgeAdjacentVertIterator<'a> {
    EdgeAdjacentVertIterator {
      count: 0,
      start: target,
    }
  }
}

impl<'a> Iterator for EdgeAdjacentVertIterator<'a> {
  type Item = VertPtr;

  fn next(&mut self) -> Option<VertPtr> {
    match self.count {
      0 => {
        self.count += 1;
        Some(self.start.origin.clone())
      },
      1 => {
        self.count += 1;
        self.start.next.upgrade()
          .map(|next_rc| next_rc.borrow().origin.clone())
      },
      _ => None,
    }
  }
}

pub struct EdgeAdjacentEdgeIterator;

use std::iter;

impl EdgeAdjacentEdgeIterator {
  pub fn new<'a> (target: &'a Edge) -> iter::Chain<VertAdjacentEdgeIterator, VertAdjacentEdgeIterator> {
    let edge_1_opt: Option<EdgePtr> = target.origin.upgrade()
      .map(|vert_ptr_1: VertRcPtr| vert_ptr_1.borrow().edge.clone());

    let edge_2_opt: Option<EdgePtr> = target.next.upgrade()
      .and_then(|edge_next: EdgeRcPtr| edge_next.borrow().origin.upgrade())
      .map(|vert_ptr_2: VertRcPtr| vert_ptr_2.borrow().edge.clone());

    return merge_options(edge_1_opt, edge_2_opt)
      .map_or_else(|| {
        iter::empty().chain(iter::empty())
      }, |res: (EdgePtr, EdgePtr)| {
        let edge_1: EdgePtr = res.0;
        let edge_2: EdgePtr = res.1;
        VertAdjacentEdgeIterator::new(edge_1.clone()).chain(VertAdjacentEdgeIterator::new(edge_2.clone()))
      });
  }
}

// impl Iterator for EdgeAdjacentEdgeIterator {
//   type Item = EdgePtr;

//   fn next(&mut self) -> Option<EdgePtr> {

//   }
// }

// VertIterators

pub struct VertAdjacentVertIterator {
  start: EdgePtr,
  current: Option<EdgePtr>,
}

impl VertAdjacentVertIterator {
  pub fn new(edge: EdgePtr) -> VertAdjacentVertIterator {
    VertAdjacentVertIterator {
      start: edge,
      current: None,
    }
  }
}

impl Iterator for VertAdjacentVertIterator {
  type Item = VertPtr;

  // edge.pair.origin, edge = edge.pair.next, edge != start
  fn next(&mut self) -> Option<VertPtr> {
    return self.current.clone()
      .and_then(|cur_weak: EdgePtr| cur_weak.upgrade())
      .and_then(|cur_rc: EdgeRcPtr| cur_rc.borrow().pair.upgrade())
      .and_then(|pair_rc: EdgeRcPtr| {
        let next_weak: EdgePtr = pair_rc.borrow().next.clone();
        return merge_upgrade(& next_weak, & self.start)
          .and_then(|(next_rc, start_rc)| {
            if next_rc != start_rc {
              self.current = Some(next_weak);
              Some(pair_rc.borrow().origin.clone())
            } else { None }
          });
      })
      .or_else(|| {
        self.current = Some(self.start.clone());
        return self.start.upgrade()
          .and_then(|cur_rc: EdgeRcPtr| cur_rc.borrow().pair.upgrade())
          .map(|pair_rc: EdgeRcPtr| pair_rc.borrow().origin.clone());
      })
  }
}

pub struct VertAdjacentEdgeIterator {
  start: EdgePtr,
  current: Option<EdgePtr>,
}

impl VertAdjacentEdgeIterator {
  pub fn new(edge: EdgePtr) -> VertAdjacentEdgeIterator {
    VertAdjacentEdgeIterator {
      start: edge,
      current: None
    }
  }
}

impl Iterator for VertAdjacentEdgeIterator {
  type Item = EdgePtr;

  fn next(&mut self) -> Option<EdgePtr> {
    return self.current.clone()
      .and_then(|cur_weak: EdgePtr| cur_weak.upgrade())
      .and_then(|cur_rc: EdgeRcPtr| cur_rc.borrow().pair.upgrade())
      .map(|pair_rc: EdgeRcPtr| pair_rc.borrow().next.clone())
      .and_then(|next_weak: EdgePtr| {
        return merge_upgrade(& next_weak, & self.start)
          .and_then(|(next_rc, start_rc)| {
            if next_rc != start_rc {
              self.current = Some(next_weak.clone());
              Some(next_weak.clone())
            } else { None }
          });
      })
      .or_else(|| {
        self.current = Some(self.start.clone());
        Some(self.start.clone())
      });
  }
}

// FaceIterators

pub struct FaceAdjacentVertIterator {
  start: EdgePtr,
  current: Option<EdgePtr>,
}

impl FaceAdjacentVertIterator {
  pub fn new(edge: EdgePtr) -> FaceAdjacentVertIterator {
    FaceAdjacentVertIterator {
      start: edge,
      current: None,
    }
  }
}

impl Iterator for FaceAdjacentVertIterator {
  type Item = VertPtr;

  // edge.origin, edge = edge.next, edge != start
  fn next(&mut self) -> Option<VertPtr> {
    // map: Option<T>, Function<T -> U> -> Option<U>
    // and_then: Option<T>, Function<T -> Option<U>> -> Option<U>
    return self.current.clone()
      .and_then(|cur_weak: EdgePtr| cur_weak.upgrade())
      .map(|cur_rc: EdgeRcPtr| cur_rc.borrow().next.clone())
      .and_then(|next_weak: EdgePtr| {
        return merge_upgrade(& next_weak, & self.start)
          .and_then(|(next_rc, start_rc)| {
            if next_rc != start_rc {
              self.current = Some(next_weak);
              Some(next_rc.borrow().origin.clone())
            } else { None }            
          });
      })
      .or_else(|| {
        return self.start.upgrade()
          .map(|cur_rc: EdgeRcPtr| {
            self.current = Some(self.start.clone());
            cur_rc.borrow().origin.clone()
          });
      });

    // This should be an equivalent calculation to the above:
    // if let Some(cur_weak) = self.current.clone() {
    //   if let Some(cur_rc) = cur_weak.upgrade() {
    //     let new_weak: EdgePtr = cur_rc.borrow().next.clone();
    //     if let (Some(next_rc), Some(start_rc)) = (new_weak.upgrade(), self.start.upgrade()) {
    //       if next_rc != start_rc {
    //         self.current = Some(new_weak);
    //         Some(next_rc.borrow().origin.clone())
    //       } else { None }
    //     } else { None }
    //   } else { None }
    // } else {
    //   if let Some(start_rc) = self.start.upgrade() {
    //     self.current = Some(self.start.clone());
    //     Some(start_rc.borrow().origin.clone())
    //   } else { None }
    // }
  }
}
