use std::path::Path;
use std::time::Instant;
use tokmd_model::{normalize_path, normalize_path_str};

fn main() {
    let paths = vec![
        "src/lib.rs",
        "crates/tokmd-model/src/lib.rs",
        "./src/main.rs",
        "C:\\Windows\\System32\\driver.sys", // Windows style
        "/usr/local/bin/tokmd",
    ];
    let iterations = 1_000_000;

    println!("Benchmarking NO PREFIX...");
    let start = Instant::now();
    for _ in 0..iterations {
        for p in &paths {
            let _ = normalize_path(Path::new(p), None);
        }
    }
    let duration = start.elapsed();
    println!(
        "Total time: {:?}, Avg per call: {:?}",
        duration,
        duration / (iterations as u32 * paths.len() as u32)
    );

    let prefix_str = "crates/tokmd-model";
    let prefix = Path::new(prefix_str);
    println!(
        "Benchmarking WITH PREFIX (Old style - repetitive normalization): {:?}",
        prefix
    );

    let start_prefix = Instant::now();
    for _ in 0..iterations {
        for p in &paths {
            let _ = normalize_path(Path::new(p), Some(prefix));
        }
    }
    let duration_prefix = start_prefix.elapsed();
    println!(
        "Total time: {:?}, Avg per call: {:?}",
        duration_prefix,
        duration_prefix / (iterations as u32 * paths.len() as u32)
    );

    let prefix_normalized = "crates/tokmd-model/";
    println!(
        "Benchmarking WITH PRE-NORMALIZED PREFIX: {:?}",
        prefix_normalized
    );

    let start_opt = Instant::now();
    for _ in 0..iterations {
        for p in &paths {
            let _ = normalize_path_str(Path::new(p), Some(prefix_normalized));
        }
    }
    let duration_opt = start_opt.elapsed();
    println!(
        "Total time: {:?}, Avg per call: {:?}",
        duration_opt,
        duration_opt / (iterations as u32 * paths.len() as u32)
    );
}
