use std::f32;

use glium::index::{PrimitiveType};

// Needed for normalize
use cgmath::EuclideanVector;
use cgmath::Vector;

use defs::*;

use half_edge_mesh::HalfEdgeMesh;

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

  pub fn from_half_edge_mesh(he_mesh: & HalfEdgeMesh) -> Mesh {
    let mut mesh = Mesh::new(PrimitiveType::TrianglesList);

    for face in he_mesh.faces.iter() {
      face.borrow_mut().compute_attrs();
      let face_norm = face.borrow().normal.clone();
      for vert in face.borrow().adjacent_verts().filter_map(|v| v.upgrade()) {
        mesh.add_vert(vert.borrow().pos.clone());
        mesh.add_norm(face_norm);
      }
      let ln = mesh.vert.len();
      mesh.add_tri([ln - 3, ln - 2, ln - 1]);
    }

    return mesh;
  }

  pub fn add_vert(&mut self, v: Pt) {
    self.vert.push(v);
  }

  pub fn add_norm(&mut self, n: Vec3) {
    self.norm.push(n);
  }

  pub fn add_tri(&mut self, t: Tri) {
    self.index.push(t);
  }

  pub fn add_triangle(&mut self, verts: & [Pt; 3], norms: & [Vec3; 3]) {
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
  norm.normalize();
  return norm;
}

pub fn split_mesh_vertices(mesh: & Mesh) -> Mesh {
  let mut split_mesh = Mesh::new(PrimitiveType::TrianglesList);

  for tri in & mesh.index {
    split_mesh.add_vert(mesh.vert[tri[0]]);
    split_mesh.add_vert(mesh.vert[tri[1]]);
    split_mesh.add_vert(mesh.vert[tri[2]]);
    let ln = split_mesh.vert.len();
    split_mesh.add_tri([ln - 3, ln - 2, ln - 1]);
  }

  return split_mesh;
}

pub fn construct_normals(vertices: & Vec<Pt>, indices: & Vec<Tri>) -> Vec<Vec3> {
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
  let mut tet = Mesh::new(PrimitiveType::TrianglesList);

  let two_pi: f32 = 2.0 * f32::consts::PI;
  let a0: f32 = two_pi * 0.0;
  let a1: f32 = two_pi * 1.0 / 3.0;
  let a2: f32 = two_pi * 2.0 / 3.0;
  // When the circumradius of the tetrahedron is 1, the height is 4 / 3
  let height: f32 = 4.0 / 3.0;
  let distance_bottom: f32 = 1.0 - height; // Distance to the 'back' of the tetrahedron
  let distance_point: f32 = (0.5_f32).sqrt() * height;

  tet.add_vert(Pt::new(0.0, 1.0, 0.0));
  tet.add_vert(Pt::new(distance_point * a0.sin(), distance_bottom, distance_point * a0.cos()));
  tet.add_vert(Pt::new(distance_point * a1.sin(), distance_bottom, distance_point * a1.cos()));
  tet.add_vert(Pt::new(distance_point * a2.sin(), distance_bottom, distance_point * a2.cos()));

  tet.add_tri([0, 1, 2]);
  tet.add_tri([2, 3, 0]);
  tet.add_tri([3, 1, 0]);
  tet.add_tri([1, 3, 2]);

  tet = split_mesh_vertices(& tet);
  tet.norm = construct_normals(& tet.vert, & tet.index);

  return tet;
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

  cube = split_mesh_vertices(& cube);
  cube.norm = construct_normals(& cube.vert, & cube.index);

  return cube;
}
