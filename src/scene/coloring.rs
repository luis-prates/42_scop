use crate::math::Vector3;

use super::model::Vertex;

pub fn face_brightness(face_index: usize) -> f32 {
    ((face_index % 11) as f32 / 11.0) * 0.6 + 0.4
}

pub fn apply_face_shading(vertices: &mut [Vertex], indices: &[u32], base_color: &Vector3) {
    for (face_index, triangle) in indices.chunks_exact(3).enumerate() {
        let brightness = face_brightness(face_index);
        let color = Vector3::new(
            (base_color.x * brightness).min(1.0),
            (base_color.y * brightness).min(1.0),
            (base_color.z * brightness).min(1.0),
        );

        for &index in triangle {
            let vertex = &mut vertices[index as usize];
            vertex.color = color;
            vertex.new_color = color;
        }
    }
}

pub fn apply_new_color(vertices: &mut [Vertex], indices: &[u32], color: &Vector3) {
    for (face_index, triangle) in indices.chunks_exact(3).enumerate() {
        let brightness = face_brightness(face_index);
        let shaded_color = Vector3::new(
            (color.x * brightness).min(1.0),
            (color.y * brightness).min(1.0),
            (color.z * brightness).min(1.0),
        );

        for &index in triangle {
            vertices[index as usize].new_color = shaded_color;
        }
    }
}
