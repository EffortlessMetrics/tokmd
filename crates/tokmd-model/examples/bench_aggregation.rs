use std::fs;
use std::path::{Path, PathBuf};
use std::time::Instant;
use tokei::{Config, Languages};
use tokmd_model::collect_file_rows;
use tokmd_types::ChildIncludeMode;

fn main() {
    let mut root = std::env::temp_dir();
    root.push("bench_tokmd_model");
    if root.exists() {
        fs::remove_dir_all(&root).unwrap();
    }
    fs::create_dir_all(&root).unwrap();

    println!("Preparing benchmark in {:?}", root);

    // Generate many files
    let num_files = 10_000;

    // Create a dummy file content once
    let content = b"fn main() {}";

    for i in 0..num_files {
        let rel_path = format!("src/module_{}/file_{}.rs", i % 50, i);
        let name = root.join(&rel_path);
        if let Some(parent) = name.parent() {
            fs::create_dir_all(parent).unwrap();
        }
        fs::write(&name, content).unwrap();
    }

    println!("Scanning with tokei...");
    let mut languages = Languages::new();
    let config = Config::default();
    let paths = vec![root.clone()];
    languages.get_statistics(&paths, &[], &config);

    let module_roots = vec!["src".to_string()];
    let module_depth = 1;

    println!("Starting benchmark with {} files...", num_files);

    // Warmup
    let _ = collect_file_rows(
        &languages,
        &module_roots,
        module_depth,
        ChildIncludeMode::ParentsOnly,
        Some(&root), // Strip the temp root prefix
    );

    let start = Instant::now();
    let iterations = 20;
    let mut total_rows = 0;

    for _ in 0..iterations {
        let rows = collect_file_rows(
            &languages,
            &module_roots,
            module_depth,
            ChildIncludeMode::ParentsOnly,
            Some(&root),
        );
        total_rows += rows.len();
    }

    let duration = start.elapsed();
    println!(
        "collect_file_rows took: {:?} for {} iterations",
        duration, iterations
    );
    println!("Average per call: {:?}", duration / iterations as u32);
    println!("Total rows collected: {}", total_rows / iterations);

    // Clean up
    fs::remove_dir_all(&root).unwrap();
}
