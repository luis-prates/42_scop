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
        let material = if let Some(material_id) = mesh.material_id {
            Some(obj_scene.materials.get(material_id).ok_or_else(|| {
                format!(
                    "OBJ mesh references unknown material id {} while loading {}",
                    material_id, model_path
                )
            })?)
        } else {
            None
        };

        let diffuse_path =
            resolve_diffuse_texture_path(&model_dir, fallback_texture_path, material, model_path)?;
        textures.push(SceneTextureRef {
            path: diffuse_path,
            kind: TextureKind::Diffuse,
        });

        if let Some(specular_path) = material
            .and_then(|mat| mat.specular_texture.as_deref())
            .filter(|path| !path.is_empty())
            .and_then(|path| resolve_optional_bmp_material_path(&model_dir, path))
        {
            textures.push(SceneTextureRef {
                path: specular_path,
                kind: TextureKind::Specular,
            });
        }

        if let Some(normal_path) = material
            .and_then(|mat| mat.normal_texture.as_deref())
            .filter(|path| !path.is_empty())
            .and_then(|path| resolve_optional_bmp_material_path(&model_dir, path))
        {
            textures.push(SceneTextureRef {
                path: normal_path,
                kind: TextureKind::Normal,
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

fn resolve_optional_bmp_material_path(base_dir: &Path, relative_path: &str) -> Option<String> {
    if is_bmp_path(relative_path) {
        resolve_material_path(base_dir, relative_path).ok()
    } else {
        None
    }
}

fn resolve_diffuse_texture_path(
    model_dir: &Path,
    fallback_texture_path: &str,
    material: Option<&obj::ObjMaterialData>,
    model_path: &str,
) -> Result<String, String> {
    if !fallback_texture_path.is_empty() {
        return Ok(fallback_texture_path.to_string());
    }

    if let Some(diffuse_texture) = material
        .and_then(|mat| mat.diffuse_texture.as_deref())
        .filter(|texture| !texture.is_empty())
    {
        if !is_bmp_path(diffuse_texture) {
            return Err(format!(
                "Material diffuse texture for '{}' must be a .bmp file when no CLI fallback texture is provided: {}",
                model_path, diffuse_texture
            ));
        }
        return resolve_material_path(model_dir, diffuse_texture);
    }

    Err(format!(
        "No diffuse texture available for '{}' (expected CLI fallback BMP or material map_Kd BMP)",
        model_path
    ))
}

fn is_bmp_path(path: &str) -> bool {
    Path::new(path)
        .extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| ext.eq_ignore_ascii_case("bmp"))
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use std::env;
    use std::fs;
    use std::path::PathBuf;
    use std::process;
    use std::time::{SystemTime, UNIX_EPOCH};

    use super::build_scene_model;

    fn unique_temp_dir(prefix: &str) -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock should be after epoch")
            .as_nanos();
        let dir = env::temp_dir().join(format!("{}_{}_{}", prefix, process::id(), nanos));
        fs::create_dir_all(&dir).expect("failed to create temporary test directory");
        dir
    }

    #[test]
    fn cli_fallback_diffuse_texture_overrides_material_map_kd() {
        let dir = unique_temp_dir("scop_model_builder_fallback");
        let obj_path = dir.join("mesh.obj");
        fs::write(
            dir.join("mesh.mtl"),
            "\
newmtl Mat
map_Kd texture.png
",
        )
        .expect("failed to write MTL fixture");
        fs::write(
            &obj_path,
            "\
mtllib mesh.mtl
usemtl Mat
v 0 0 0
v 1 0 0
v 0 1 0
f 1 2 3
",
        )
        .expect("failed to write OBJ fixture");

        let fallback_path = "resources/textures/brickwall.bmp";
        let scene = build_scene_model(
            obj_path
                .to_str()
                .expect("temporary path should be valid UTF-8"),
            fallback_path,
        )
        .expect("scene should build with CLI fallback texture");

        assert_eq!(scene.meshes.len(), 1);
        assert_eq!(scene.meshes[0].textures[0].path, fallback_path);

        fs::remove_dir_all(&dir).expect("failed to cleanup temp directory");
    }
}
