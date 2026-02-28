use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

type FaceVertex = (usize, Option<usize>, Option<usize>);
type MaterialFaces = HashMap<Option<String>, Vec<Vec<FaceVertex>>>;

#[derive(Default, Clone)]
pub struct LoadOptions {
    pub triangulate: bool,
    pub single_index: bool,
}

#[derive(Default, Clone)]
pub struct Mesh {
    pub positions: Vec<f32>,
    pub normals: Vec<f32>,
    pub texcoords: Vec<f32>,
    pub indices: Vec<u32>,
    pub material_id: Option<usize>,
}

#[derive(Default, Clone)]
pub struct Model {
    pub mesh: Mesh,
}

#[derive(Default, Clone)]
pub struct Material {
    pub name: String,
    pub diffuse_texture: Option<String>,
    pub specular_texture: Option<String>,
    pub normal_texture: Option<String>,
}

pub fn load_obj(
    path: &Path,
    options: &LoadOptions,
) -> Result<(Vec<Model>, Option<Vec<Material>>), String> {
    let file = File::open(path)
        .map_err(|e| format!("Failed to open OBJ file '{}': {}", path.display(), e))?;
    let reader = BufReader::new(file);

    let mut positions: Vec<[f32; 3]> = Vec::new();
    let mut normals: Vec<[f32; 3]> = Vec::new();
    let mut texcoords: Vec<[f32; 2]> = Vec::new();

    // Face data: each face stores (position_idx, texcoord_idx, normal_idx) tuples.
    let mut current_material: Option<String> = None;
    let mut material_faces: MaterialFaces = HashMap::new();
    let mut mtl_file: Option<String> = None;

    // Currently ignored by the mesh output format.
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

                // Triangulate if needed.
                if options.triangulate && face.len() > 3 {
                    // Fan triangulation.
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

    // Load materials if MTL file is specified.
    let materials = if let Some(mtl_filename) = mtl_file {
        let mtl_path = path
            .parent()
            .unwrap_or_else(|| Path::new(""))
            .join(mtl_filename);
        Some(load_mtl(&mtl_path)?)
    } else {
        None
    };

    // Build material name to index mapping.
    let material_map: HashMap<String, usize> = if let Some(ref mats) = materials {
        mats.iter()
            .enumerate()
            .map(|(i, mat)| (mat.name.clone(), i))
            .collect()
    } else {
        HashMap::new()
    };

    // Convert faces to meshes grouped by material.
    let mut models = Vec::new();

    for (mat_name, mat_faces) in material_faces {
        let material_id = mat_name
            .as_ref()
            .and_then(|name| material_map.get(name).copied());

        let mut mesh = Mesh {
            material_id,
            ..Default::default()
        };

        // Track unique vertices.
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

                    // Add position.
                    let position = positions[pos_idx];
                    mesh.positions.extend_from_slice(&position);

                    // Always push a normal vector per vertex (default to zero when omitted).
                    let normal = norm_idx
                        .map(|normal_idx| normals[normal_idx])
                        .unwrap_or([0.0, 0.0, 0.0]);
                    mesh.normals.extend_from_slice(&normal);

                    // Track UVs, but only publish them if every vertex has valid coordinates.
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

        models.push(Model { mesh });
    }

    Ok((models, materials))
}

fn load_mtl(path: &Path) -> Result<Vec<Material>, String> {
    let file = File::open(path)
        .map_err(|e| format!("Failed to open MTL file '{}': {}", path.display(), e))?;
    let reader = BufReader::new(file);

    let mut materials = Vec::new();
    let mut current_material: Option<Material> = None;

    for (line_number, line_result) in reader.lines().enumerate() {
        let line_number = line_number + 1;
        let line =
            line_result.map_err(|e| format!("Failed to read MTL line {}: {}", line_number, e))?;
        let line = line.trim();

        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.is_empty() {
            continue;
        }

        match parts[0] {
            "newmtl" => {
                // Save previous material.
                if let Some(mat) = current_material.take() {
                    materials.push(mat);
                }

                let material_name = directive_value(line, "newmtl", line_number)?;
                current_material = Some(Material {
                    name: material_name.to_string(),
                    ..Default::default()
                });
            }
            "map_Kd" => {
                let diffuse_texture = directive_value(line, "map_Kd", line_number)?;
                if let Some(ref mut mat) = current_material {
                    mat.diffuse_texture = Some(diffuse_texture.to_string());
                }
            }
            "map_Ks" => {
                let specular_texture = directive_value(line, "map_Ks", line_number)?;
                if let Some(ref mut mat) = current_material {
                    mat.specular_texture = Some(specular_texture.to_string());
                }
            }
            "map_Bump" | "bump" => {
                let key = parts[0];
                let normal_texture = directive_value(line, key, line_number)?;
                if let Some(ref mut mat) = current_material {
                    mat.normal_texture = Some(normal_texture.to_string());
                }
            }
            _ => {}
        }
    }

    // Save last material.
    if let Some(mat) = current_material {
        materials.push(mat);
    }

    Ok(materials)
}

fn parse_f32_component(raw: &str, line_number: usize, label: &str) -> Result<f32, String> {
    raw.parse::<f32>().map_err(|error| {
        format!(
            "OBJ line {}: invalid {} '{}': {}",
            line_number, label, raw, error
        )
    })
}

fn parse_face_vertex(
    token: &str,
    line_number: usize,
    positions_len: usize,
    texcoords_len: usize,
    normals_len: usize,
) -> Result<FaceVertex, String> {
    let fields: Vec<&str> = token.split('/').collect();
    if fields.is_empty() || fields.len() > 3 {
        return Err(format!(
            "OBJ line {}: invalid face vertex token '{}'",
            line_number, token
        ));
    }

    if fields[0].is_empty() {
        return Err(format!(
            "OBJ line {}: missing vertex position index in face token '{}'",
            line_number, token
        ));
    }

    let position_index = parse_obj_index(fields[0], positions_len, line_number, "position")?;

    let texcoord_index = if fields.len() > 1 && !fields[1].is_empty() {
        Some(parse_obj_index(
            fields[1],
            texcoords_len,
            line_number,
            "texcoord",
        )?)
    } else {
        None
    };

    let normal_index = if fields.len() > 2 && !fields[2].is_empty() {
        Some(parse_obj_index(
            fields[2],
            normals_len,
            line_number,
            "normal",
        )?)
    } else {
        None
    };

    Ok((position_index, texcoord_index, normal_index))
}

fn parse_obj_index(
    raw: &str,
    count: usize,
    line_number: usize,
    label: &str,
) -> Result<usize, String> {
    let parsed = raw.parse::<isize>().map_err(|error| {
        format!(
            "OBJ line {}: invalid {} index '{}': {}",
            line_number, label, raw, error
        )
    })?;

    if parsed == 0 {
        return Err(format!(
            "OBJ line {}: {} index 0 is invalid in OBJ format",
            line_number, label
        ));
    }

    if count == 0 {
        return Err(format!(
            "OBJ line {}: {} index '{}' referenced before any {} data was defined",
            line_number, label, raw, label
        ));
    }

    let resolved = if parsed > 0 {
        parsed - 1
    } else {
        count as isize + parsed
    };

    if resolved < 0 || resolved as usize >= count {
        return Err(format!(
            "OBJ line {}: {} index '{}' is out of bounds (count={})",
            line_number, label, raw, count
        ));
    }

    Ok(resolved as usize)
}

fn directive_value<'a>(
    line: &'a str,
    directive: &str,
    line_number: usize,
) -> Result<&'a str, String> {
    let value = line
        .strip_prefix(directive)
        .map(str::trim)
        .filter(|s| !s.is_empty());

    value.ok_or_else(|| {
        format!(
            "Line {}: directive '{}' is missing a required value",
            line_number, directive
        )
    })
}
