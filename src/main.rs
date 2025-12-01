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

fn main() {
    let args = std::env::args().collect::<Vec<String>>();
    if args.len() == 3 {
        start_renderer(
            args.get(1).map(|s| s.as_str()).unwrap(),
            args.get(2).map(|s| s.as_str()).unwrap(),
        );
    } else {
        eprintln!(
            "Usage: {} <path_to_model> <path_to_texture>",
            args.first().map(|s| s.as_str()).unwrap_or("scop_42")
        );
        eprintln!(
            "Example: cargo run -- resources/objects/teapot.obj resources/textures/brickwall.bmp"
        );
        std::process::exit(1);
    }
}
