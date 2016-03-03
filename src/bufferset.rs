use std::collections::HashMap;

use glium::backend::Facade;
use glium::index::{PrimitiveType, IndexBuffer};
use glium::vertex::VertexBuffer;

use itertools::Zip;

use defs::*;
use mesh::Mesh;
use half_edge_mesh::{HalfEdgeMesh, ToPtrVec};

#[derive(Copy, Clone)]
#[allow(unused_attributes)]
#[repr="C"]
pub struct Vert {
  a_pos: [f32; 3],
  a_norm: [f32; 3],
}

implement_vertex!(Vert, a_pos, a_norm);

impl Vert {
  pub fn new(pos: & [f32; 3], norm: & [f32; 3]) -> Vert {
    Vert {
      a_pos: pos.clone(),
      a_norm: norm.clone(),
    }
  }

  pub fn pos_only(pos: [f32; 3]) -> Vert {
    Vert {
      a_pos: pos,
      a_norm: [0.0; 3],
    }
  }
}

pub struct BufferSet {
  pub vertices: VertexBuffer<Vert>,
  pub indices: IndexBuffer<u32>
}

impl BufferSet {
  pub fn new <T> (gl: & T, primtype: PrimitiveType) -> BufferSet
  where T: Facade {
    BufferSet {
      indices: IndexBuffer::<u32>::empty(gl, primtype, 0).unwrap(),
      vertices: VertexBuffer::<Vert>::empty(gl, 0).unwrap()
    }
  }

  pub fn from_mesh <T> (gl: & T, mesh: & Mesh) -> BufferSet
  where T: Facade {
    let i_buffer: Vec<u32> = mesh.index.iter()
      .fold(Vec::with_capacity(mesh.index.len() * 3), |mut memo, tri| {
        for & idx in tri { memo.push(idx as u32); }
        return memo;
      });

    let v_buffer: Vec<Vert> = Zip::new((mesh.vert.iter(), mesh.norm.iter()))
      .map(|(v, n): (& Pt, & Vec3)| Vert::new(v.as_ref(), n.as_ref()))
      .collect::<Vec<Vert>>();

    BufferSet {
      indices: IndexBuffer::new(gl, mesh.primitive, & i_buffer[..]).unwrap(),
      vertices: VertexBuffer::new(gl, & v_buffer[..]).unwrap(),
    }
  }

  // Creates a bufferset from a HalfEdgeMesh
  pub fn from_half_edge_mesh_flat_faces <T> (gl: & T, mesh: & HalfEdgeMesh) -> BufferSet
  where T: Facade {
    let mut i_buffer: Vec<u32> = Vec::with_capacity(mesh.faces.len() * 3);
    let mut v_buffer: Vec<Vert> = Vec::with_capacity(mesh.faces.len() * 3);
    let mut vertex_count: u32 = 0;

    for face in mesh.faces.values() {
      face.borrow_mut().compute_attrs();
      let face_borrow = face.borrow();
      let face_normal = face_borrow.normal.clone();
      for vert in face_borrow.adjacent_verts().to_ptr_vec() {
        v_buffer.push(Vert::new(vert.borrow().pos.as_ref(), face_normal.as_ref()));
        i_buffer.push(vertex_count);
        vertex_count += 1;
      }
    }

    BufferSet {
      indices: IndexBuffer::new(gl, PrimitiveType::TrianglesList, & i_buffer[..]).unwrap(),
      vertices: VertexBuffer::new(gl, & v_buffer[..]).unwrap(),
    }
  }

  pub fn from_half_edge_mesh_shared_verts <T> (gl: & T, mesh: & HalfEdgeMesh) -> BufferSet
  where T: Facade {
    let mut v_buffer: Vec<Vert> = Vec::with_capacity(mesh.vertices.len());
    let mut index_map: HashMap<u32, usize> = HashMap::new();

    for (i, vert) in mesh.vertices.values().enumerate() {
      v_buffer.push(Vert::pos_only(vert.borrow().pos.clone().into()));
      index_map.insert(vert.borrow().id, i); // Gotta convert i: usize to i: u32
    }

    let mut i_buffer: Vec<u32> = Vec::with_capacity(mesh.faces.len() * 3);
    let mut normal_counts: Vec<f32> = Vec::new();
    normal_counts.resize(v_buffer.len(), 0.0);

    for face in mesh.faces.values() {
      face.borrow_mut().compute_attrs();
      let face_normal = face.borrow().normal.clone();
      for vert in face.borrow().adjacent_verts().to_ptr_vec() {
        match index_map.get(& vert.borrow().id) {
          Some(& idx) => {
            i_buffer.push(idx as u32);
            normal_counts[idx] += 1.0;
            let a_norm = &mut v_buffer[idx].a_norm;
            a_norm[0] += face_normal.x;
            a_norm[1] += face_normal.y;
            a_norm[2] += face_normal.z;
          },
          None => (),
        }
      }
    }

    for (idx, norm_count) in normal_counts.into_iter().enumerate() {
      let a_norm = &mut v_buffer[idx].a_norm;
      a_norm[0] /= norm_count;
      a_norm[1] /= norm_count;
      a_norm[2] /= norm_count;
    }

    BufferSet {
      indices: IndexBuffer::new(gl, PrimitiveType::TrianglesList, & i_buffer[..]).unwrap(),
      vertices: VertexBuffer::new(gl, & v_buffer[..]).unwrap(),
    }
  }
}
