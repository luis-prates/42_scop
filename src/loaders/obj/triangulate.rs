use super::index::FaceVertex;

const EPSILON: f64 = 1e-9;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TriangulationOutcome {
    Robust(Vec<[usize; 3]>),
    FallbackFan,
}

#[derive(Clone, Copy)]
struct Vec2 {
    x: f64,
    y: f64,
}

pub fn triangulate_face(face: &[FaceVertex], positions: &[[f32; 3]]) -> TriangulationOutcome {
    if face.len() < 3 {
        return TriangulationOutcome::FallbackFan;
    }

    let mut cleaned_original_indices: Vec<usize> = Vec::with_capacity(face.len());
    for (face_index, vertex) in face.iter().enumerate() {
        let position = as_vec3_f64(positions[vertex.0]);
        let is_duplicate_of_previous = cleaned_original_indices.last().is_some_and(|&prev_index| {
            let prev_position = as_vec3_f64(positions[face[prev_index].0]);
            nearly_equal3(position, prev_position)
        });

        if !is_duplicate_of_previous {
            cleaned_original_indices.push(face_index);
        }
    }

    if cleaned_original_indices.len() >= 2 {
        let first_position = as_vec3_f64(positions[face[cleaned_original_indices[0]].0]);
        let last_position = as_vec3_f64(
            positions[face[*cleaned_original_indices.last().expect("length checked")].0],
        );
        if nearly_equal3(first_position, last_position) {
            cleaned_original_indices.pop();
        }
    }

    if cleaned_original_indices.len() < 3 {
        return TriangulationOutcome::FallbackFan;
    }

    let cleaned_positions: Vec<[f64; 3]> = cleaned_original_indices
        .iter()
        .map(|&face_index| as_vec3_f64(positions[face[face_index].0]))
        .collect();

    let normal = newell_normal(&cleaned_positions);
    let normal_len2 = normal[0] * normal[0] + normal[1] * normal[1] + normal[2] * normal[2];
    if normal_len2 <= EPSILON {
        return TriangulationOutcome::FallbackFan;
    }

    let projected = project_to_2d(&cleaned_positions, normal);
    let signed_area = polygon_signed_area(&projected);
    if signed_area.abs() <= EPSILON {
        return TriangulationOutcome::FallbackFan;
    }

    if has_self_intersections(&projected) {
        return TriangulationOutcome::FallbackFan;
    }

    let local_triangles = match ear_clip(&projected, signed_area > 0.0) {
        Some(triangles) => triangles,
        None => return TriangulationOutcome::FallbackFan,
    };

    let mapped_triangles = local_triangles
        .into_iter()
        .map(|[a, b, c]| {
            [
                cleaned_original_indices[a],
                cleaned_original_indices[b],
                cleaned_original_indices[c],
            ]
        })
        .collect();

    TriangulationOutcome::Robust(mapped_triangles)
}

fn as_vec3_f64(v: [f32; 3]) -> [f64; 3] {
    [v[0] as f64, v[1] as f64, v[2] as f64]
}

fn nearly_equal3(a: [f64; 3], b: [f64; 3]) -> bool {
    (a[0] - b[0]).abs() <= EPSILON
        && (a[1] - b[1]).abs() <= EPSILON
        && (a[2] - b[2]).abs() <= EPSILON
}

fn newell_normal(points: &[[f64; 3]]) -> [f64; 3] {
    let mut normal = [0.0, 0.0, 0.0];
    let len = points.len();

    for i in 0..len {
        let current = points[i];
        let next = points[(i + 1) % len];

        normal[0] += (current[1] - next[1]) * (current[2] + next[2]);
        normal[1] += (current[2] - next[2]) * (current[0] + next[0]);
        normal[2] += (current[0] - next[0]) * (current[1] + next[1]);
    }

    normal
}

fn project_to_2d(points: &[[f64; 3]], normal: [f64; 3]) -> Vec<Vec2> {
    let ax = normal[0].abs();
    let ay = normal[1].abs();
    let az = normal[2].abs();

    if ax >= ay && ax >= az {
        points.iter().map(|p| Vec2 { x: p[1], y: p[2] }).collect()
    } else if ay >= az {
        points.iter().map(|p| Vec2 { x: p[0], y: p[2] }).collect()
    } else {
        points.iter().map(|p| Vec2 { x: p[0], y: p[1] }).collect()
    }
}

fn polygon_signed_area(points: &[Vec2]) -> f64 {
    let len = points.len();
    let mut sum = 0.0;

    for i in 0..len {
        let a = points[i];
        let b = points[(i + 1) % len];
        sum += a.x * b.y - b.x * a.y;
    }

    0.5 * sum
}

fn has_self_intersections(points: &[Vec2]) -> bool {
    let len = points.len();
    for i in 0..len {
        let i_next = (i + 1) % len;
        let a = points[i];
        let b = points[i_next];

        for j in (i + 1)..len {
            let j_next = (j + 1) % len;
            let c = points[j];
            let d = points[j_next];

            if i == j || i_next == j || j_next == i {
                continue;
            }

            if i == 0 && j_next == 0 {
                continue;
            }

            if segments_intersect(a, b, c, d) {
                return true;
            }
        }
    }

    false
}

