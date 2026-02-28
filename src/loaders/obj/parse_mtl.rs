use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

use super::index::directive_value;
use super::types::ObjMaterialData;

pub fn load_mtl(path: &Path) -> Result<Vec<ObjMaterialData>, String> {
    let file = File::open(path)
        .map_err(|e| format!("Failed to open MTL file '{}': {}", path.display(), e))?;
    let reader = BufReader::new(file);

    let mut materials = Vec::new();
    let mut current_material: Option<ObjMaterialData> = None;

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
                if let Some(mat) = current_material.take() {
                    materials.push(mat);
                }

                let material_name = directive_value(line, "newmtl", line_number)?;
                current_material = Some(ObjMaterialData {
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

    if let Some(mat) = current_material {
        materials.push(mat);
    }

    Ok(materials)
}
