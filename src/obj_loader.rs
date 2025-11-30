use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

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
    let file = File::open(path).map_err(|e| format!("Failed to open OBJ file: {}", e))?;
    let reader = BufReader::new(file);

    let mut positions: Vec<[f32; 3]> = Vec::new();
    let mut normals: Vec<[f32; 3]> = Vec::new();
    let mut texcoords: Vec<[f32; 2]> = Vec::new();

    // Face data: each face stores (position_idx, texcoord_idx, normal_idx) tuples
    let mut faces: Vec<Vec<(usize, Option<usize>, Option<usize>)>> = Vec::new();
    let mut current_material: Option<String> = None;
    let mut material_faces: HashMap<Option<String>, Vec<Vec<(usize, Option<usize>, Option<usize>)>>> =
        HashMap::new();
    let mut mtl_file: Option<String> = None;

    for line in reader.lines() {
        let line = line.map_err(|e| format!("Failed to read line: {}", e))?;
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
                // Vertex position
                if parts.len() >= 4 {
                    let x = parts[1].parse::<f32>().unwrap_or(0.0);
                    let y = parts[2].parse::<f32>().unwrap_or(0.0);
                    let z = parts[3].parse::<f32>().unwrap_or(0.0);
                    positions.push([x, y, z]);
                }
            }
            "vn" => {
                // Vertex normal
                if parts.len() >= 4 {
                    let x = parts[1].parse::<f32>().unwrap_or(0.0);
                    let y = parts[2].parse::<f32>().unwrap_or(0.0);
                    let z = parts[3].parse::<f32>().unwrap_or(0.0);
                    normals.push([x, y, z]);
                }
            }
            "vt" => {
                // Texture coordinate
                if parts.len() >= 3 {
                    let u = parts[1].parse::<f32>().unwrap_or(0.0);
                    let v = parts[2].parse::<f32>().unwrap_or(0.0);
                    texcoords.push([u, v]);
                }
            }
            "f" => {
                // Face
                let mut face = Vec::new();
                for i in 1..parts.len() {
                    let indices: Vec<&str> = parts[i].split('/').collect();
                    let pos_idx = indices[0].parse::<usize>().unwrap_or(1) - 1;
                    let tex_idx = if indices.len() > 1 && !indices[1].is_empty() {
                        Some(indices[1].parse::<usize>().unwrap_or(1) - 1)
                    } else {
                        None
                    };
                    let norm_idx = if indices.len() > 2 && !indices[2].is_empty() {
                        Some(indices[2].parse::<usize>().unwrap_or(1) - 1)
                    } else {
                        None
                    };
                    face.push((pos_idx, tex_idx, norm_idx));
                }

                // Triangulate if needed
                if options.triangulate && face.len() > 3 {
                    // Fan triangulation
                    for i in 1..face.len() - 1 {
                        let tri = vec![face[0], face[i], face[i + 1]];
                        material_faces
                            .entry(current_material.clone())
                            .or_insert_with(Vec::new)
                            .push(tri);
                    }
                } else {
                    material_faces
                        .entry(current_material.clone())
                        .or_insert_with(Vec::new)
                        .push(face);
                }
            }
            "usemtl" => {
                // Use material
                if parts.len() > 1 {
                    current_material = Some(parts[1].to_string());
                }
            }
            "mtllib" => {
                // Material library
                if parts.len() > 1 {
                    mtl_file = Some(parts[1].to_string());
                }
            }
            _ => {}
        }
    }

    // Load materials if MTL file is specified
    let materials = if let Some(mtl_filename) = mtl_file {
        let mtl_path = path.parent().unwrap_or_else(|| Path::new("")).join(mtl_filename);
        match load_mtl(&mtl_path) {
            Ok(mats) => Some(mats),
            Err(_) => None,
        }
    } else {
        None
    };

    // Build material name to index mapping
    let material_map: HashMap<String, usize> = if let Some(ref mats) = materials {
        mats.iter()
            .enumerate()
            .map(|(i, mat)| (mat.name.clone(), i))
            .collect()
    } else {
        HashMap::new()
    };

    // Convert faces to meshes grouped by material
    let mut models = Vec::new();

    for (mat_name, mat_faces) in material_faces {
        let material_id = mat_name.as_ref().and_then(|name| material_map.get(name).copied());

        let mut mesh = Mesh::default();
        mesh.material_id = material_id;

        // Track unique vertices
        let mut vertex_map: HashMap<(usize, Option<usize>, Option<usize>), u32> = HashMap::new();
        let mut next_index = 0u32;

        for face in mat_faces {
            for &(pos_idx, tex_idx, norm_idx) in &face {
                let vertex_key = (pos_idx, tex_idx, norm_idx);

                let index = if let Some(&idx) = vertex_map.get(&vertex_key) {
                    idx
                } else {
                    let idx = next_index;
                    next_index += 1;

                    // Add position
                    if pos_idx < positions.len() {
                        mesh.positions.push(positions[pos_idx][0]);
                        mesh.positions.push(positions[pos_idx][1]);
                        mesh.positions.push(positions[pos_idx][2]);
                    } else {
                        mesh.positions.push(0.0);
                        mesh.positions.push(0.0);
                        mesh.positions.push(0.0);
                    }

                    // Add normal
                    if let Some(nidx) = norm_idx {
                        if nidx < normals.len() {
                            mesh.normals.push(normals[nidx][0]);
                            mesh.normals.push(normals[nidx][1]);
                            mesh.normals.push(normals[nidx][2]);
                        } else {
                            mesh.normals.push(0.0);
                            mesh.normals.push(0.0);
                            mesh.normals.push(0.0);
                        }
                    }

                    // Add texcoord
                    if let Some(tidx) = tex_idx {
                        if tidx < texcoords.len() {
                            mesh.texcoords.push(texcoords[tidx][0]);
                            mesh.texcoords.push(texcoords[tidx][1]);
                        } else {
                            mesh.texcoords.push(0.0);
                            mesh.texcoords.push(0.0);
                        }
                    }

                    vertex_map.insert(vertex_key, idx);
                    idx
                };

                mesh.indices.push(index);
            }
        }

        models.push(Model { mesh });
    }

    Ok((models, materials))
}

