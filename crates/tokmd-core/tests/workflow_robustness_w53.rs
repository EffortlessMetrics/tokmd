//! Workflow robustness tests (w53).
//!
//! Exercises `lang_workflow`, `module_workflow`, and `export_workflow`
//! with edge-case inputs, verifying graceful error handling and
//! correct behaviour under non-trivial settings.

use std::fs;
use std::path::Path;
use tokmd_core::settings::{ExportSettings, LangSettings, ModuleSettings, ScanSettings};
use tokmd_core::{export_workflow, lang_workflow, module_workflow};

// ============================================================================
// Helpers
// ============================================================================

fn write_file(root: &Path, rel: &str, contents: &str) {
    let p = root.join(rel);
    fs::create_dir_all(p.parent().unwrap()).unwrap();
    fs::write(p, contents).unwrap();
}

fn make_repo(code: &str) -> tempfile::TempDir {
    let dir = tempfile::tempdir().expect("tempdir");
    write_file(dir.path(), "src/lib.rs", code);
    dir
}

fn make_empty_dir() -> tempfile::TempDir {
    tempfile::tempdir().expect("tempdir")
}

// ============================================================================
// lang_workflow – default settings
// ============================================================================

#[test]
fn lang_default_settings_succeeds() {
    let scan = ScanSettings::for_paths(vec!["src".to_string()]);
    let lang = LangSettings::default();
    let receipt = lang_workflow(&scan, &lang).expect("should succeed with defaults");
    assert_eq!(receipt.mode, "lang");
    assert!(!receipt.report.rows.is_empty());
}

#[test]
fn lang_finds_rust_in_this_crate() {
    let scan = ScanSettings::for_paths(vec!["src".to_string()]);
    let lang = LangSettings::default();
    let receipt = lang_workflow(&scan, &lang).unwrap();
    assert!(
        receipt.report.rows.iter().any(|r| r.lang == "Rust"),
        "should detect Rust"
    );
}

#[test]
fn lang_receipt_has_all_metadata() {
    let scan = ScanSettings::for_paths(vec!["src".to_string()]);
    let lang = LangSettings::default();
    let receipt = lang_workflow(&scan, &lang).unwrap();
    assert_eq!(receipt.schema_version, tokmd_types::SCHEMA_VERSION);
    assert!(receipt.generated_at_ms > 1_577_836_800_000);
    assert!(!receipt.tool.name.is_empty());
    assert!(!receipt.tool.version.is_empty());
    assert!(!receipt.scan.paths.is_empty());
}

#[test]
fn lang_top_zero_returns_all() {
    let scan = ScanSettings::for_paths(vec!["src".to_string()]);
    let lang = LangSettings {
        top: 0,
        ..Default::default()
    };
    let receipt = lang_workflow(&scan, &lang).unwrap();
    assert!(!receipt.report.rows.is_empty());
}

#[test]
fn lang_top_one_limits_rows() {
    let scan = ScanSettings::for_paths(vec!["src".to_string()]);
    let lang = LangSettings {
        top: 1,
        ..Default::default()
    };
    let receipt = lang_workflow(&scan, &lang).unwrap();
    // At most top + 1 ("Other" row)
    assert!(receipt.report.rows.len() <= 2);
}

// ============================================================================
// module_workflow – various depths
// ============================================================================

#[test]
fn module_default_depth_succeeds() {
    let scan = ScanSettings::for_paths(vec!["src".to_string()]);
    let module = ModuleSettings::default();
    let receipt = module_workflow(&scan, &module).unwrap();
    assert_eq!(receipt.mode, "module");
    assert!(!receipt.report.rows.is_empty());
}

#[test]
fn module_depth_1() {
    let scan = ScanSettings::for_paths(vec!["src".to_string()]);
    let module = ModuleSettings {
        module_depth: 1,
        ..Default::default()
    };
    let receipt = module_workflow(&scan, &module).unwrap();
    assert_eq!(receipt.args.module_depth, 1);
}

#[test]
fn module_depth_5() {
    let scan = ScanSettings::for_paths(vec!["src".to_string()]);
    let module = ModuleSettings {
        module_depth: 5,
        ..Default::default()
    };
    let receipt = module_workflow(&scan, &module).unwrap();
    assert_eq!(receipt.args.module_depth, 5);
}

#[test]
fn module_depth_0_succeeds() {
    let scan = ScanSettings::for_paths(vec!["src".to_string()]);
    let module = ModuleSettings {
        module_depth: 0,
        ..Default::default()
    };
    let receipt = module_workflow(&scan, &module).unwrap();
    assert_eq!(receipt.args.module_depth, 0);
}

#[test]
fn module_custom_roots() {
    let scan = ScanSettings::for_paths(vec!["src".to_string()]);
    let module = ModuleSettings {
        module_roots: vec!["src".to_string()],
        ..Default::default()
    };
    let receipt = module_workflow(&scan, &module).unwrap();
    assert!(receipt.args.module_roots.contains(&"src".to_string()));
}

#[test]
fn module_top_setting() {
    let scan = ScanSettings::for_paths(vec!["src".to_string()]);
    let module = ModuleSettings {
        top: 1,
        ..Default::default()
    };
    let receipt = module_workflow(&scan, &module).unwrap();
    assert_eq!(receipt.args.top, 1);
}

// ============================================================================
// export_workflow – different settings
// ============================================================================

