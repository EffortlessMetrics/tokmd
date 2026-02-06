use std::time::Instant;

use serde_json::json;
use tokei::{Language, LanguageType, Languages, Report};

fn main() {
    // Generate synthetic data
    // 100 modules, 1000 files each = 100,000 files
    let modules = 100;
    let files_per_module = 1000;
    let iterations = 10;

    println!(
        "Generating {} files across {} modules...",
        modules * files_per_module,
        modules
    );

    let mut languages = Languages::new();
    let mut reports = Vec::new();

    for m in 0..modules {
        let module_name = format!("crates/module_{}", m);
        for f in 0..files_per_module {
            let path_str = format!("{}/src/file_{}.rs", module_name, f);

            // Construct Report via JSON because Report is non_exhaustive
            let report_json = json!({
                "name": path_str,
                "stats": {
                    "blanks": 0,
                    "code": 100,
                    "comments": 0,
                    "blobs": {}
                }
            });

            let report: Report =
                serde_json::from_value(report_json).expect("Failed to deserialize Report");
            reports.push(report);
        }
    }

    let mut rust_lang = Language::new();
    rust_lang.reports = reports;
    languages.insert(LanguageType::Rust, rust_lang);

    let module_roots = vec!["crates".to_string()];
    let module_depth = 2;
    let children_mode = tokmd_types::ChildIncludeMode::Separate;
    let top = 0;

    println!("Starting benchmark ({} iterations)...", iterations);
    let start = Instant::now();

    for _ in 0..iterations {
        let _report = tokmd_model::create_module_report(
            &languages,
            &module_roots,
            module_depth,
            children_mode,
            top,
        );
    }

    let duration = start.elapsed();
    let avg = duration / iterations;

    println!("Total time: {:?}", duration);
    println!("Average time per call: {:?}", avg);
}
