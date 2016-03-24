use std::collections::LinkedList;

use defs::*;

use half_edge_mesh::{HalfEdgeMesh, FaceRc};

use cgmath::{EuclideanVector, Vector, Point};

// Original Java implementation of this function in comments
// distSqPointSegment(float[] a, float[] b, float[] c)
fn line_to_pt_dist_sq(pt1: Pt, pt2: Pt, target: Pt) -> f32 {
  // float[] ab = DwVec3.sub_new(b,a);
  let line = pt2 - pt1;
  // float[] ac = DwVec3.sub_new(c,a);
  let p1_to_target = target - pt1;
  // float[] bc = DwVec3.sub_new(c,b);
  let p2_to_target = target - pt2;

  // float e = DwVec3.dot(ac, ab);
  let p1_t_on_line = p1_to_target.dot(line);
  // if (e < 0.0f) return DwVec3.dot(ac,ac);
  if p1_t_on_line < 0.0 { return p1_to_target.length2(); }
  // float f = DwVec3.dot(ab, ab);
  let line_length2 = line.length2();
  // if (e >= f) return DwVec3.dot(bc,bc);
  if p1_t_on_line >= line_length2 { return p2_to_target.length2(); }
  // return DwVec3.dot(ac,ac) - e * e / f;
  return p1_to_target.length2() - p1_t_on_line * p1_t_on_line / line_length2;
}

fn triangle_normal(pt1: Pt, pt2: Pt, pt3: Pt) -> Vec3 {
  let side_1 = pt2 - pt1;
  let side_2 = pt3 - pt1;
  side_1.cross(side_2).normalize()
}

fn triangle_center(pt1: Pt, pt2: Pt, pt3: Pt) -> Pt {
  Pt::from_vec((pt1.to_vec() + pt2.to_vec() + pt3.to_vec()) / 3.0)
}

#[derive(Copy, Clone, Debug)]
struct Pair {
  pub idx: usize,
  pub pt: Pt
}

impl Pair {
  fn new(i: usize, p: Pt) -> Pair { Pair { idx: i, pt: p } }
}

fn update_min_max(idx_pair: Pair, start: usize, pairs: &mut [Pair; 6]) {
  let mut replaced = false;

  match start {
    0 => {
      if idx_pair.pt.x < pairs[0].pt.x {
        update_min_max(pairs[0], 1, pairs);
        pairs[0] = idx_pair;
        replaced = true;
      }
    },
    1 => {
      if idx_pair.pt.x > pairs[1].pt.x {
        update_min_max(pairs[1], 2, pairs);
        pairs[1] = idx_pair;
        replaced = true;
      }
    },
    2 => {
      if idx_pair.pt.y < pairs[2].pt.y {
        update_min_max(pairs[2], 3, pairs);
        pairs[2] = idx_pair;
        replaced = true;
      }
    },
    3 => {
      if idx_pair.pt.y > pairs[3].pt.y {
        update_min_max(pairs[3], 4, pairs);
        pairs[3] = idx_pair;
        replaced = true;
      }
    },
    4 => {
      if idx_pair.pt.z < pairs[4].pt.z {
        update_min_max(pairs[4], 5, pairs);
        pairs[4] = idx_pair;
        replaced = true;
      }
    },
    5 => {
      if idx_pair.pt.z > pairs[5].pt.z {
        pairs[5] = idx_pair;
        replaced = true;
      }
    },
    _ => { return; },
  }

  if !replaced {
    update_min_max(idx_pair, start + 1, pairs);
  }
}

fn construct_tetrahedron_order(p0: Pair, p1: Pair, p2: Pair, p3: Pair) -> Vec<usize> {
  let base_norm = triangle_normal(p0.pt, p1.pt, p2.pt);
  let to_tri = (p0.pt - p3.pt).normalize();
  if to_tri.dot(base_norm) < 0.0 {
    // The face normal and the vector to the face are pointing in opposite directions,
    // So we can conclude that the triangle is facing towards the fourth point.
    // Meaning we know that the triangle's vertices are in counterclockwise order when seen from the fourth point
    vec![p3.idx, p0.idx, p1.idx, p2.idx]
  } else {
    // The face normal, and the vector to the face, are pointing in the same direction,
    // Therefore, the triangle is facing away from the fourth point, meaning we know
    // that the triangle's vertices are in clockwise order when seen from the fourth point
    // The order of indices for the tetrahedron should be apex, front left, front right, back
    vec![p3.idx, p1.idx, p0.idx, p2.idx]
  }
}

// TODO: this algorithm for choosing the initial tetrahedron points needs to be rethought.
// For one, it doesn't take into account the edge case where some of the max values are the same
// Point. For another, it doesn't handle certain edge cases that arise with degenerate positioning
// of the tetrahedron points. Rethink this algorithm for better robustness.
fn get_extreme_points(list: & Vec<Pt>) -> Vec<usize> {
  debug_assert!(list.len() >= 4);

  let mut boundaries = [Pair::new(0, list[0]); 6];

  for (i, pt) in list.iter().cloned().enumerate() {
    update_min_max(Pair::new(i, pt), 0, &mut boundaries);
  }

  let mut p0 = boundaries[0];
  let mut p1 = boundaries[1];
  let mut pt_dist_sq_max = (p0.pt - p1.pt).length2();
  for (idx_a, pair_a) in boundaries.iter().enumerate() {
    for pair_b in boundaries[(idx_a + 1)..].iter() {
      let dist = (pair_a.pt - pair_b.pt).length2();
      if dist > pt_dist_sq_max {
        pt_dist_sq_max = dist;
        p0 = pair_a.clone();
        p1 = pair_b.clone();
      }
    }
  }

  let mut p2 = boundaries[0];
  let mut line_dist_sq_max = 0.0;
  for pair in boundaries.iter() {
    if pair.idx == p0.idx || pair.idx == p1.idx { continue; }
    let dist = line_to_pt_dist_sq(p0.pt, p1.pt, pair.pt);
    if dist > line_dist_sq_max {
      line_dist_sq_max = dist;
      p2 = pair.clone();
    }
  }

  let mut p3 = boundaries[0];
  let mut tri_dist_sq_max = 0.0;
  let face_center = triangle_center(p0.pt, p1.pt, p2.pt);
  for pair in boundaries.iter() {
    if pair.idx == p0.idx || pair.idx == p1.idx || pair.idx == p2.idx { continue; }
    let dist = (pair.pt - face_center).length2();
    if dist > tri_dist_sq_max {
      tri_dist_sq_max = dist;
      p3 = pair.clone();
    }
  }

  return construct_tetrahedron_order(p0, p1, p2, p3);
}