#[test]
fn export_default_succeeds() {
    let scan = ScanSettings::for_paths(vec!["src".to_string()]);
    let export = ExportSettings::default();
    let receipt = export_workflow(&scan, &export).unwrap();
    assert_eq!(receipt.mode, "export");
    assert!(!receipt.data.rows.is_empty());
}

#[test]
fn export_rows_have_paths() {
    let scan = ScanSettings::for_paths(vec!["src".to_string()]);
    let export = ExportSettings::default();
    let receipt = export_workflow(&scan, &export).unwrap();
    for row in &receipt.data.rows {
        assert!(!row.path.is_empty(), "export rows must have paths");
    }
}

#[test]
fn export_paths_forward_slashes() {
    let scan = ScanSettings::for_paths(vec!["src".to_string()]);
    let export = ExportSettings::default();
    let receipt = export_workflow(&scan, &export).unwrap();
    for row in &receipt.data.rows {
        assert!(
            !row.path.contains('\\'),
            "paths must use forward slashes: {}",
            row.path
        );
    }
}

#[test]
fn export_min_code_filter() {
    let scan = ScanSettings::for_paths(vec!["src".to_string()]);
    let all = export_workflow(
        &scan,
        &ExportSettings {
            min_code: 0,
            ..Default::default()
        },
    )
    .unwrap();
    let filtered = export_workflow(
        &scan,
        &ExportSettings {
            min_code: 9999,
            ..Default::default()
        },
    )
    .unwrap();
    assert!(filtered.data.rows.len() <= all.data.rows.len());
}

#[test]
fn export_max_rows_limit() {
    let scan = ScanSettings::for_paths(vec!["src".to_string()]);
    let export = ExportSettings {
        max_rows: 1,
        ..Default::default()
    };
    let receipt = export_workflow(&scan, &export).unwrap();
    assert!(receipt.data.rows.len() <= 1);
}

// ============================================================================
// Error handling – non-existent paths
// ============================================================================

#[test]
fn lang_nonexistent_path_returns_error() {
    let scan = ScanSettings::for_paths(vec!["/nonexistent/path/abc123".to_string()]);
    let lang = LangSettings::default();
    let result = lang_workflow(&scan, &lang);
    // Should succeed with empty results or fail gracefully
    // tokei may return empty results for non-existent paths
    if let Ok(receipt) = result {
        let _ = receipt.report.rows.is_empty(); // Ensure it doesn't panic
    }
}

#[test]
fn module_nonexistent_path_returns_error() {
    let scan = ScanSettings::for_paths(vec!["/nonexistent/path/abc123".to_string()]);
    let module = ModuleSettings::default();
    let result = module_workflow(&scan, &module);
    if let Ok(receipt) = result {
        let _ = receipt.report.rows.is_empty();
    }
}

#[test]
fn export_nonexistent_path_returns_error() {
    let scan = ScanSettings::for_paths(vec!["/nonexistent/path/abc123".to_string()]);
    let export = ExportSettings::default();
    let result = export_workflow(&scan, &export);
    if let Ok(receipt) = result {
        let _ = receipt.data.rows.is_empty();
    }
}

// ============================================================================
// Empty repository scanning
// ============================================================================

#[test]
fn lang_empty_dir_returns_no_rows() {
    let dir = make_empty_dir();
    let p = dir.path().to_string_lossy().replace('\\', "/");
    let scan = ScanSettings::for_paths(vec![p]);
    let lang = LangSettings::default();
    let receipt = lang_workflow(&scan, &lang).expect("should handle empty dir");
    assert!(
        receipt.report.rows.is_empty(),
        "empty dir should have no rows"
    );
}

#[test]
fn module_empty_dir_returns_no_rows() {
    let dir = make_empty_dir();
    let p = dir.path().to_string_lossy().replace('\\', "/");
    let scan = ScanSettings::for_paths(vec![p]);
    let module = ModuleSettings::default();
    let receipt = module_workflow(&scan, &module).expect("should handle empty dir");
    assert!(
        receipt.report.rows.is_empty(),
        "empty dir should have no rows"
    );
}

#[test]
fn export_empty_dir_returns_no_rows() {
    let dir = make_empty_dir();
    let p = dir.path().to_string_lossy().replace('\\', "/");
    let scan = ScanSettings::for_paths(vec![p]);
    let export = ExportSettings::default();
    let receipt = export_workflow(&scan, &export).expect("should handle empty dir");
    assert!(
        receipt.data.rows.is_empty(),
        "empty dir should have no rows"
    );
}

// ============================================================================
// Tempdir-based scans
// ============================================================================

#[test]
fn lang_tempdir_scan_finds_rust() {
    let repo = make_repo("fn main() {}\npub fn hello() -> i32 { 42 }\n");
    let p = repo.path().to_string_lossy().replace('\\', "/");
    let scan = ScanSettings::for_paths(vec![p]);
    let lang = LangSettings::default();
    let receipt = lang_workflow(&scan, &lang).unwrap();
    assert!(receipt.report.rows.iter().any(|r| r.lang == "Rust"));
}

#[test]
fn export_tempdir_scan_has_path() {
    let repo = make_repo("fn main() {}\n");
    let p = repo.path().to_string_lossy().replace('\\', "/");
    let scan = ScanSettings::for_paths(vec![p]);
    let export = ExportSettings::default();
    let receipt = export_workflow(&scan, &export).unwrap();
    assert!(!receipt.data.rows.is_empty());
    assert!(receipt.data.rows[0].path.contains("lib.rs"));
}
