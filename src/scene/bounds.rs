use super::model::SceneMesh;

pub fn center_all_axes(meshes: &[SceneMesh]) -> (f32, f32, f32) {
    let (min_x, max_x) = min_max_axis(meshes, |x, _, _| x);
    let (min_y, max_y) = min_max_axis(meshes, |_, y, _| y);
    let (min_z, max_z) = min_max_axis(meshes, |_, _, z| z);

    (
        center_from_range(min_x, max_x),
        center_from_range(min_y, max_y),
        center_from_range(min_z, max_z),
    )
}

fn min_max_axis<F>(meshes: &[SceneMesh], axis_fn: F) -> (f32, f32)
where
    F: Fn(f32, f32, f32) -> f32,
{
    meshes.iter().flat_map(|mesh| mesh.vertices.iter()).fold(
        (f32::INFINITY, f32::NEG_INFINITY),
        |(min, max), vertex| {
            let value = axis_fn(vertex.position.x, vertex.position.y, vertex.position.z);
            (min.min(value), max.max(value))
        },
    )
}

fn center_from_range(min: f32, max: f32) -> f32 {
    let center = (max - min) / 2.0;

    if f32::abs(max) == f32::abs(min) || max > 0.0 {
        max - center
    } else {
        min + center
    }
}
