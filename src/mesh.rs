use std::f32;

use glium::index::{PrimitiveType};

// Needed for normalize
use cgmath::EuclideanVector;

use defs::*;

pub struct Mesh {
  pub vert: Vec<Pt>,
  pub norm: Vec<Vec3>,
  pub index: Vec<u32>,
  pub primitive: PrimitiveType,
}

impl Mesh {
  pub fn new(primtype: PrimitiveType) -> Mesh {
    Mesh {
      vert: Vec::<Pt>::new(),
      norm: Vec::<Vec3>::new(),
      index: Vec::<u32>::new(),
      primitive: primtype
    }
  }

  pub fn add_vert(&mut self, v: Pt) {
    self.vert.push(v);
  }

  pub fn add_norm(&mut self, n: Vec3) {
    self.norm.push(n);
  }

  pub fn add_index(&mut self, i: u32) {
    self.index.push(i);
  }

  pub fn add_face(&mut self, f: [u32; 3]) {
    self.index.extend_from_slice(& f);
  }

  pub fn add_triangle(&mut self, verts: [Pt; 3], norms: [Vec3; 3]) {
    match self.primitive {
      PrimitiveType::TrianglesList => {
        for (& vertex, & normal) in verts.iter().zip(norms.iter()) {
          let len = self.vert.len();
          self.add_index(len as u32);
          self.add_vert(vertex);
          self.add_norm(normal);
        }
      },
      _ => panic!("add_triangle not implemented for the primitive type you provided")
    }
  }
}

/*
export function genFaceNormalsPerVertex(vertices, triangles) {
  let normals = triangles.reduce((list, triangle) => {
    let norm = getNormal(vertices, triangle);

    list[triangle[0]] = norm;
    // Clone for other cases
    list[triangle[1]] = vec3.clone(norm);
    list[triangle[2]] = vec3.clone(norm);

    return list;
  }, []);

  return normals;
}

 */

pub fn construct_normals(vertices: & Vec<Pt>, indices: & Vec<u32>) -> Vec<Vec3> {
  unimplemented!();
  // indices.iter().chunks_lazy(3).reduce(|triangle| {

  // })
}

pub fn get_tetrahedron() -> Mesh {
  unimplemented!();
}

pub fn get_cube() -> Mesh {
  let mut cube = Mesh::new(PrimitiveType::TrianglesList);

  let bounds = [-1_f32, 1_f32];
  // Make it a real cube here ...
  for & x in bounds.iter() {
    for & y in bounds.iter() {
      for & z in bounds.iter() {
        cube.add_vert(Pt::new(x, y, z));
        cube.add_norm(Vec3::new(x, y, z).normalize());
        println!("{}, {}, {}", x, y, z);
      }
    }
  }

  let iratio = 1_f32 / 3_f32;
  let irad = iratio.sqrt();
  let diagratio = 2_f32 / 3_f32;
  let diag = diagratio.sqrt();
  let zero = 0_f32;

  let v0 = Pt::new(zero, irad, diag);
  let v1 = Pt::new(diag, irad, zero);
  let v2 = Pt::new(zero, irad, -diag);
  let v3 = Pt::new(-diag, irad, zero);
  let v4 = Pt::new(-diag, -irad, zero);
  let v5 = Pt::new(zero, -irad, diag);
  let v6 = Pt::new(diag, -irad, zero);
  let v7 = Pt::new(zero, -irad, -diag);

// 0, 1, 2

// 2, 3, 0
// 3, 7, 4
// 4, 0, 3
// 4, 5, 1
// 1, 0, 4




  // front top
  // triangles.push([0, 1, 2]);
  // triangles.push([0, 2, 3]);

  // // front bottom right
  // triangles.push([0, 6, 1]);
  // triangles.push([0, 5, 6]);

  // // front bottom left
  // triangles.push([0, 3, 4]);
  // triangles.push([0, 4, 5]);

  // // back top right
  // triangles.push([7, 2, 1]);
  // triangles.push([7, 1, 6]);

  // // back bottom
  // triangles.push([7, 6, 5]);
  // triangles.push([7, 5, 4]);

  // // back top left
  // triangles.push([7, 4, 3]);
  // triangles.push([7, 3, 2]);




// n
// a, m, m
// a, 
// 0, 1, 2

//   cube.add_face([0, 1, 2]);
// // 1, 3, 2
// 2, 



  return cube;
}
