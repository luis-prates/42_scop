mod bmp_loader;
mod camera;
mod common;
mod macros;
mod math;
mod mesh;
mod model;
mod my_bmp_loader;
mod obj_loader;
mod rng;
mod shader;

mod model_loading;
use model_loading::*;
use std::fs::File;
use std::path::Path;

fn main() {
    let args = std::env::args().collect::<Vec<String>>();
    if args.len() != 3 {
        eprintln!(
            "Usage: {} <path_to_model> <path_to_texture>",
            args.first().map(|s| s.as_str()).unwrap_or("scop_42")
        );
        eprintln!(
            "Example: cargo run -- resources/models/teapot.obj resources/textures/brickwall.bmp"
        );
        std::process::exit(1);
    }

    let model_path = &args[1];
    let texture_path = &args[2];

    if let Err(error) = validate_cli_inputs(model_path, texture_path) {
        eprintln!("Error: {}", error);
        std::process::exit(1);
    }

    start_renderer(model_path, texture_path);
}

fn validate_cli_inputs(model_path: &str, texture_path: &str) -> Result<(), String> {
    validate_path(model_path, "obj", "model")?;
    validate_path(texture_path, "bmp", "texture")?;
    Ok(())
}

fn validate_path(path: &str, expected_extension: &str, label: &str) -> Result<(), String> {
    let file_path = Path::new(path);
    if !file_path.exists() {
        return Err(format!("{} file does not exist: {}", label, path));
    }
    if !file_path.is_file() {
        return Err(format!("{} path is not a file: {}", label, path));
    }

    let extension = file_path
        .extension()
        .and_then(|ext| ext.to_str())
        .ok_or_else(|| format!("{} file has no extension: {}", label, path))?;
    if !extension.eq_ignore_ascii_case(expected_extension) {
        return Err(format!(
            "{} file must have .{} extension: {}",
            label, expected_extension, path
        ));
    }

    File::open(file_path)
        .map(|_| ())
        .map_err(|error| format!("Failed to open {} file '{}': {}", label, path, error))
}
