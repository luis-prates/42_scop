use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

use super::index::{FaceVertex, directive_value, parse_f32_component, parse_face_vertex};
use super::parse_mtl::load_mtl;
use super::types::{ObjLoadOptions, ObjMeshData, ObjObjectData, ObjSceneData};

type MaterialFaces = HashMap<Option<String>, Vec<Vec<FaceVertex>>>;

pub fn load(path: &Path, options: &ObjLoadOptions) -> Result<ObjSceneData, String> {
    let file = File::open(path)
        .map_err(|e| format!("Failed to open OBJ file '{}': {}", path.display(), e))?;
    let reader = BufReader::new(file);

    let mut positions: Vec<[f32; 3]> = Vec::new();
    let mut normals: Vec<[f32; 3]> = Vec::new();
    let mut texcoords: Vec<[f32; 2]> = Vec::new();

    let mut current_material: Option<String> = None;
    let mut material_faces: MaterialFaces = HashMap::new();
    let mut mtl_file: Option<String> = None;

    let _ = options.single_index;

    for (line_number, line_result) in reader.lines().enumerate() {
        let line_number = line_number + 1;
        let line =
            line_result.map_err(|e| format!("Failed to read OBJ line {}: {}", line_number, e))?;
        let line = line.trim();

        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.is_empty() {
            continue;
        }

        match parts[0] {
            "v" => {
                if parts.len() < 4 {
                    return Err(format!(
                        "OBJ line {}: vertex position requires 3 components",
                        line_number
                    ));
                }
                let x = parse_f32_component(parts[1], line_number, "vertex x")?;
                let y = parse_f32_component(parts[2], line_number, "vertex y")?;
                let z = parse_f32_component(parts[3], line_number, "vertex z")?;
                positions.push([x, y, z]);
            }
            "vn" => {
                if parts.len() < 4 {
                    return Err(format!(
                        "OBJ line {}: vertex normal requires 3 components",
                        line_number
                    ));
                }
                let x = parse_f32_component(parts[1], line_number, "normal x")?;
                let y = parse_f32_component(parts[2], line_number, "normal y")?;
                let z = parse_f32_component(parts[3], line_number, "normal z")?;
                normals.push([x, y, z]);
            }
            "vt" => {
                if parts.len() < 3 {
                    return Err(format!(
                        "OBJ line {}: texture coordinate requires at least 2 components",
                        line_number
                    ));
                }
                let u = parse_f32_component(parts[1], line_number, "texcoord u")?;
                let v = parse_f32_component(parts[2], line_number, "texcoord v")?;
                texcoords.push([u, v]);
            }
            "f" => {
                if parts.len() < 4 {
                    return Err(format!(
                        "OBJ line {}: face requires at least 3 vertices",
                        line_number
                    ));
                }

                let mut face = Vec::with_capacity(parts.len() - 1);
                for item in parts.iter().skip(1) {
                    let parsed = parse_face_vertex(
                        item,
                        line_number,
                        positions.len(),
                        texcoords.len(),
                        normals.len(),
                    )?;
                    face.push(parsed);
                }

                if options.triangulate && face.len() > 3 {
                    for i in 1..face.len() - 1 {
                        let tri = vec![face[0], face[i], face[i + 1]];
                        material_faces
                            .entry(current_material.clone())
                            .or_default()
                            .push(tri);
                    }
                } else {
                    if !options.triangulate && face.len() != 3 {
                        return Err(format!(
                            "OBJ line {}: non-triangular face requires triangulate=true",
                            line_number
                        ));
                    }
                    material_faces
                        .entry(current_material.clone())
                        .or_default()
                        .push(face);
                }
            }
            "usemtl" => {
                let material_name = directive_value(line, "usemtl", line_number)?;
                current_material = Some(material_name.to_string());
            }
            "mtllib" => {
                let file_name = directive_value(line, "mtllib", line_number)?;
                mtl_file = Some(file_name.to_string());
            }
            _ => {}
        }
    }

    let materials = if let Some(mtl_filename) = mtl_file {
        let mtl_path = path
            .parent()
            .unwrap_or_else(|| Path::new(""))
            .join(mtl_filename);
        load_mtl(&mtl_path)?
    } else {
        Vec::new()
    };

    let material_map: HashMap<String, usize> = materials
        .iter()
        .enumerate()
        .map(|(i, mat)| (mat.name.clone(), i))
        .collect();

    let mut objects = Vec::new();

    for (mat_name, mat_faces) in material_faces {
        let material_id = mat_name
            .as_ref()
            .and_then(|name| material_map.get(name).copied());

        let mut mesh = ObjMeshData {
            material_id,
            ..Default::default()
        };

        let mut vertex_map: HashMap<FaceVertex, u32> = HashMap::new();
        let mut next_index = 0u32;
        let mut vertex_texcoords: Vec<Option<[f32; 2]>> = Vec::new();

        for face in mat_faces {
            if face.len() != 3 {
                return Err(
                    "Internal OBJ loader error: non-triangulated face reached mesh assembly"
                        .to_string(),
                );
            }

            for &(pos_idx, tex_idx, norm_idx) in &face {
                let vertex_key: FaceVertex = (pos_idx, tex_idx, norm_idx);

                let index = if let Some(&idx) = vertex_map.get(&vertex_key) {
                    idx
                } else {
                    let idx = next_index;
                    next_index += 1;

                    let position = positions[pos_idx];
                    mesh.positions.extend_from_slice(&position);

                    let normal = norm_idx
                        .map(|normal_idx| normals[normal_idx])
                        .unwrap_or([0.0, 0.0, 0.0]);
                    mesh.normals.extend_from_slice(&normal);

                    let texcoord = tex_idx.map(|texcoord_idx| texcoords[texcoord_idx]);
                    vertex_texcoords.push(texcoord);

                    vertex_map.insert(vertex_key, idx);
                    idx
                };

                mesh.indices.push(index);
            }
        }

        if vertex_texcoords.iter().all(|uv| uv.is_some()) {
            for uv in vertex_texcoords {
                let uv = uv.expect("checked above");
                mesh.texcoords.extend_from_slice(&uv);
            }
        }

        objects.push(ObjObjectData { mesh });
    }

    Ok(ObjSceneData { objects, materials })
}
