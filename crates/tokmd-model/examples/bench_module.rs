use std::fs;
use std::path::PathBuf;
use std::time::Instant;
use tokei::{Config, Languages};
use tokmd_model::create_module_report;
use tokmd_types::ChildIncludeMode;

fn main() {
    let root = PathBuf::from("target/bench_temp_module");
    if root.exists() {
        fs::remove_dir_all(&root).unwrap();
    }
    fs::create_dir_all(&root).unwrap();

    // Create 1000 files in various modules to simulate a repo
    println!("Generating 1000 files...");
    let mut paths = Vec::new();
    for i in 0..1000 {
        let module_id = i % 20; // 20 top-level modules
        let submodule_id = (i / 20) % 5; // 5 submodules per module
        let dir = root.join(format!("module_{}/sub_{}", module_id, submodule_id));
        fs::create_dir_all(&dir).unwrap();
        let file = dir.join(format!("file_{}.rs", i));
        fs::write(&file, "fn main() { println!(\"Hello\"); }").unwrap();
        // We don't strictly need to track paths here as tokei will scan the root
    }
    paths.push(root.clone());

    println!("Scanning with tokei...");
    let mut languages = Languages::new();
    languages.get_statistics(&paths, &[], &Config::default());

    println!("Benchmarking create_module_report...");
    let start = Instant::now();
    let iter = 100;

    // We use a few "module roots" to trigger the module key logic
    let module_roots = vec![
        "module_0".to_string(),
        "module_1".to_string(),
        "module_2".to_string(),
        "module_3".to_string(),
        "module_4".to_string(),
    ];

    for _ in 0..iter {
        let _ = create_module_report(
            &languages,
            &module_roots,
            2,
            ChildIncludeMode::ParentsOnly,
            50, // top
        );
    }
    let duration = start.elapsed();
    println!("Total time for {} iterations: {:?}", iter, duration);
    println!("Avg time per iteration: {:?}", duration / iter as u32);

    // Cleanup
    fs::remove_dir_all(&root).unwrap();
}
