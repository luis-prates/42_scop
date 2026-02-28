use crate::math::Vector3;

use super::model::Vertex;

pub fn face_brightness(face_index: usize) -> f32 {
    ((face_index % 11) as f32 / 11.0) * 0.6 + 0.4
}

pub fn apply_face_shading(vertices: &mut [Vertex], base_color: &Vector3) {
    for (i, vertex) in vertices.iter_mut().enumerate() {
        let brightness = face_brightness(i / 3);
        vertex.color = Vector3::new(
            (base_color.x * brightness).min(1.0),
            (base_color.y * brightness).min(1.0),
            (base_color.z * brightness).min(1.0),
        );
        vertex.new_color = vertex.color;
    }
}

pub fn apply_new_color(vertices: &mut [Vertex], color: &Vector3) {
    for (i, vertex) in vertices.iter_mut().enumerate() {
        let brightness = face_brightness(i / 3);
        vertex.new_color = Vector3::new(
            (color.x * brightness).min(1.0),
            (color.y * brightness).min(1.0),
            (color.z * brightness).min(1.0),
        );
    }
}
