use std::path::Path;

use crate::math;
use crate::mesh;
use crate::my_bmp_loader;
use crate::obj_loader;
use crate::shader;
use obj_loader::LoadOptions;

use math::{Vector2, Vector3};
use mesh::{Mesh, Texture, Vertex};
use my_bmp_loader::load_texture_bmp;
use shader::Shader;

#[derive(Default)]
pub struct Model {
    /*  Model Data */
    pub meshes: Vec<Mesh>,
    pub textures_loaded: Vec<Texture>, // stores all the textures loaded so far, optimization to make sure textures aren't loaded more than once.
    directory: String,
    pub base_color_x: f32,
    pub base_color_y: f32,
    pub base_color_z: f32,
}

impl Model {
    /// constructor, expects a filepath to a 3D model.
    pub fn new(model_path: &str, texture_path: &str) -> Result<Model, String> {
        let mut model = Model::default();
        model.load_model(model_path, texture_path)?;
        Ok(model)
    }

    pub fn draw(&self, shader: &Shader) {
        for mesh in &self.meshes {
            unsafe {
                mesh.draw(shader);
            }
        }
    }

    pub fn get_center_all_axes(&self) -> (f32, f32, f32) {
        let (min_x, max_x) = self.get_min_max_axis(|vertice| vertice.position.x);
        let (min_y, max_y) = self.get_min_max_axis(|vertice| vertice.position.y);
        let (min_z, max_z) = self.get_min_max_axis(|vertice| vertice.position.z);

        let center_x = self.calculate_center(min_x, max_x);
        let center_y = self.calculate_center(min_y, max_y);
        let center_z = self.calculate_center(min_z, max_z);

        (center_x, center_y, center_z)
    }

    pub fn change_color(&mut self, new_color: &Vector3) {
        // Update base color
        self.base_color_x = new_color.x;
        self.base_color_y = new_color.y;
        self.base_color_z = new_color.z;

        // Apply face-based brightness variation to create distinguishable shades.
        for mesh in &mut self.meshes {
            for (i, vertice) in mesh.vertices.iter_mut().enumerate() {
                // Same face-based variation as initial loading.
                let face_index = i / 3;
                let brightness = ((face_index % 11) as f32 / 11.0) * 0.6 + 0.4; // Range: 0.4 to 1.0

                vertice.new_color.x = (new_color.x * brightness).min(1.0);
                vertice.new_color.y = (new_color.y * brightness).min(1.0);
                vertice.new_color.z = (new_color.z * brightness).min(1.0);
            }
            unsafe {
                mesh.setup_mesh();
            }
        }
    }

    fn get_min_max_axis<F>(&self, axis_fn: F) -> (f32, f32)
    where
        F: Fn(&Vertex) -> f32,
    {
        self.meshes
            .iter()
            .flat_map(|mesh| &mesh.vertices)
            .fold((f32::INFINITY, f32::NEG_INFINITY), |(min, max), vertice| {
                (min.min(axis_fn(vertice)), max.max(axis_fn(vertice)))
            })
    }

    fn calculate_center(&self, min: f32, max: f32) -> f32 {
        let center = (max - min) / 2.0;

        if f32::abs(max) == f32::abs(min) || max > 0.0 {
            max - center
        } else {
            min + center
        }
    }

