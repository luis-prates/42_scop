use scop_42::app;

fn main() {
    if let Err(error) = app::run_from_env() {
        eprintln!("Error: {}", error);
        std::process::exit(1);
    }
}