// Build a convex hull. Takes the points list by move because it needs to mutate the list
// Either the caller makes a clone of the list to pass in, or the function takes a reference
// and clones it. Either way, a clone is necessary, and this way the caller can just use a
// throwaway vector.
pub fn get_convex_hull(mut points_list: Vec<Pt>) -> HalfEdgeMesh {
  // Check that we have a valid list of points
  if points_list.len() < 4 { return HalfEdgeMesh::empty(); }
  // Get the tetrahedron of the points at maxX, maxY, maxZ, and minZ
  // These points are on the hull.
  let mut tet_points = get_extreme_points(& points_list);
  // This is the starting point of the mesh
  let mut hull_mesh = HalfEdgeMesh::from_tetrahedron_pts(points_list[tet_points[0]], points_list[tet_points[1]], points_list[tet_points[2]], points_list[tet_points[3]]);

  // Remove them from the list of points, since for performance it's important to minimise the size of this list
  // let mut remove_indexes: Vec<usize> = (& tet_points).into(); // vec![tet_points[0], tet_points[1], tet_points[2], tet_points[3]];
  tet_points.sort_by(|a, b| a.cmp(b).reverse());
  tet_points.dedup();
  for i in tet_points.into_iter() {
    // Remove in descending order, to not disturb the indexes of the list
    if i < points_list.len() {
      points_list.remove(i);
    }
  }

  // Filter the points list based on whether the point is inside the tetrahedron.
  // This saves a lot of iteration steps. It removes any points from the points list
  // Which cannot be seen by any face (i.e. they are behind all faces in the tetrahedron)
  points_list.retain(|p| {
    hull_mesh.faces.values().any(|f| {
      f.borrow().can_see(& p)
    })
  });

  // Add all faces of the hull to a FIFO queue
  let mut face_queue: LinkedList<FaceRc> = hull_mesh.faces.values().cloned().collect();

  // While the queue has faces, iterate
  // take a face off the front of the queue
  while let Some(test_face) = face_queue.pop_front() {
    // Check to make sure it's still in the mesh (many faces will be removed)
    if !hull_mesh.faces.contains_key(& test_face.borrow().id) { continue; }
    // For all the points in the list, find the one that is both visible to and farthest from the face

    let face_visible_points: Vec<Pt> = points_list.iter()
        .filter(|pt| test_face.borrow().can_see(pt))
        .cloned()
        .collect();

    let (point_maxima, _) = face_visible_points.iter()
        .enumerate()
        .fold((None, 0.0), |(mut point_maxima, mut max_dist), (idx, pt)| {
          let dist = test_face.borrow().directed_distance_to(pt);
          if dist > max_dist {
            point_maxima = Some((idx, pt.clone()));
            max_dist = dist;
          }
          (point_maxima, max_dist)
        });

    if point_maxima.is_none() { continue; }
    let (max_index, max_point) = point_maxima.unwrap();

    // Removes the maximum point from the list of possible points
    // This is essential, because it avoids a certain situation where
    // The algorithm generates new faces which can still see the original maximum point,
    // And these faces then are split, and generate new faces which can still see the
    // Maximum point, along with two invalid faces.
    // I still don't understand The exact conditions under which this occurs, but try
    // removing this line and generating a random convex hull about 40 or 50 times, and it'll
    // probably happen once.
    points_list.remove(max_index);

    // For all the faces in the mesh, check whether they are visible from the point
    // i.e. check whether the point is in the direction of the face normal,
    // and sufficiently far away that it should count
    // For each face where this is the case, add it to a list.
    let light_faces: Vec<FaceRc> = hull_mesh.faces.values().filter(|f| f.borrow().can_see(& max_point)).cloned().collect();

    // These faces should all be adjacent.
    // Find their outline on the mesh. This is the "horizon"
    // Then, replace all such faces with new faces which connect
    // To the farthest point.
    // Add the new faces to the end of the queue
    match hull_mesh.attach_point_for_faces(max_point, & light_faces) {
      Ok(new_faces) => {
        // Filter out from points_list any point which is
        // in face_visible_points and behind all of the new faces
        // This might be a performance improvement, and it might not. I'm not sure
        points_list.retain(|p| {
          // If the point isn't visible to the current test face, don't worry about it here
          face_visible_points.iter().all(|face_pt| * face_pt != * p) || new_faces.iter().any(|n_face| n_face.borrow().can_see(p))
        });

        face_queue.extend(new_faces);
      },
      Err(message) => { println!("Error occurred while attaching a new point, {}", message); },
    }
  }

  // Once all faces have been iterated over, the convex hull should be complete
  return hull_mesh;
}
