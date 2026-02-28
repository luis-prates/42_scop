use crate::math::{Vector2, Vector3};

use super::bounds;
use super::coloring;

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct Vertex {
    pub position: Vector3,
    pub normal: Vector3,
    pub tex_coords: Vector2,
    pub tangent: Vector3,
    pub bitangent: Vector3,
    pub color: Vector3,
    pub new_color: Vector3,
}

impl Default for Vertex {
    fn default() -> Self {
        Self {
            position: Vector3::zero(),
            normal: Vector3::zero(),
            tex_coords: Vector2::zero(),
            tangent: Vector3::zero(),
            bitangent: Vector3::zero(),
            color: Vector3::zero(),
            new_color: Vector3::zero(),
        }
    }
}

#[derive(Clone, Debug)]
pub enum TextureKind {
    Diffuse,
    Specular,
    Normal,
}

impl TextureKind {
    pub fn shader_uniform_prefix(&self) -> &'static str {
        match self {
            TextureKind::Diffuse => "texture_diffuse",
            TextureKind::Specular => "texture_specular",
            TextureKind::Normal => "texture_normal",
        }
    }
}

#[derive(Clone, Debug)]
pub struct SceneTextureRef {
    pub path: String,
    pub kind: TextureKind,
}

#[derive(Clone, Debug)]
pub struct SceneMesh {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>,
    pub textures: Vec<SceneTextureRef>,
    pub has_uv_mapping: bool,
}

#[derive(Debug)]
pub struct SceneModel {
    pub meshes: Vec<SceneMesh>,
    pub base_color: Vector3,
    center: Vector3,
}

impl SceneModel {
    pub fn new(mut meshes: Vec<SceneMesh>, base_color: Vector3) -> Self {
        for mesh in &mut meshes {
            coloring::apply_face_shading(&mut mesh.vertices, &mesh.indices, &base_color);
        }

        let (center_x, center_y, center_z) = bounds::center_all_axes(&meshes);

        Self {
            meshes,
            base_color,
            center: Vector3::new(center_x, center_y, center_z),
        }
    }

    pub fn get_center_all_axes(&self) -> (f32, f32, f32) {
        (self.center.x, self.center.y, self.center.z)
    }

    pub fn change_color(&mut self, new_color: &Vector3) {
        self.base_color = *new_color;
        for mesh in &mut self.meshes {
            coloring::apply_new_color(&mut mesh.vertices, &mesh.indices, new_color);
        }
    }
}

impl Default for SceneModel {
    fn default() -> Self {
        Self {
            meshes: Vec::new(),
            base_color: Vector3::zero(),
            center: Vector3::zero(),
        }
    }
}
