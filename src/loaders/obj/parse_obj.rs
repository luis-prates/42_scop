use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

use super::index::{FaceVertex, parse_f32_component, parse_face_vertex};
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
    let mut mtl_files: Vec<String> = Vec::new();

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
                    if item.starts_with('#') {
                        break;
                    }
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
                let material_name =
                    collect_directive_values(parts.as_slice(), "usemtl", line_number)?
                        .into_iter()
                        .next()
                        .ok_or_else(|| {
                            format!(
                                "Line {}: directive '{}' is missing a required value",
                                line_number, "usemtl"
                            )
                        })?;
                current_material = Some(material_name.to_string());
            }
            "mtllib" => {
                let file_names = collect_directive_values(parts.as_slice(), "mtllib", line_number)?;
                mtl_files.extend(file_names.into_iter().map(str::to_string));
            }
            _ => {}
        }
    }

    let mut materials = Vec::new();
    for mtl_filename in mtl_files {
        let mtl_path = path
            .parent()
            .unwrap_or_else(|| Path::new(""))
            .join(mtl_filename);
        materials.extend(load_mtl(&mtl_path)?);
    }

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
                let position = positions[pos_idx];
                mesh.positions.extend_from_slice(&position);

                let normal = norm_idx
                    .map(|normal_idx| normals[normal_idx])
                    .unwrap_or([0.0, 0.0, 0.0]);
                mesh.normals.extend_from_slice(&normal);

                let texcoord = tex_idx.map(|texcoord_idx| texcoords[texcoord_idx]);
                vertex_texcoords.push(texcoord);

                mesh.indices.push(next_index);
                next_index += 1;
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

fn collect_directive_values<'a>(
    parts: &[&'a str],
    directive: &str,
    line_number: usize,
) -> Result<Vec<&'a str>, String> {
    if parts.first().copied() != Some(directive) {
        return Err(format!(
            "Line {}: expected directive '{}'",
            line_number, directive
        ));
    }

    let mut values = Vec::new();
    for token in parts.iter().skip(1) {
        if token.starts_with('#') {
            break;
        }
        values.push(*token);
    }

    if values.is_empty() {
        return Err(format!(
            "Line {}: directive '{}' is missing a required value",
            line_number, directive
        ));
    }

    Ok(values)
}

#[cfg(test)]
mod tests {
    use std::env;
    use std::fs;
    use std::path::PathBuf;
    use std::process;
    use std::time::{SystemTime, UNIX_EPOCH};

    use super::load;
    use crate::loaders::obj::ObjLoadOptions;

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
    fn parses_face_line_with_inline_comment() {
        let dir = unique_temp_dir("scop_obj_inline_comment");
        let obj_path = dir.join("inline_comment.obj");
        let obj_data = "\
v 0 0 0
v 1 0 0
v 0 1 0
f 1 2 3 # triangle
";
        fs::write(&obj_path, obj_data).expect("failed to write OBJ fixture");

        let scene = load(
            &obj_path,
            &ObjLoadOptions {
                triangulate: true,
                single_index: false,
            },
        )
        .expect("OBJ with inline face comment should parse");

        assert_eq!(scene.objects.len(), 1);
        assert_eq!(scene.objects[0].mesh.indices, vec![0, 1, 2]);

        fs::remove_dir_all(&dir).expect("failed to cleanup temp directory");
    }

    #[test]
    fn parses_multiple_mtllib_entries() {
        let dir = unique_temp_dir("scop_obj_multi_mtllib");
        let obj_path = dir.join("multi_mtllib.obj");
        fs::write(
            dir.join("a.mtl"),
            "\
newmtl MatA
",
        )
        .expect("failed to write a.mtl");
        fs::write(
            dir.join("b.mtl"),
            "\
newmtl MatB
",
        )
        .expect("failed to write b.mtl");
        let obj_data = "\
mtllib a.mtl b.mtl
usemtl MatB
v 0 0 0
v 1 0 0
v 0 1 0
f 1 2 3
";
        fs::write(&obj_path, obj_data).expect("failed to write OBJ fixture");

        let scene = load(
            &obj_path,
            &ObjLoadOptions {
                triangulate: true,
                single_index: false,
            },
        )
        .expect("OBJ with multiple mtllib files should parse");

        assert_eq!(scene.materials.len(), 2);
        assert!(scene.materials.iter().any(|mat| mat.name == "MatA"));
        assert!(scene.materials.iter().any(|mat| mat.name == "MatB"));

        let mat_b_index = scene
            .materials
            .iter()
            .position(|mat| mat.name == "MatB")
            .expect("MatB should be loaded");
        assert_eq!(scene.objects.len(), 1);
        assert_eq!(scene.objects[0].mesh.material_id, Some(mat_b_index));

        fs::remove_dir_all(&dir).expect("failed to cleanup temp directory");
    }
}
