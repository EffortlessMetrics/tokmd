use std::path::PathBuf;
use tempfile::TempDir;
use tokmd_analysis_content::{ContentLimits, build_duplicate_report, build_todo_report};
use tokmd_types::*;

fn make_row(path: &str, module: &str, lang: &str, bytes: usize) -> FileRow {
    FileRow {
        path: path.to_string(),
        module: module.to_string(),
        lang: lang.to_string(),
        bytes,
        kind: FileKind::Parent,
        code: 0,
        comments: 0,
        blanks: 0,
        lines: 0,
        tokens: 0,
    }
}

fn make_export(rows: Vec<FileRow>) -> ExportData {
    ExportData {
        rows,
        module_roots: vec!["root".to_string()],
        module_depth: 1,
        children: ChildIncludeMode::Separate,
    }
}

#[test]
fn test_build_duplicate_report_size_limit_boundary() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();

    let content = "dup\n"; // 4 bytes
    std::fs::write(root.join("a.txt"), content).unwrap();
    std::fs::write(root.join("b.txt"), content).unwrap();

    let files = vec![PathBuf::from("a.txt"), PathBuf::from("b.txt")];
    let export = make_export(vec![
        make_row("a.txt", "root", "Text", content.len()),
        make_row("b.txt", "root", "Text", content.len()),
    ]);

    // Test with limit exactly equal to file size - should NOT be skipped
    // catches `replace > with >=` on `size > limit`
    let limits_exact = ContentLimits {
        max_bytes: None,
        max_file_bytes: Some(4),
    };

    let report = build_duplicate_report(root, &files, &export, &limits_exact).unwrap();
    assert_eq!(
        report.groups.len(),
        1,
        "files exactly at limit should be included"
    );
    assert_eq!(report.wasted_bytes, 4);

    // Test with limit less than file size - should be skipped
    let limits_less = ContentLimits {
        max_bytes: None,
        max_file_bytes: Some(3),
    };

    let report_less = build_duplicate_report(root, &files, &export, &limits_less).unwrap();
    assert_eq!(
        report_less.groups.len(),
        0,
        "files exceeding limit should be skipped"
    );
}

#[test]
fn test_duplicate_report_wasted_bytes_three_files() {
    // Tests catching the `wasted_bytes += ...` mutation from `(files.len() - 1) * size`
    let temp = TempDir::new().unwrap();
    let root = temp.path();

    let content = "dup content\n"; // 12 bytes
    std::fs::write(root.join("a.txt"), content).unwrap();
    std::fs::write(root.join("b.txt"), content).unwrap();
    std::fs::write(root.join("c.txt"), content).unwrap();

    let files = vec![
        PathBuf::from("a.txt"),
        PathBuf::from("b.txt"),
        PathBuf::from("c.txt"),
    ];
    let export = make_export(vec![
        make_row("a.txt", "root", "Text", content.len()),
        make_row("b.txt", "root", "Text", content.len()),
        make_row("c.txt", "root", "Text", content.len()),
    ]);

    let report = build_duplicate_report(root, &files, &export, &ContentLimits::default()).unwrap();

    // (3 - 1) * 12 = 24 wasted bytes
    assert_eq!(report.wasted_bytes, 24);

    let density = report.density.unwrap();
    assert_eq!(density.wasted_bytes, 24);
    assert!((density.wasted_pct_of_codebase - 0.6667).abs() < 0.0001);

    let by_module = &density.by_module[0];
    assert_eq!(by_module.wasted_bytes, 24);
    assert_eq!(by_module.duplicate_files, 3);
    assert_eq!(by_module.wasted_files, 2);
}

#[test]
fn test_duplicate_report_wasted_bytes_three_files_catch_mul_mutant() {
    // Tests catching the `duplicated_bytes += size` mutation to `*= size`
    let temp = TempDir::new().unwrap();
    let root = temp.path();

    let content = "dup\n"; // 4 bytes
    std::fs::write(root.join("a.txt"), content).unwrap();
    std::fs::write(root.join("b.txt"), content).unwrap();
    std::fs::write(root.join("c.txt"), content).unwrap();

    let files = vec![
        PathBuf::from("a.txt"),
        PathBuf::from("b.txt"),
        PathBuf::from("c.txt"),
    ];
    let export = make_export(vec![
        make_row("a.txt", "root", "Text", content.len()),
        make_row("b.txt", "root", "Text", content.len()),
        make_row("c.txt", "root", "Text", content.len()),
    ]);

    let report = build_duplicate_report(root, &files, &export, &ContentLimits::default()).unwrap();

    let density = report.density.unwrap();
    assert_eq!(density.duplicated_bytes, 12); // 4 + 4 + 4, not 4 * 4 * 4 = 64

    let by_module = &density.by_module[0];
    assert_eq!(by_module.duplicated_bytes, 12);
}

