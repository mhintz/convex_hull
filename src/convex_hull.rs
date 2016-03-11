use std::collections::LinkedList;

use defs::*;

use half_edge_mesh::HalfEdgeMesh;
use half_edge_mesh::FaceRc;

// TODO: this algorithm for choosing the initial tetrahedron points needs to be rethought.
// For one, it doesn't take into account the edge case where some of the max values are the same
// Point. For another, it doesn't handle certain edge cases that arise with degenerate positioning
// of the tetrahedron points. Rethink this algorithm for better robustness.
fn get_extreme_points(list: & Vec<Pt>) -> (usize, usize, usize, usize) {
  debug_assert!(list.len() > 0);

  let mut max_x = list[0].x; let mut max_x_idx = 0;
  let mut max_y = list[0].y; let mut max_y_idx = 0;
  let mut max_z = list[0].z; let mut max_z_idx = 0;
  let mut min_z = list[0].z; let mut min_z_idx = 0;

  for (i, pt) in list.iter().enumerate() {
    if max_x < pt.x {
      max_x = pt.x; max_x_idx = i;
    }
    if max_y < pt.y {
      max_y = pt.y; max_y_idx = i;
    }
    if max_z < pt.z {
      max_z = pt.z; max_z_idx = i;
    }
    if min_z > pt.z {
      min_z = pt.z; min_z_idx = i;
    }
  }

  (max_x_idx, max_y_idx, max_z_idx, min_z_idx)
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
  let (max_x_idx, max_y_idx, max_z_idx, min_z_idx) = get_extreme_points(& points_list);
  // This is the starting point of the mesh
  let mut hull_mesh = HalfEdgeMesh::from_tetrahedron_pts(points_list[max_y_idx], points_list[max_z_idx], points_list[max_x_idx], points_list[min_z_idx]);

  // Remove them from the list of points, since for performance it's important to minimise the size of this list
  let mut remove_indexes = vec![max_x_idx, max_y_idx, max_z_idx, min_z_idx];
  remove_indexes.sort_by(|a, b| a.cmp(b).reverse());
  for i in remove_indexes {
    // Remove in descending order, to not disturb the indexes of the list
    points_list.remove(i);
  }

  // Filter the points list based on whether the point is inside the tetrahedron.
  // This saves a lot of iteration steps
  points_list.retain(|p| !hull_mesh.contains(p));

  // Add all faces of the hull to a FIFO queue
  let mut face_queue: LinkedList<FaceRc> = hull_mesh.faces.values().cloned().collect();

  // While the queue has faces, iterate
  // take a face off the front of the queue
  while let Some(test_face) = face_queue.pop_front() {
    // Check to make sure it's still in the mesh (many faces will be removed)
    if !hull_mesh.faces.contains_key(& test_face.borrow().id) { continue; }
    // For all the points in the list, find the one that is both visible to and farthest from the face

    let (point_maxima, _) = points_list.iter()
        .filter(|pt| test_face.borrow().can_see(pt))
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
    let (max_pt_index, max_point) = point_maxima.unwrap();

    // Remove the point from the list of searchable points.
    // It will be added to the hull and will always be strictly within the hull from here on
    points_list.remove(max_pt_index);

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
    match hull_mesh.attach_point_for_faces(max_point.clone(), & light_faces) {
      Ok(new_faces) => face_queue.extend(new_faces),
      Err(message) => println!("Error occurred while building convex hull, {}", message),
    }
    // TODO: filter out from points_list any point which is behind all of the new faces
  }
  // Once all faces have been iterated over, the convex hull should be complete
  return hull_mesh;
}
