use glium::index::{PrimitiveType};

// Needed for normalize
use cgmath::EuclideanVector;
use cgmath::Vector;

use defs::*;

pub struct Mesh {
  pub vert: Vec<Pt>,
  pub norm: Vec<Vec3>,
  pub index: Vec<Tri>,
  pub primitive: PrimitiveType,
}

impl Mesh {
  pub fn new(primtype: PrimitiveType) -> Mesh {
    Mesh {
      vert: Vec::<Pt>::new(),
      norm: Vec::<Vec3>::new(),
      index: Vec::<Tri>::new(),
      primitive: primtype
    }
  }

  pub fn add_vert(&mut self, v: Pt) {
    self.vert.push(v);
  }

  pub fn add_norm(&mut self, n: Vec3) {
    self.norm.push(n);
  }

  pub fn add_tri(&mut self, f: Tri) {
    self.index.push(f);
  }

  pub fn add_triangle(&mut self, verts: [Pt; 3], norms: [Vec3; 3]) {
    match self.primitive {
      PrimitiveType::TrianglesList => {
        for (& vertex, & normal) in verts.iter().zip(norms.iter()) {
          self.add_vert(vertex);
          self.add_norm(normal);
        }
        let ln = self.vert.len();
        self.add_tri([ln - 3, ln - 2, ln - 1]);
      },
      _ => panic!("add_triangle not implemented for the primitive type you provided")
    }
  }
}

pub fn get_normal(vtxs: & Vec<Pt>, idxs: & Tri) -> Vec3 {
  return get_verts_normal(& vtxs[idxs[0]], & vtxs[idxs[1]], & vtxs[idxs[2]]);
}

pub fn get_verts_normal(vert0: & Pt, vert1: & Pt, vert2: & Pt) -> Vec3 {
  let s1 = vert1 - vert0;
  let s2 = vert2 - vert0;
  let norm: Vec3 = s1.cross(s2);
  return norm.normalize();
}

pub fn construct_normals<'a>(vertices: & Vec<Pt>, indices: & Vec<Tri>) -> Vec<Vec3> {
  let mut normals: Vec<Vec3> = Vec::new();
  normals.resize(vertices.len(), Vec3::zero());

  let mut sums: Vec<i32> = Vec::new();
  sums.resize(vertices.len(), 0);

  for tri in indices {
    let norm = get_verts_normal(& vertices[tri[0]], & vertices[tri[1]], & vertices[tri[2]]);
    for & i in tri {
      normals[i] = normals[i] + norm;
      sums[i] += 1;
    }
  }

  let normalized = normals.iter().zip(sums.iter()).map(|(& norm, & sum)| norm / (sum as f32));

  return normalized.collect::<Vec<Vec3>>();
}

pub fn get_tetrahedron() -> Mesh {
  unimplemented!();
}

pub fn get_cube() -> Mesh {
  let mut cube = Mesh::new(PrimitiveType::TrianglesList);

  let irad = (1_f32 / 3_f32).sqrt();
  let diag = (2_f32 / 3_f32).sqrt();
  let zero = 0_f32;

  cube.add_vert(Pt::new(zero, irad, diag));
  cube.add_vert(Pt::new(diag, irad, zero));
  cube.add_vert(Pt::new(zero, irad, -diag));
  cube.add_vert(Pt::new(-diag, irad, zero));
  cube.add_vert(Pt::new(-diag, -irad, zero));
  cube.add_vert(Pt::new(zero, -irad, diag));
  cube.add_vert(Pt::new(diag, -irad, zero));
  cube.add_vert(Pt::new(zero, -irad, -diag));

  // front top
  cube.add_tri([0, 1, 2]);
  cube.add_tri([0, 2, 3]);

  // // front bottom right
  cube.add_tri([0, 6, 1]);
  cube.add_tri([0, 5, 6]);

  // // front bottom left
  cube.add_tri([0, 3, 4]);
  cube.add_tri([0, 4, 5]);

  // // back top right
  cube.add_tri([7, 2, 1]);
  cube.add_tri([7, 1, 6]);

  // // back bottom
  cube.add_tri([7, 6, 5]);
  cube.add_tri([7, 5, 4]);

  // // back top left
  cube.add_tri([7, 4, 3]);
  cube.add_tri([7, 3, 2]);

  cube.norm = construct_normals(& cube.vert, & cube.index);

  return cube;
}
