mod bounds;
mod coloring;
mod model;
mod model_builder;

pub use model::{SceneMesh, SceneModel, SceneTextureRef, TextureKind, Vertex};
pub use model_builder::build_scene_model;
