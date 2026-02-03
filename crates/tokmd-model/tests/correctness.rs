use std::fs;
use std::path::PathBuf;
use tokei::{Config, Languages};
use tokmd_model::create_module_report;
use tokmd_types::ChildIncludeMode;

#[test]
fn module_report_counts_all_files_despite_top() {
    let root = PathBuf::from("target/test_temp_correctness");
    if root.exists() {
        fs::remove_dir_all(&root).unwrap();
    }
    fs::create_dir_all(&root).unwrap();

    // Create 10 files in "module_a"
    let mod_a = root.join("module_a");
    fs::create_dir_all(&mod_a).unwrap();
    for i in 0..10 {
        fs::write(mod_a.join(format!("f{}.rs", i)), "fn main() {}").unwrap();
    }

    // Create 10 files in "module_b"
    let mod_b = root.join("module_b");
    fs::create_dir_all(&mod_b).unwrap();
    for i in 0..10 {
        fs::write(mod_b.join(format!("f{}.rs", i)), "fn main() {}").unwrap();
    }

    // Scan
    let mut languages = Languages::new();
    languages.get_statistics(&[&root], &[], &Config::default());

    // Call create_module_report with top = 1
    // This should result in 1 module row + "Other".
    // But the "Other" row should correctly sum up the remaining files.
    // And if we check the rows, the file counts for the visible modules must be correct (10).

    // Paths are like: target/test_temp_correctness/module_a/f0.rs
    // To separate them, we treat "target" as a root, and depth 3 to reach module_a.
    // key: target/test_temp_correctness/module_a

    let report = create_module_report(
        &languages,
        &["target".to_string()],
        3,
        ChildIncludeMode::ParentsOnly,
        1, // top = 1
    );

    // Verify Total Files
    // Total files should be 20.
    assert_eq!(report.total.files, 20);

    // Verify Rows
    // We expect 2 rows: one for the top module (either A or B), and one for "Other".
    // Wait, create_module_report logic:
    // if rows.len() > top { ... }
    // Here rows.len() is 2. top is 1. 2 > 1.
    // truncate(1). push(other). Result len is 2.
    assert_eq!(report.rows.len(), 2);

    let top_row = &report.rows[0];
    assert!(
        top_row.module.contains("module_a") || top_row.module.contains("module_b"),
        "Module name {} should contain module_a or module_b",
        top_row.module
    );
    assert_eq!(top_row.files, 10, "Top module should have 10 files");

    let other_row = &report.rows[1];
    assert_eq!(other_row.module, "Other");
    assert_eq!(other_row.files, 10, "Other module should have 10 files");

    // Cleanup
    fs::remove_dir_all(&root).unwrap();
}