    // loads a model from file and stores the resulting meshes in the meshes vector.
    fn load_model(&mut self, model_path: &str, texture_path: &str) -> Result<(), String> {
        let path = Path::new(model_path);
        let model_dir = path.parent().unwrap_or_else(|| Path::new("")).to_path_buf();

        // retrieve the directory path of the filepath
        self.directory = model_dir
            .to_str()
            .ok_or_else(|| format!("Invalid UTF-8 in model path: {}", model_path))?
            .into();
        let (models, materials) = obj_loader::load_obj(
            path,
            &LoadOptions {
                triangulate: true,
                single_index: false,
            },
        )?;

        // Default to medium grey so face-based color variations are visible.
        self.base_color_x = 0.6;
        self.base_color_y = 0.6;
        self.base_color_z = 0.6;

        let materials = materials.unwrap_or_default();
        for model in models {
            let mesh = &model.mesh;

            if mesh.positions.len() % 3 != 0 {
                return Err(
                    "Malformed OBJ mesh: positions array length is not a multiple of 3".to_string(),
                );
            }

            let num_vertices = mesh.positions.len() / 3;
            if !mesh.normals.is_empty() && mesh.normals.len() != mesh.positions.len() {
                return Err(
                    "Malformed OBJ mesh: normals array length must match positions length"
                        .to_string(),
                );
            }
            if !mesh.texcoords.is_empty() && mesh.texcoords.len() != num_vertices * 2 {
                return Err(
                    "Malformed OBJ mesh: texcoords array length must be vertex_count * 2"
                        .to_string(),
                );
            }

            // data to fill
            let mut vertices: Vec<Vertex> = Vec::with_capacity(num_vertices);
            let indices: Vec<u32> = mesh.indices.clone();

            let (p, n, t) = (&mesh.positions, &mesh.normals, &mesh.texcoords);

            // Calculate bounding box for planar UV mapping when texcoords are missing.
            let (min_x, max_x, min_y, max_y) = if mesh.texcoords.is_empty() {
                let mut min_x = f32::INFINITY;
                let mut max_x = f32::NEG_INFINITY;
                let mut min_y = f32::INFINITY;
                let mut max_y = f32::NEG_INFINITY;

                for i in 0..num_vertices {
                    let x = p[i * 3];
                    let y = p[i * 3 + 1];
                    min_x = min_x.min(x);
                    max_x = max_x.max(x);
                    min_y = min_y.min(y);
                    max_y = max_y.max(y);
                }
                (min_x, max_x, min_y, max_y)
            } else {
                (0.0, 1.0, 0.0, 1.0) // Not used when texcoords exist.
            };

            for i in 0..num_vertices {
                let mut vertex = Vertex {
                    position: Vector3::new(p[i * 3], p[i * 3 + 1], p[i * 3 + 2]),
                    ..Default::default()
                };

                if !mesh.normals.is_empty() {
                    vertex.normal = Vector3::new(n[i * 3], n[i * 3 + 1], n[i * 3 + 2]);
                }

                if !mesh.texcoords.is_empty() {
                    vertex.tex_coords = Vector2::new(t[i * 2], t[i * 2 + 1]);
                } else {
                    // Planar UV mapping: project XY plane to [0,1] texture space.
                    let u = if max_x != min_x {
                        (vertex.position.x - min_x) / (max_x - min_x)
                    } else {
                        0.5
                    };
                    let v = if max_y != min_y {
                        (vertex.position.y - min_y) / (max_y - min_y)
                    } else {
                        0.5
                    };
                    vertex.tex_coords = Vector2::new(u, v);
                }

                // Face-based color variation: each triangle gets a distinct shade.
                let face_index = i / 3;
                let brightness = ((face_index % 11) as f32 / 11.0) * 0.6 + 0.4; // Range: 0.4 to 1.0

                vertex.color = Vector3::new(
                    (self.base_color_x * brightness).min(1.0),
                    (self.base_color_y * brightness).min(1.0),
                    (self.base_color_z * brightness).min(1.0),
                );
                vertex.new_color = vertex.color;

                vertices.push(vertex);
            }

            // process material
            let mut textures = Vec::new();
            if let Some(material_id) = mesh.material_id {
                let material = materials.get(material_id).ok_or_else(|| {
                    format!(
                        "OBJ mesh references unknown material id {} while loading {}",
                        material_id, model_path
                    )
                })?;

                // 1. diffuse map
                match &material.diffuse_texture {
                    Some(diffuse_texture) if !diffuse_texture.is_empty() => {
                        let diffuse_path = resolve_material_path(&model_dir, diffuse_texture)?;
                        let texture =
                            self.load_material_texture(diffuse_path.as_str(), "texture_diffuse")?;
                        textures.push(texture);
                    }
                    _ => {
                        let texture =
                            self.load_material_texture(texture_path, "texture_diffuse")?;
                        textures.push(texture);
                    }
                }

                // 2. specular map
                if let Some(specular_texture) = &material.specular_texture
                    && !specular_texture.is_empty()
                {
                    let specular_path = resolve_material_path(&model_dir, specular_texture)?;
                    let texture =
                        self.load_material_texture(specular_path.as_str(), "texture_specular")?;
                    textures.push(texture);
                }

                // 3. normal map
                if let Some(normal_texture) = &material.normal_texture
                    && !normal_texture.is_empty()
                {
                    let normal_path = resolve_material_path(&model_dir, normal_texture)?;
                    let texture =
                        self.load_material_texture(normal_path.as_str(), "texture_normal")?;
                    textures.push(texture);
                }
            } else {
                // No MTL file - use the provided texture.
                let texture = self.load_material_texture(texture_path, "texture_diffuse")?;
                textures.push(texture);
            }

            self.meshes.push(Mesh::new(vertices, indices, textures));
        }

        Ok(())
    }

    fn load_material_texture(
        &mut self,
        texture_path: &str,
        type_name: &str,
    ) -> Result<Texture, String> {
        if let Some(texture) = self.textures_loaded.iter().find(|t| t.path == texture_path) {
            return Ok(texture.clone());
        }

        let texture = Texture {
            id: load_texture_bmp(texture_path)?,
            type_: type_name.into(),
            path: texture_path.into(),
        };
        self.textures_loaded.push(texture.clone());
        Ok(texture)
    }
}

fn resolve_material_path(base_dir: &Path, relative_path: &str) -> Result<String, String> {
    let path = base_dir.join(relative_path);
    path.to_str()
        .map(|s| s.to_string())
        .ok_or_else(|| format!("Invalid UTF-8 in material texture path: {}", path.display()))
}
