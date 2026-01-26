fn main() {
    if let Err(err) = tokmd::run() {
        eprintln!("Error: {err:#}");
        std::process::exit(1);
    }
}
