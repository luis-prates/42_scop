pub mod cli;
pub mod error;

use crate::renderer;
use crate::scene;

use cli::AppConfig;
use error::AppError;

pub fn run_from_env() -> Result<(), AppError> {
    let config = cli::parse_from_env().map_err(AppError::Cli)?;
    run(config)
}

pub fn run(config: AppConfig) -> Result<(), AppError> {
    let scene_model = scene::build_scene_model(&config.model_path, &config.texture_path)
        .map_err(AppError::SceneBuild)?;
    renderer::run(scene_model).map_err(AppError::Renderer)
}