fn load_mtl(path: &Path) -> Result<Vec<Material>, String> {
    let file = File::open(path).map_err(|e| format!("Failed to open MTL file: {}", e))?;
    let reader = BufReader::new(file);

    let mut materials = Vec::new();
    let mut current_material: Option<Material> = None;

    for line in reader.lines() {
        let line = line.map_err(|e| format!("Failed to read line: {}", e))?;
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
                // Save previous material
                if let Some(mat) = current_material.take() {
                    materials.push(mat);
                }
                // Start new material
                if parts.len() > 1 {
                    current_material = Some(Material {
                        name: parts[1].to_string(),
                        ..Default::default()
                    });
                }
            }
            "map_Kd" => {
                // Diffuse texture
                if parts.len() > 1 {
                    if let Some(ref mut mat) = current_material {
                        mat.diffuse_texture = Some(parts[1].to_string());
                    }
                }
            }
            "map_Ks" => {
                // Specular texture
                if parts.len() > 1 {
                    if let Some(ref mut mat) = current_material {
                        mat.specular_texture = Some(parts[1].to_string());
                    }
                }
            }
            "map_Bump" | "bump" => {
                // Normal/bump texture
                if parts.len() > 1 {
                    if let Some(ref mut mat) = current_material {
                        mat.normal_texture = Some(parts[1].to_string());
                    }
                }
            }
            _ => {}
        }
    }

    // Save last material
    if let Some(mat) = current_material {
        materials.push(mat);
    }

    Ok(materials)
}
