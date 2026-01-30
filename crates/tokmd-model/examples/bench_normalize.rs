use std::path::Path;
use std::time::Instant;
use tokmd_model::normalize_path;

fn main() {
    let paths = vec![
        "src/lib.rs",
        "crates/tokmd-model/src/lib.rs",
        "./src/main.rs",
        "C:\\Windows\\System32\\driver.sys", // Windows style
        "/usr/local/bin/tokmd",
    ];
    let iterations = 1_000_000;

    let start = Instant::now();
    for _ in 0..iterations {
        for p in &paths {
            let _ = normalize_path(Path::new(p), None);
        }
    }
    let duration = start.elapsed();

    println!(
        "Time taken for {} iterations: {:?}",
        iterations * paths.len(),
        duration
    );
    println!(
        "Average time per call: {:?}",
        duration / (iterations as u32 * paths.len() as u32)
    );
}
