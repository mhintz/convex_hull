use std::rc::{Rc, Weak};

#[allow(unused_imports)]
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

pub struct EdgeAdjacentEdgeIterator {
  vert_iter_1: Option<VertAdjacentEdgeIterator>,
  vert_iter_2: Option<VertAdjacentEdgeIterator>,
  state: DualIterState,
}

// Implementation here is borrowed from std::iter::Chain
#[derive(Clone)]
enum DualIterState {
  // both iterators running
  Both,
  // only first running
  First,
  // only second running
  Second,
  // neither works
  // (this doesn't exist on the chain iterator,
  // because both must be valid iterators,
  // but it can exist here, in case the weak pointers fail to upgrade)
  Neither
}

impl EdgeAdjacentEdgeIterator {
  pub fn new(target: & Edge) -> EdgeAdjacentEdgeIterator {
    let iter_1_opt: Option<VertAdjacentEdgeIterator> = target.origin.upgrade()
      .map(|vert_ptr: VertRcPtr| vert_ptr.borrow().edge.clone())
      .map(|vert_edge: EdgePtr| VertAdjacentEdgeIterator::new(vert_edge));

    let iter_2_opt: Option<VertAdjacentEdgeIterator> = target.next.upgrade()
      .and_then(|edge_next: EdgeRcPtr| edge_next.borrow().origin.upgrade())
      .map(|vert_ptr: VertRcPtr| vert_ptr.borrow().edge.clone())
      .map(|vert_edge: EdgePtr| VertAdjacentEdgeIterator::new(vert_edge));

    let state = match (iter_1_opt.as_ref(), iter_2_opt.as_ref()) {
      (Some(_), Some(_)) => DualIterState::Both,
      (Some(_), None) => DualIterState::First,
      (None, Some(_)) => DualIterState::Second,
      (None, None) => DualIterState::Neither
    }; // <-- because this match is an assignment statement, this semicolon is essential

    EdgeAdjacentEdgeIterator {
      state: state,
      vert_iter_1: iter_1_opt,
      vert_iter_2: iter_2_opt
    }
  }
}

impl Iterator for EdgeAdjacentEdgeIterator {
  type Item = EdgePtr;

  fn next(&mut self) -> Option<EdgePtr> {
    match self.state {
      DualIterState::Both => {
        match self.vert_iter_1.as_mut().unwrap().next() {
          val @ Some(..) => val,
          None => {
            self.state = DualIterState::Second;
            self.vert_iter_2.as_mut().unwrap().next()
          }
        }
      },
      DualIterState::First => self.vert_iter_1.as_mut().unwrap().next(),
      DualIterState::Second => self.vert_iter_2.as_mut().unwrap().next(),
      DualIterState::Neither => None,
    }
  }
}

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