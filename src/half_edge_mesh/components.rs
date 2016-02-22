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
      vert: None,
      phantom: PhantomData
    }
  }
}

pub struct FaceAdjacentVertIterator<'a> {
  start: EdgePtr,
  current: Option<EdgePtr>,
  vert: Option<VertRcPtr>,
  phantom: PhantomData<Ref<'a, Vert>>
}

impl<'a> Iterator for FaceAdjacentVertIterator<'a> {
  type Item = Ref<'a, Vert>;

  fn next(&mut self) -> Option<Ref<'a, Vert>> {

    // map: T -> U
    // and_then: T -> Option<U>
    // I think this is an equivalent calculation: 

    // self.current
    //   .and_then(|cur_weak| cur_weak.upgrade())
    //   .and_then(|cur_ref| {
    //     let new_weak: EdgePtr = cur_ref.borrow().next.clone();
    //     new_weak.upgrade()
    //       .and_then(|new_strong: Rc<RefCell<Edge>>| {
    //         if new_strong != self.start {
    //           self.current = Some(new_weak.clone());
    //           new_strong.borrow().origin.upgrade()
    //             .map(|vert_ref| vert_ref.borrow())
    //         } else { None }
    //       })
    //   })
    //   .or_else(|| {
    //     self.current = Some(self.start.clone());
    //     return self.start.upgrade()
    //       .and_then(|start_ref| start_ref.borrow().origin.upgrade())
    //       .map(|vert_ref| vert_ref.borrow());
    //   })

    if let Some(cur_weak) = self.current.clone() {
      if let Some(cur_ref) = cur_weak.upgrade() {
        let new_weak: EdgePtr = cur_ref.borrow().next.clone();
        if let (Some(new_strong), Some(start_strong)) = (new_weak.upgrade(), self.start.upgrade()) {
          if new_strong != start_strong {
            self.current = Some(new_weak.clone());
            self.vert = new_strong.borrow().origin.upgrade();
            self.vert.map(|v| Ref::clone(& v.borrow()))
          } else { None }
        } else { None }
      } else { None }
    } else {
      if let Some(start_ref) = self.start.upgrade() {
        self.current = Some(self.start.clone());
        self.vert = start_ref.borrow().origin.upgrade();
        self.vert.map(|v| Ref::clone(& v.borrow()))
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
