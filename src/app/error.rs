use std::fmt;

#[derive(Debug)]
pub enum AppError {
    Cli(String),
    SceneBuild(String),
    Renderer(String),
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AppError::Cli(message) => write!(f, "{}", message),
            AppError::SceneBuild(message) => write!(f, "{}", message),
            AppError::Renderer(message) => write!(f, "{}", message),
        }
    }
}

impl std::error::Error for AppError {}
