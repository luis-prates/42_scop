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
    if args.len() != 2 {
        start_renderer(
            args.get(1).map(|s| s.as_str()).unwrap(),
            args.get(2).map(|s| s.as_str()).unwrap(),
        );
    } else {
        println!("Usage: cargo run -- <path_to_model> <path_to_texture>");
    }
}