#[test]
fn test_density_wasted_pct_of_codebase() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();

    let dup = "dup\n"; // 4 bytes
    let unique = "unique content\n"; // 15 bytes

    std::fs::write(root.join("d1.txt"), dup).unwrap();
    std::fs::write(root.join("d2.txt"), dup).unwrap();
    std::fs::write(root.join("u.txt"), unique).unwrap();

    let files = vec![
        PathBuf::from("d1.txt"),
        PathBuf::from("d2.txt"),
        PathBuf::from("u.txt"),
    ];
    let export = make_export(vec![
        make_row("d1.txt", "root", "Text", dup.len()),
        make_row("d2.txt", "root", "Text", dup.len()),
        make_row("u.txt", "root", "Text", unique.len()),
    ]);

    let report = build_duplicate_report(root, &files, &export, &ContentLimits::default()).unwrap();

    // 1 duplicate file (d2), 4 wasted bytes
    // total codebase bytes = 4 + 4 + 15 = 23
    // wasted pct = 4 / 23 = 0.1739
    assert_eq!(report.wasted_bytes, 4);

    let density = report.density.unwrap();
    assert!((density.wasted_pct_of_codebase - 0.1739).abs() < 0.0001);
}

#[test]
fn test_density_wasted_pct_of_codebase_catch_div_mutant() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();

    let dup = "dup\n"; // 4 bytes
    let unique = "unique content\n"; // 15 bytes

    std::fs::write(root.join("d1.txt"), dup).unwrap();
    std::fs::write(root.join("d2.txt"), dup).unwrap();
    std::fs::write(root.join("u.txt"), unique).unwrap();

    let files = vec![
        PathBuf::from("d1.txt"),
        PathBuf::from("d2.txt"),
        PathBuf::from("u.txt"),
    ];
    let export = make_export(vec![
        make_row("d1.txt", "root", "Text", dup.len()),
        make_row("d2.txt", "root", "Text", dup.len()),
        make_row("u.txt", "root", "Text", unique.len()),
    ]);

    let report = build_duplicate_report(root, &files, &export, &ContentLimits::default()).unwrap();

    let density = report.density.unwrap();
    // 4 wasted bytes / 23 total bytes = 0.1739
    // mutant `/` -> `*` would be 4 * 23 = 92
    // mutant `/` -> `%` would be 4 % 23 = 4
    let val = density.wasted_pct_of_codebase;
    assert!(
        (val - 0.1739).abs() < 0.0001,
        "expected ~0.1739, got {}",
        val
    );
}

#[test]
fn test_todo_report_max_bytes_edge_cases() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();

    std::fs::write(root.join("a.rs"), "// TODO: a\n").unwrap(); // 11 bytes

    // Default limit should be exactly 128 * 1024, if changed to + it will be 128 + 1024 = 1152
    // If it's a + mutant, a file size of 2000 will be greater than the limit and skip it.
    let big_content = "// TODO: a\n".repeat(200); // 2200 bytes
    std::fs::write(root.join("b.rs"), &big_content).unwrap();
    let big_files = vec![PathBuf::from("b.rs")];

    // Test with default limits (which uses DEFAULT_MAX_FILE_BYTES)
    let limits = ContentLimits::default();

    let report = build_todo_report(root, &big_files, &limits, 1000).unwrap();
    assert_eq!(
        report.total, 200,
        "Should read all 2000+ bytes since 128 * 1024 is much larger"
    );
}

#[test]
fn test_density_module_total_zero_codebase() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();

    let dup = "dup\n"; // 4 bytes

    std::fs::write(root.join("d1.txt"), dup).unwrap();
    std::fs::write(root.join("d2.txt"), dup).unwrap();

    let files = vec![PathBuf::from("d1.txt"), PathBuf::from("d2.txt")];
    let export = make_export(vec![
        // Simulate a scenario where the file rows map to 0 bytes,
        // e.g. for module_total == 0
        make_row("d1.txt", "root", "Text", 0),
        make_row("d2.txt", "root", "Text", 0),
    ]);

    let report = build_duplicate_report(root, &files, &export, &ContentLimits::default()).unwrap();

    let density = report.density.unwrap();
    let by_module = &density.by_module[0];

    // the mutation `==` to `!=` would make `module_total != 0` evaluate to false (if it's 0)
    // then it would hit the else branch and divide by zero, resulting in NaN or panic
    // Wait, the mutation replaces `if module_total == 0` with `if module_total != 0`.
    // If it's `!= 0`, and total IS 0, it hits the `else` block: 4 / 0.0 -> +inf or NaN, catching the mutant.
    assert_eq!(by_module.density, 0.0);
}

#[test]
fn test_density_module_density_calc() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();

    let dup = "dup\n"; // 4 bytes

    std::fs::write(root.join("d1.txt"), dup).unwrap();
    std::fs::write(root.join("d2.txt"), dup).unwrap();

    let files = vec![PathBuf::from("d1.txt"), PathBuf::from("d2.txt")];
    let export = make_export(vec![
        // module_total = 23
        make_row("d1.txt", "root", "Text", 15),
        make_row("d2.txt", "root", "Text", 8),
    ]);

    let report = build_duplicate_report(root, &files, &export, &ContentLimits::default()).unwrap();

    let density = report.density.unwrap();
    let by_module = &density.by_module[0];

    // wasted bytes = 4
    // module total = 23
    // density = 4 / 23 = 0.1739

    assert!(
        (by_module.density - 0.1739).abs() < 0.0001,
        "Expected ~0.1739, got {}",
        by_module.density
    );
}
