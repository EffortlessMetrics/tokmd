use std::fs;
use std::time::Instant;
use tokei::{Config, Languages};
use tokmd_model::collect_file_rows;
use tokmd_types::ChildIncludeMode;

#[test]
#[ignore]
fn bench_collect_file_rows() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();

    // Create 1,000 files to simulate a medium/large repo
    let file_count = 1000;
    println!("Creating {} files...", file_count);

    // Create nested structure
    for i in 0..file_count {
        let p = root.join(format!("file_{}.rs", i));
        fs::write(&p, "fn main() { println!(\"hello\"); }").unwrap();
    }

    println!("Scanning files...");
    let mut languages = Languages::new();
    let paths = vec![root.to_path_buf()];
    let excluded = vec![];
    let config = Config::default();
    languages.get_statistics(&paths, &excluded, &config);

    let module_roots = vec![];

    println!("Benchmarking...");
    let start = Instant::now();
    let iterations = 500;
    for _ in 0..iterations {
        // We pass the same languages map every time.
        // collect_file_rows does not modify it.
        let _ = collect_file_rows(
            &languages,
            &module_roots,
            0,
            ChildIncludeMode::ParentsOnly,
            None,
        );
    }
    let duration = start.elapsed();

    println!("Total time for {} iterations: {:?}", iterations, duration);
    let per_iter = duration / iterations as u32;
    println!("Time per iteration: {:?}", per_iter);
}
