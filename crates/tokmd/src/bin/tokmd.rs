fn main() {
    if let Err(err) = tokmd::run() {
        eprintln!("{}", tokmd::format_error(&err));
        std::process::exit(1);
    }
}
