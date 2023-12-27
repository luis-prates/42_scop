
mod common;
mod shader;
mod macros;
mod camera;
mod model;
mod mesh;
mod math;
mod my_bmp_loader;
mod bmp_loader;

mod _3_model_loading;
use _3_model_loading::*;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 2 {
        println!("Call with the number of the tutorial, e.g. `1_1_2` for _1_2_hello_window_clear.rs");
        std::process::exit(1);
    }
    let tutorial_id = &args[1];

    match tutorial_id.as_str() {
		// #[cfg(feature = "chapter-3")] "3_1"   => main_3_1(),
		"3_2"   => main_3_2(),

        _     => println!("Unknown tutorial id")
    }
}