fn segments_intersect(a: Vec2, b: Vec2, c: Vec2, d: Vec2) -> bool {
    let o1 = orient(a, b, c);
    let o2 = orient(a, b, d);
    let o3 = orient(c, d, a);
    let o4 = orient(c, d, b);

    let proper_intersection = ((o1 > EPSILON && o2 < -EPSILON) || (o1 < -EPSILON && o2 > EPSILON))
        && ((o3 > EPSILON && o4 < -EPSILON) || (o3 < -EPSILON && o4 > EPSILON));

    if proper_intersection {
        return true;
    }

    (o1.abs() <= EPSILON && on_segment(a, b, c))
        || (o2.abs() <= EPSILON && on_segment(a, b, d))
        || (o3.abs() <= EPSILON && on_segment(c, d, a))
        || (o4.abs() <= EPSILON && on_segment(c, d, b))
}

fn on_segment(a: Vec2, b: Vec2, p: Vec2) -> bool {
    p.x >= a.x.min(b.x) - EPSILON
        && p.x <= a.x.max(b.x) + EPSILON
        && p.y >= a.y.min(b.y) - EPSILON
        && p.y <= a.y.max(b.y) + EPSILON
}

fn orient(a: Vec2, b: Vec2, c: Vec2) -> f64 {
    (b.x - a.x) * (c.y - a.y) - (b.y - a.y) * (c.x - a.x)
}

fn ear_clip(points: &[Vec2], ccw: bool) -> Option<Vec<[usize; 3]>> {
    let mut remaining: Vec<usize> = (0..points.len()).collect();
    let mut triangles = Vec::with_capacity(points.len().saturating_sub(2));

    while remaining.len() > 3 {
        let mut ear_found = false;
        let len = remaining.len();

        for i in 0..len {
            let prev = remaining[(i + len - 1) % len];
            let curr = remaining[i];
            let next = remaining[(i + 1) % len];

            if !is_convex(points[prev], points[curr], points[next], ccw) {
                continue;
            }

            let tri_area = orient(points[prev], points[curr], points[next]).abs();
            if tri_area <= EPSILON {
                continue;
            }

            let mut contains_other_point = false;
            for &candidate in &remaining {
                if candidate == prev || candidate == curr || candidate == next {
                    continue;
                }

                if point_in_triangle(points[candidate], points[prev], points[curr], points[next]) {
                    contains_other_point = true;
                    break;
                }
            }

            if contains_other_point {
                continue;
            }

            triangles.push([prev, curr, next]);
            remaining.remove(i);
            ear_found = true;
            break;
        }

        if !ear_found {
            return None;
        }
    }

    if remaining.len() == 3 {
        triangles.push([remaining[0], remaining[1], remaining[2]]);
        Some(triangles)
    } else {
        None
    }
}

fn is_convex(prev: Vec2, curr: Vec2, next: Vec2, ccw: bool) -> bool {
    let cross = orient(prev, curr, next);
    if ccw {
        cross > EPSILON
    } else {
        cross < -EPSILON
    }
}

fn point_in_triangle(p: Vec2, a: Vec2, b: Vec2, c: Vec2) -> bool {
    let d1 = orient(p, a, b);
    let d2 = orient(p, b, c);
    let d3 = orient(p, c, a);

    let has_neg = d1 < -EPSILON || d2 < -EPSILON || d3 < -EPSILON;
    let has_pos = d1 > EPSILON || d2 > EPSILON || d3 > EPSILON;

    !(has_neg && has_pos)
}

#[cfg(test)]
mod tests {
    use super::{FaceVertex, TriangulationOutcome, triangulate_face};

    fn face(indices: &[usize]) -> Vec<FaceVertex> {
        indices.iter().map(|&i| (i, None, None)).collect()
    }

    #[test]
    fn triangulates_concave_polygon_with_ear_clipping() {
        let positions = vec![
            [0.0, 0.0, 0.0],
            [2.0, 0.0, 0.0],
            [2.0, 1.0, 0.0],
            [1.0, 0.4, 0.0],
            [0.0, 1.0, 0.0],
        ];

        let result = triangulate_face(&face(&[0, 1, 2, 3, 4]), &positions);
        match result {
            TriangulationOutcome::Robust(triangles) => assert_eq!(triangles.len(), 3),
            TriangulationOutcome::FallbackFan => panic!("expected robust triangulation"),
        }
    }

    #[test]
    fn triangulates_noncoplanar_quad() {
        let positions = vec![
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [1.0, 1.0, 0.2],
            [0.0, 1.0, 0.0],
        ];

        let result = triangulate_face(&face(&[0, 1, 2, 3]), &positions);
        match result {
            TriangulationOutcome::Robust(triangles) => assert_eq!(triangles.len(), 2),
            TriangulationOutcome::FallbackFan => panic!("expected robust triangulation"),
        }
    }

    #[test]
    fn falls_back_for_self_intersecting_polygon() {
        let positions = vec![
            [0.0, 0.0, 0.0],
            [2.0, 2.0, 0.0],
            [0.0, 2.0, 0.0],
            [2.0, 0.0, 0.0],
        ];

        let result = triangulate_face(&face(&[0, 1, 2, 3]), &positions);
        assert_eq!(result, TriangulationOutcome::FallbackFan);
    }
}
