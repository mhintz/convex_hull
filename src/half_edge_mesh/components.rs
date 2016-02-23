use std::rc::Rc;
use std::rc::Weak;
use std::cell::RefCell;
use std::cell::Ref;
use std::marker::PhantomData;

// TODO: make this class generic over point type, vector type, and vertex data type
use defs::*;

pub type EdgePtr = Weak<RefCell<Edge>>;
pub type EdgeRcPtr = Rc<RefCell<Edge>>;
pub type VertPtr = Weak<RefCell<Vert>>;
pub type VertRcPtr = Rc<RefCell<Vert>>;
pub type FacePtr = Weak<RefCell<Face>>;
pub type FaceRcPtr = Rc<RefCell<Face>>;

fn merge_upgrade<T>(weak_a: & Weak<T>, weak_b: & Weak<T>) -> Option<(Rc<T>, Rc<T>)> {
  match (weak_a.upgrade(), weak_b.upgrade()) {
    (Some(strong_a), Some(strong_b)) => Some((strong_a, strong_b)),
    _ => None
  }
}

pub struct Edge {
  pub next: EdgePtr,
  pub pair: EdgePtr,
  pub origin: VertPtr,
  pub face: FacePtr,
  pub id: u32,
}

impl Edge {
  pub fn adjacent_verts<'a> (&'a self) -> EdgeAdjacentVertIterator<'a> {
    EdgeAdjacentVertIterator {
      count: 0,
      start: self,
    }
  }
}

impl PartialEq<Edge> for Edge {
  fn eq(& self, other: & Edge) -> bool { self.id == other.id }
}

pub struct EdgeAdjacentVertIterator<'a> {
  count: u8,
  start: &'a Edge,
}

impl<'a> Iterator for EdgeAdjacentVertIterator<'a> {
  type Item = VertPtr;

  fn next(&mut self) -> Option<VertPtr> {
    match self.count {
      0 => {
        self.count += 1;
        Some(self.start.origin.clone())
        // self.start.upgrade().map(|start_rc| start_rc.borrow().origin.clone())
      },
      1 => {
        self.count += 1;
        self.start.next.upgrade()
          .map(|next_rc| next_rc.borrow().origin.clone())
        // self.start.upgrade()
        //   .and_then(|start_rc| start_rc.borrow().next.upgrade())
        //   .map(|next_rc| next_rc.borrow().origin.clone())
      },
      _ => None,
    }
  }
}

pub struct Vert {
  pub edge: EdgePtr,
  pub pos: Pt,
  pub id: u32,
}

impl Vert {
  pub fn adjacent_verts(& self) -> VertAdjacentVertIterator {
    VertAdjacentVertIterator {
      start: self.edge.clone(),
      current: None,
    }
  }
}

impl PartialEq<Vert> for Vert {
  fn eq(& self, other: & Vert) -> bool { self.id == other.id }
}

pub struct VertAdjacentVertIterator {
  start: EdgePtr,
  current: Option<EdgePtr>,
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

pub struct Face {
  pub edge: EdgePtr,
  pub normal: Vec3,
  pub center: Pt,
  pub id: u32,
}

impl Face {
  pub fn adjacent_verts(& self) -> FaceAdjacentVertIterator {
    FaceAdjacentVertIterator {
      start: self.edge.clone(),
      current: None,
    }
  }
}

impl PartialEq<Face> for Face {
  fn eq(& self, other: & Face) -> bool { self.id == other.id }
}

pub struct FaceAdjacentVertIterator {
  start: EdgePtr,
  current: Option<EdgePtr>,
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
