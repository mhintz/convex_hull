#[macro_use]
extern crate glium;
extern crate cgmath;
extern crate itertools;

pub mod bufferset;
pub mod defs;
pub mod half_edge_mesh;
pub mod mesh;
pub mod convex_hull;
pub use convex_hull::get_convex_hull;
