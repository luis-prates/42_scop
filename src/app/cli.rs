use std::fs::File;
use std::path::Path;

#[derive(Debug, Clone)]
pub struct AppConfig {
    pub model_path: String,
    pub texture_path: String,
}

pub fn parse_from_env() -> Result<AppConfig, String> {
    let args = std::env::args().collect::<Vec<String>>();
    if args.len() != 3 {
        return Err(format!(
            "Usage: {} <path_to_model> <path_to_texture>\nExample: cargo run -- resources/models/teapot.obj resources/textures/brickwall.bmp",
            args.first().map(|s| s.as_str()).unwrap_or("scop_42")
        ));
    }

    let config = AppConfig {
        model_path: args[1].clone(),
        texture_path: args[2].clone(),
    };

    validate_cli_inputs(&config.model_path, &config.texture_path)?;
    Ok(config)
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
