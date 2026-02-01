use std::fs::File;
use std::io::Write;
use std::time::Instant;
use tempfile::tempdir;
use tokei::{Config, Languages};
use tokmd_model::create_module_report;
use tokmd_types::ChildIncludeMode;

fn main() {
    let dir = tempdir().unwrap();
    let root = dir.path();

    println!("Generating files in {:?}", root);
    // Generate files
    for i in 0..1000 {
        let module_id = i % 20;
        let module_path = root.join(format!("module_{}", module_id));
        std::fs::create_dir_all(&module_path).unwrap();
        let file_path = module_path.join(format!("file_{}.rs", i));
        let mut file = File::create(file_path).unwrap();
        writeln!(file, "fn main() {{ println!(\"Hello\"); }}").unwrap();
    }

    println!("Scanning files...");
    let mut languages = Languages::new();
    let paths = vec![root];
    let config = Config::default();
    languages.get_statistics(&paths, &[], &config);

    let module_roots = vec!["module_0".to_string(), "module_1".to_string()];

    println!("Benchmarking create_module_report...");
    let start = Instant::now();
    let iterations = 1000;
    for _ in 0..iterations {
        let _ = create_module_report(
            &languages,
            &module_roots,
            2,
            ChildIncludeMode::ParentsOnly,
            10,
        );
    }
    let duration = start.elapsed();
    println!("Total time for {} iterations: {:?}", iterations, duration);
    println!(
        "Average time per iteration: {:?}",
        duration / iterations as u32
    );
}
