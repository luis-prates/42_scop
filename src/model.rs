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

        // Apply face-based brightness variation to create distinguishable shades
        for (i, vertice) in &mut self.meshes[0].vertices.iter_mut().enumerate() {
            vertice.color = vertice.new_color;

            // Same face-based variation as initial loading
            let face_index = i / 3;
            let brightness = ((face_index % 11) as f32 / 11.0) * 0.6 + 0.4; // Range: 0.4 to 1.0

            vertice.new_color.x = (new_color.x * brightness).min(1.0);
            vertice.new_color.y = (new_color.y * brightness).min(1.0);
            vertice.new_color.z = (new_color.z * brightness).min(1.0);
        }
        unsafe { self.meshes[0].setup_mesh() };
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

        // retrieve the directory path of the filepath
        self.directory = path
            .parent()
            .unwrap_or_else(|| Path::new(""))
            .to_str()
            .ok_or_else(|| format!("Invalid UTF-8 in model path: {}", model_path))?
            .into();
        let obj = obj_loader::load_obj(
            path,
            &LoadOptions {
                triangulate: true,
                single_index: false,
            },
        )?;

        // Default to medium grey so face-based color variations are visible
        self.base_color_x = 0.6;
        self.base_color_y = 0.6;
        self.base_color_z = 0.6;
        let (models, materials) = obj;
        let materials = materials.unwrap_or_default();
        for model in models {
            let mesh = &model.mesh;

            let num_vertices = mesh.positions.len() / 3;

            // data to fill
            let mut vertices: Vec<Vertex> = Vec::with_capacity(num_vertices);
            let indices: Vec<u32> = mesh.indices.clone();

            let (p, n, t) = (&mesh.positions, &mesh.normals, &mesh.texcoords);

            // Calculate bounding box for planar UV mapping when texcoords are missing
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
                (0.0, 1.0, 0.0, 1.0) // Not used when texcoords exist
            };

            for i in 0..num_vertices {
                let mut vertexx = Vertex {
                    position: Vector3::new(p[i * 3], p[i * 3 + 1], p[i * 3 + 2]),
                    ..Default::default()
                };
                if mesh.normals.len() == mesh.positions.len() {
                    vertexx.normal = Vector3::new(n[i * 3], n[i * 3 + 1], n[i * 3 + 2]);
                }
                if !mesh.texcoords.is_empty() {
                    vertexx.tex_coords = Vector2::new(t[i * 2], t[i * 2 + 1]);
                } else {
                    // Planar UV mapping: project XY plane to [0,1] texture space
                    let u = if max_x != min_x {
                        (vertexx.position.x - min_x) / (max_x - min_x)
                    } else {
                        0.5
                    };
                    let v = if max_y != min_y {
                        (vertexx.position.y - min_y) / (max_y - min_y)
                    } else {
                        0.5
                    };
                    vertexx.tex_coords = Vector2::new(u, v);

                    // Face-based color variation: each triangle gets a distinct shade
                    // Using modulo to create variation, each face gets a brightness from 0.4 to 1.0
                    let face_index = i / 3;
                    let brightness = ((face_index % 11) as f32 / 11.0) * 0.6 + 0.4; // Range: 0.4 to 1.0

                    // Apply brightness multiplier to base color, clamped to [0, 1]
                    vertexx.color = Vector3::new(
                        (self.base_color_x * brightness).min(1.0),
                        (self.base_color_y * brightness).min(1.0),
                        (self.base_color_z * brightness).min(1.0),
                    );
                    vertexx.new_color = vertexx.color;
                }
                vertices.push(vertexx);
            }

            // process material
            let mut textures = Vec::new();
            if let Some(material_id) = mesh.material_id {
                println!("Material id is: {}", material_id);
                let material = &materials[material_id];

                // 1. diffuse map
                match &material.diffuse_texture {
                    Some(diffuse_texture) if !diffuse_texture.is_empty() => {
                        println!("Diffuse texture path: {}", diffuse_texture);
                        let texture = self.load_material_texture(
                            Path::new(model_path)
                                .parent()
                                .unwrap()
                                .join(diffuse_texture)
                                .to_str()
                                .unwrap(),
                            "texture_diffuse",
                        );
                        println!("Material diffuse: {} and {}", texture.type_, texture.id);
                        textures.push(texture);
                    }
                    _ => {
                        println!("No texture in MTL. Setting default");
                        let texture = Texture {
                            id: unsafe { load_texture_bmp(texture_path) },
                            type_: "texture_diffuse".into(),
                            path: texture_path.into(),
                        };
                        self.textures_loaded.push(texture.clone());
                        textures.push(texture);
                    }
                }
                // 2. specular map
                if let Some(specular_texture) = &material.specular_texture {
                    println!("Specular texture path: {}", specular_texture);
                    if !specular_texture.is_empty() {
                        let texture = self.load_material_texture(
                            Path::new(model_path)
                                .parent()
                                .unwrap()
                                .join(specular_texture)
                                .to_str()
                                .unwrap(),
                            "texture_specular",
                        );
                        println!("Material specular: {} and {}", texture.type_, texture.id);

                        textures.push(texture);
                    }
                }

                // 3. normal map
                if let Some(normal_texture) = &material.normal_texture {
                    println!("Normal texture path: {}", normal_texture);
                    if !normal_texture.is_empty() {
                        let texture = self.load_material_texture(
                            Path::new(model_path)
                                .parent()
                                .unwrap()
                                .join(normal_texture)
                                .to_str()
                                .unwrap(),
                            "texture_normal",
                        );
                        println!("Material normal: {} and {}", texture.type_, texture.id);

                        textures.push(texture);
                    }
                }
            } else {
                // No MTL file - use the provided texture
                let texture = Texture {
                    id: unsafe { load_texture_bmp(texture_path) },
                    type_: "texture_diffuse".into(),
                    path: texture_path.into(),
                };
                self.textures_loaded.push(texture.clone());
                textures.push(texture);
            }

            self.meshes.push(Mesh::new(vertices, indices, textures));
        }
        Ok(())
    }

    fn load_material_texture(&mut self, texture_path: &str, type_name: &str) -> Texture {
        {
            let texture = self.textures_loaded.iter().find(|t| t.path == texture_path);
            if let Some(texture) = texture {
                return texture.clone();
            }
        }

        let texture = Texture {
            id: unsafe { load_texture_bmp(texture_path) },
            type_: type_name.into(),
            path: texture_path.into(),
        };
        self.textures_loaded.push(texture.clone());
        texture
    }
}
