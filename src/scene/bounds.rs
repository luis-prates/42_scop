use super::model::SceneMesh;

pub fn center_all_axes(meshes: &[SceneMesh]) -> (f32, f32, f32) {
    let (min_x, max_x) = min_max_axis(meshes, |x, _, _| x).unwrap_or((0.0, 0.0));
    let (min_y, max_y) = min_max_axis(meshes, |_, y, _| y).unwrap_or((0.0, 0.0));
    let (min_z, max_z) = min_max_axis(meshes, |_, _, z| z).unwrap_or((0.0, 0.0));

    (
        center_from_range(min_x, max_x),
        center_from_range(min_y, max_y),
        center_from_range(min_z, max_z),
    )
}

fn min_max_axis<F>(meshes: &[SceneMesh], axis_fn: F) -> Option<(f32, f32)>
where
    F: Fn(f32, f32, f32) -> f32,
{
    meshes
        .iter()
        .flat_map(|mesh| mesh.vertices.iter())
        .fold(None, |acc, vertex| {
            let value = axis_fn(vertex.position.x, vertex.position.y, vertex.position.z);
            match acc {
                Some((min, max)) => Some((min.min(value), max.max(value))),
                None => Some((value, value)),
            }
        })
}

fn center_from_range(min: f32, max: f32) -> f32 {
    (min + max) * 0.5
}

#[cfg(test)]
mod tests {
    use crate::math::{Vector2, Vector3};
    use crate::scene::model::{SceneMesh, SceneTextureRef, TextureKind, Vertex};

    use super::center_all_axes;

    fn build_vertex(position: Vector3) -> Vertex {
        Vertex {
            position,
            normal: Vector3::zero(),
            tex_coords: Vector2::zero(),
            tangent: Vector3::zero(),
            bitangent: Vector3::zero(),
            color: Vector3::zero(),
            new_color: Vector3::zero(),
        }
    }

    fn mesh_from_positions(positions: &[Vector3]) -> SceneMesh {
        let vertices = positions.iter().map(|p| build_vertex(*p)).collect();
        SceneMesh {
            vertices,
            indices: vec![0, 1, 2],
            textures: vec![SceneTextureRef {
                path: String::new(),
                kind: TextureKind::Diffuse,
            }],
            has_uv_mapping: false,
        }
    }

    #[test]
    fn center_all_axes_uses_aabb_midpoint() {
        let mesh = mesh_from_positions(&[
            Vector3::new(-5.0, -3.0, 2.0),
            Vector3::new(7.0, 1.0, 10.0),
            Vector3::new(0.0, 4.0, -6.0),
        ]);

        let (x, y, z) = center_all_axes(&[mesh]);
        assert_eq!(x, 1.0);
        assert_eq!(y, 0.5);
        assert_eq!(z, 2.0);
    }

    #[test]
    fn center_all_axes_returns_origin_for_empty_meshes() {
        let (x, y, z) = center_all_axes(&[]);
        assert_eq!(x, 0.0);
        assert_eq!(y, 0.0);
        assert_eq!(z, 0.0);
    }
}
