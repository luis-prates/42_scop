use std::os::raw::c_void;
use std::path::Path;

use cgmath::{InnerSpace, vec2, vec3};
use gl;
use image;
use image::DynamicImage::*;
use image::GenericImage;
use tobj;

use crate::shader;
use crate::mesh;
use cgmath::{ Vector3, Vector2 };

use mesh::{ Mesh, Texture, Vertex };
use shader::Shader;
use crate::common::load_texture;

#[derive(Default)]
pub struct Model {
    /*  Model Data */
    pub meshes: Vec<Mesh>,
    pub textures_loaded: Vec<Texture>,   // stores all the textures loaded so far, optimization to make sure textures aren't loaded more than once.
    directory: String,
	pub base_color_x: f32,
	pub base_color_y: f32,
	pub base_color_z: f32,
}

impl Model {
	/// constructor, expects a filepath to a 3D model.
    pub fn new(path: &str) -> Model {
        let mut model = Model::default();
        model.load_model(path);
        model
    }

	pub fn draw(&self, shader: &Shader) {
        for mesh in &self.meshes {
            unsafe { mesh.draw(shader); }
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

	pub fn change_color(&mut self, new_color: &Vector3<f32>) {
		for (i, vertice) in &mut self.meshes[0].vertices.iter_mut().enumerate() {
			vertice.color = vertice.new_color;
			vertice.new_color.x = new_color.x + (i / 3) as f32 * 0.05;
			vertice.new_color.y = new_color.y + (i / 3) as f32 * 0.05;
			vertice.new_color.z = new_color.z + (i / 3) as f32 * 0.05;
			if vertice.new_color.x > 1.0 {
				vertice.new_color.x = 1.0;
			}
			if vertice.new_color.y > 1.0 {
				vertice.new_color.y = 1.0;
			}
			if vertice.new_color.z > 1.0 {
				vertice.new_color.z = 1.0;
			}
			self.base_color_x = new_color.x;
			self.base_color_y = new_color.y;
			self.base_color_z = new_color.z;
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
	fn load_model(&mut self, path: &str) {
        let path = Path::new(path);

        // retrieve the directory path of the filepath
        self.directory = path.parent().unwrap_or_else(|| Path::new("")).to_str().unwrap().into();
        let obj = tobj::load_obj(path);

		self.base_color_x = 0.05;
		self.base_color_y = 0.05;
		self.base_color_z = 0.05;
        let (models, materials) = obj.unwrap();
        for model in models {
            let mesh = &model.mesh;
            let num_vertices = mesh.positions.len() / 3;

            // data to fill
            let mut vertices: Vec<Vertex> = Vec::with_capacity(num_vertices);
            let indices: Vec<u32> = mesh.indices.clone();

            let (p, n, t) = (&mesh.positions, &mesh.normals, &mesh.texcoords);
            for i in 0..num_vertices {
				let mut vertexx = Vertex::default();
				vertexx.position = vec3(p[i*3], p[i*3+1], p[i*3+2]);
				if &mesh.normals.len() == &mesh.positions.len() {
					vertexx.normal = vec3(n[i*3], n[i*3+1], n[i*3+2]);
				}
				if &mesh.texcoords.len() > &0 {
					vertexx.tex_coords = vec2(t[i*2], t[i*2+1]);
				} else {
					vertexx.tex_coords = generate_texture_coordinates(&vertexx.position);
					// println!("tex coords are: {:?}", vertexx.tex_coords);
					vertexx.color = vec3(self.base_color_x + (i / 3) as f32 * 0.05, self.base_color_y + (i / 3) as f32 * 0.05, self.base_color_z + (i / 3) as f32 * 0.05);
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
                if !material.diffuse_texture.is_empty() {
                    let texture = self.load_material_texture(&material.diffuse_texture, "texture_diffuse");
					println!("Material diffuse: {} and {}", texture.type_, texture.id);
                    textures.push(texture);
                } else {
					println!("No texture. Setting default");
					let texture = Texture {
						id: unsafe { load_texture("resources/textures/container2.png") },
						type_: "texture_diffuse".into(),
						path: "resources/textures/container2.png".into()
					};
					self.textures_loaded.push(texture.clone());
					textures.push(texture);
				}
                // 2. specular map
                if !material.specular_texture.is_empty() {
                    let texture = self.load_material_texture(&material.specular_texture, "texture_specular");
					println!("Material specular: {} and {}", texture.type_, texture.id);

                    textures.push(texture);
                }
                // 3. normal map
                if !material.normal_texture.is_empty() {
                    let texture = self.load_material_texture(&material.normal_texture, "texture_normal");
					println!("Material normal: {} and {}", texture.type_, texture.id);
                    
					textures.push(texture);
                }
                // NOTE: no height maps
            }

            self.meshes.push(Mesh::new(vertices, indices, textures));
        }
	}

	fn load_material_texture(&mut self, path: &str, type_name: &str) -> Texture {
		{
			let texture = self.textures_loaded.iter().find(|t| t.path == path);
			if let Some(texture) = texture {
				return texture.clone();
			}
		}

		let texture = Texture {
			id: unsafe { texture_from_file(path, &self.directory) },
			type_: type_name.into(),
			path: path.into()
		};
		self.textures_loaded.push(texture.clone());
		texture
	}
}
	
unsafe fn texture_from_file(path: &str, directory: &str) -> u32 {
	let filename = format!("{}/{}", directory, path);

	let mut texture_id = 0;
	gl::GenTextures(1, &mut texture_id);

	let img = image::open(&Path::new(&filename)).expect("Texture failed to load");
	let img = img.flipv();
	let format = match img {
		ImageLuma8(_) => gl::RED,
		ImageLumaA8(_) => gl::RG,
		ImageRgb8(_) => gl::RGB,
		ImageRgba8(_) => gl::RGBA,
	};

	let data = img.raw_pixels();

	gl::BindTexture(gl::TEXTURE_2D, texture_id);
	gl::TexImage2D(gl::TEXTURE_2D, 0, format as i32, img.width() as i32, img.height() as i32,
		0, format, gl::UNSIGNED_BYTE, &data[0] as *const u8 as *const c_void);
	gl::GenerateMipmap(gl::TEXTURE_2D);

	gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::REPEAT as i32);
	gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::REPEAT as i32);
	gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR_MIPMAP_LINEAR as i32);
	gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as i32);

	texture_id
}

fn generate_texture_coordinates(vertex: &Vector3<f32>) -> Vector2<f32> {
	let mut tex_coords = Vector2::new(0.0, 0.0);
	let u = (vertex.x + 1.0) / 2.0; // Map x to [0, 1]
	let v = (vertex.y + 1.0) / 2.0; // Map y to [0, 1]
	tex_coords.x = u;
	tex_coords.y = v;
	tex_coords.normalize()
}