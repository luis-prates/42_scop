use std::path::Path;

use crate::loaders::obj::{self, ObjLoadOptions};
use crate::math::{Vector2, Vector3};

use super::model::{SceneMesh, SceneModel, SceneTextureRef, TextureKind, Vertex};

pub fn build_scene_model(
    model_path: &str,
    fallback_texture_path: &str,
) -> Result<SceneModel, String> {
    let path = Path::new(model_path);
    let model_dir = path.parent().unwrap_or_else(|| Path::new("")).to_path_buf();

    let obj_scene = obj::load(
        path,
        &ObjLoadOptions {
            triangulate: true,
            single_index: false,
        },
    )?;

    let base_color = Vector3::new(0.6, 0.6, 0.6);
    let mut meshes = Vec::new();

    for object in obj_scene.objects {
        let mesh = &object.mesh;

        if mesh.positions.len() % 3 != 0 {
            return Err(
                "Malformed OBJ mesh: positions array length is not a multiple of 3".to_string(),
            );
        }

        let num_vertices = mesh.positions.len() / 3;
        if !mesh.normals.is_empty() && mesh.normals.len() != mesh.positions.len() {
            return Err(
                "Malformed OBJ mesh: normals array length must match positions length".to_string(),
            );
        }
        if !mesh.texcoords.is_empty() && mesh.texcoords.len() != num_vertices * 2 {
            return Err(
                "Malformed OBJ mesh: texcoords array length must be vertex_count * 2".to_string(),
            );
        }
        let has_uv_mapping = !mesh.texcoords.is_empty();

        let mut vertices: Vec<Vertex> = Vec::with_capacity(num_vertices);
        let indices: Vec<u32> = mesh.indices.clone();
        let (p, n, t) = (&mesh.positions, &mesh.normals, &mesh.texcoords);

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
            (0.0, 1.0, 0.0, 1.0)
        };

        for i in 0..num_vertices {
            let mut vertex = Vertex {
                position: Vector3::new(p[i * 3], p[i * 3 + 1], p[i * 3 + 2]),
                ..Default::default()
            };

            if !mesh.normals.is_empty() {
                vertex.normal = Vector3::new(n[i * 3], n[i * 3 + 1], n[i * 3 + 2]);
            }

            if has_uv_mapping {
                vertex.tex_coords = Vector2::new(t[i * 2], t[i * 2 + 1]);
            } else {
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

            vertices.push(vertex);
        }

        let mut textures = Vec::new();
        if let Some(material_id) = mesh.material_id {
            let material = obj_scene.materials.get(material_id).ok_or_else(|| {
                format!(
                    "OBJ mesh references unknown material id {} while loading {}",
                    material_id, model_path
                )
            })?;

            match &material.diffuse_texture {
                Some(diffuse_texture) if !diffuse_texture.is_empty() => {
                    let diffuse_path = resolve_material_path(&model_dir, diffuse_texture)?;
                    textures.push(SceneTextureRef {
                        path: diffuse_path,
                        kind: TextureKind::Diffuse,
                    });
                }
                _ => {
                    textures.push(SceneTextureRef {
                        path: fallback_texture_path.to_string(),
                        kind: TextureKind::Diffuse,
                    });
                }
            }

            if let Some(specular_texture) = &material.specular_texture
                && !specular_texture.is_empty()
            {
                let specular_path = resolve_material_path(&model_dir, specular_texture)?;
                textures.push(SceneTextureRef {
                    path: specular_path,
                    kind: TextureKind::Specular,
                });
            }

            if let Some(normal_texture) = &material.normal_texture
                && !normal_texture.is_empty()
            {
                let normal_path = resolve_material_path(&model_dir, normal_texture)?;
                textures.push(SceneTextureRef {
                    path: normal_path,
                    kind: TextureKind::Normal,
                });
            }
        } else {
            textures.push(SceneTextureRef {
                path: fallback_texture_path.to_string(),
                kind: TextureKind::Diffuse,
            });
        }

        meshes.push(SceneMesh {
            vertices,
            indices,
            textures,
            has_uv_mapping,
        });
    }

    Ok(SceneModel::new(meshes, base_color))
}

fn resolve_material_path(base_dir: &Path, relative_path: &str) -> Result<String, String> {
    let path = base_dir.join(relative_path);
    path.to_str()
        .map(|s| s.to_string())
        .ok_or_else(|| format!("Invalid UTF-8 in material texture path: {}", path.display()))
}
