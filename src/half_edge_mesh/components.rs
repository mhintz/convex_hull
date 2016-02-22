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

pub struct Edge {
  pub next: EdgePtr,
  pub pair: EdgePtr,
  pub origin: VertPtr,
  pub face: FacePtr,
  pub id: u32,
}

impl Edge {

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

pub struct FaceAdjacentVertIterator {
  start: EdgePtr,
  current: Option<EdgePtr>,
}

impl Iterator for FaceAdjacentVertIterator {
  type Item = VertPtr;

  fn next(&mut self) -> Option<VertPtr> {

    // map: Option<T>, Function<T -> U> -> Option<U>
    // and_then: Option<T>, Function<T -> Option<U> -> Option<U>
    // I think this is an equivalent calculation: 

    // self.current.clone()
    //   .and_then(|cur_weak: EdgePtr| cur_weak.upgrade())
    //   .map(|cur_rc: EdgeRcPtr| cur_rc.borrow().next.clone())
    //   .and_then(|next_weak: EdgePtr| {
    //     if let (Some(next_strong), Some(start_strong)) = (next_weak.upgrade(), self.start.upgrade()) {
    //       if next_strong != start_strong {
    //         self.current = Some(next_weak);
    //         Some(next_strong.borrow().origin.clone())
    //       } else { None }
    //     } else { None }
    //   })
    //   .or_else(|| {
    //     self.current = Some(self.start.clone());
    //     self.start.upgrade()
    //       .map(|cur_rc: EdgeRcPtr| cur_rc.borrow().origin.clone())
    //   })

    if let Some(cur_weak) = self.current.clone() {
      if let Some(cur_rc) = cur_weak.upgrade() {
        let new_weak: EdgePtr = cur_rc.borrow().next.clone();
        if let (Some(new_strong), Some(start_strong)) = (new_weak.upgrade(), self.start.upgrade()) {
          if new_strong != start_strong {
            self.current = Some(new_weak);
            Some(new_strong.borrow().origin.clone())
          } else { None }
        } else { None }
      } else { None }
    } else {
      if let Some(start_strong) = self.start.upgrade() {
        self.current = Some(self.start.clone());
        Some(start_strong.borrow().origin.clone())
      } else { None }
    }

  }

}

// pub struct VertAdjacentVertIterator {
//   start: EdgePtr,
//   current: EdgePtr,
// }

// impl  Iterator for VertAdjacentVertIterator {
//   type Item = Vert;

//   fn next(&mut self) -> Option<Vert> {
//     unimplemented!();
//   }
// }
