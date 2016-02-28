use std::rc::{Rc, Weak};
use std::cell::RefCell;

use half_edge_mesh::components::{Edge, Vert, Face};

pub type EdgePtr = Ptr<Edge>;
pub type EdgeRc = Rc<RefCell<Edge>>;
pub type VertPtr = Ptr<Vert>;
pub type VertRc = Rc<RefCell<Vert>>;
pub type FacePtr = Ptr<Face>;
pub type FaceRc = Rc<RefCell<Face>>;

pub struct Ptr<T> {
  val: Option<Weak<RefCell<T>>>
}

fn getup<T>(v: & Weak<RefCell<T>>) -> Option<Rc<RefCell<T>>> { v.upgrade() }

impl<T> Ptr<T> {
  pub fn new_rc(val: T) -> Rc<RefCell<T>> { Rc::new(RefCell::new(val)) }

  pub fn new(val: & Rc<RefCell<T>>) -> Ptr<T> {
    Ptr { val: Some(Rc::downgrade(val)) }
  }

  pub fn empty() -> Ptr<T> {
    Ptr { val: None }
  }

  pub fn merge_upgrade(weak_a: & Ptr<T>, weak_b: & Ptr<T>) -> Option<(Rc<RefCell<T>>, Rc<RefCell<T>>)> {
    match (weak_a.upgrade(), weak_b.upgrade()) {
      (Some(strong_a), Some(strong_b)) => Some((strong_a, strong_b)),
      _ => None
    }
  }

  // Is it a bad idea to call upgrade() in this function? Too expensive?
  pub fn is_valid(& self) -> bool { self.val.is_some() && self.upgrade().is_some() }

  pub fn upgrade(& self) -> Option<Rc<RefCell<T>>> {
    self.val.as_ref().and_then(getup)
  }

  // pub fn as_ref(& self) -> Option<& Weak<RefCell<T>>> {
  //   self.val.as_ref()
  // }
}

impl<T> Clone for Ptr<T> {
  fn clone(& self) -> Self {
    Ptr { val: self.val.clone() }
  }
}

