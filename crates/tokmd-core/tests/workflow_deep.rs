//! Deep workflow integration tests using temporary fixture directories.
//!
//! These tests exercise `lang_workflow`, `module_workflow`, `export_workflow`,
//! and `diff_workflow` with isolated temp dirs containing known source files,
//! covering edge cases like empty directories, error handling, redaction,
//! children modes, and cross-workflow consistency.

use std::fs;
use tempfile::TempDir;
use tokmd_core::settings::{
    DiffSettings, ExportSettings, LangSettings, ModuleSettings, ScanSettings,
};
use tokmd_core::{export_workflow, lang_workflow, module_workflow};
use tokmd_types::RedactMode;

// ============================================================================
// Fixtures
// ============================================================================

/// Create a temp dir with a single Rust source file.
fn fixture_rust() -> TempDir {
    let dir = TempDir::new().expect("create tempdir");
    fs::write(
        dir.path().join("main.rs"),
        "fn main() {\n    println!(\"hello\");\n}\n",
    )
    .unwrap();
    dir
}

/// Create a temp dir with multiple language files.
fn fixture_multi_lang() -> TempDir {
    let dir = TempDir::new().expect("create tempdir");
    fs::write(
        dir.path().join("app.rs"),
        "fn main() {\n    let x = 1;\n    let y = 2;\n}\n",
    )
    .unwrap();
    fs::write(
        dir.path().join("lib.py"),
        "def hello():\n    return 42\n\ndef world():\n    return 0\n",
    )
    .unwrap();
    dir
}

/// Create a temp dir with nested subdirectories.
fn fixture_nested() -> TempDir {
    let dir = TempDir::new().expect("create tempdir");
    let sub = dir.path().join("src").join("util");
    fs::create_dir_all(&sub).unwrap();
    fs::write(
        dir.path().join("src").join("lib.rs"),
        "pub mod util;\n\npub fn init() {}\n",
    )
    .unwrap();
    fs::write(
        sub.join("helpers.rs"),
        "pub fn add(a: i32, b: i32) -> i32 {\n    a + b\n}\n",
    )
    .unwrap();
    dir
}

/// Create an empty temp dir.
fn fixture_empty() -> TempDir {
    TempDir::new().expect("create tempdir")
}

fn scan_for(dir: &TempDir) -> ScanSettings {
    ScanSettings::for_paths(vec![dir.path().display().to_string()])
}

// ============================================================================
// lang_workflow with fixtures
// ============================================================================

#[test]
fn lang_fixture_rust_detects_language() {
    let dir = fixture_rust();
    let receipt = lang_workflow(&scan_for(&dir), &LangSettings::default()).unwrap();

    assert_eq!(receipt.mode, "lang");
    assert_eq!(receipt.schema_version, tokmd_types::SCHEMA_VERSION);
    assert!(!receipt.report.rows.is_empty());
    assert!(receipt.report.rows.iter().any(|r| r.lang == "Rust"));
}

#[test]
fn lang_fixture_multi_lang_detects_all() {
    let dir = fixture_multi_lang();
    let receipt = lang_workflow(&scan_for(&dir), &LangSettings::default()).unwrap();

    let langs: Vec<&str> = receipt
        .report
        .rows
        .iter()
        .map(|r| r.lang.as_str())
        .collect();
    assert!(langs.contains(&"Rust"), "should find Rust");
    assert!(langs.contains(&"Python"), "should find Python");
}

#[test]
fn lang_fixture_empty_dir_returns_empty_rows() {
    let dir = fixture_empty();
    let receipt = lang_workflow(&scan_for(&dir), &LangSettings::default()).unwrap();

    assert!(
        receipt.report.rows.is_empty(),
        "empty dir should produce no language rows"
    );
    assert_eq!(receipt.mode, "lang");
    assert_eq!(format!("{:?}", receipt.status), "Complete");
}

#[test]
fn lang_fixture_top_limits_rows() {
    let dir = fixture_multi_lang();
    let lang = LangSettings {
        top: 1,
        ..Default::default()
    };
    let receipt = lang_workflow(&scan_for(&dir), &lang).unwrap();

    // top=1 → at most 2 rows (1 real + optional "Other")
    assert!(
        receipt.report.rows.len() <= 2,
        "top=1 should limit, got {} rows",
        receipt.report.rows.len()
    );
}

#[test]
fn lang_fixture_top_zero_returns_all() {
    let dir = fixture_multi_lang();
    let lang = LangSettings {
        top: 0,
        ..Default::default()
    };
    let receipt = lang_workflow(&scan_for(&dir), &lang).unwrap();

    assert!(
        receipt.report.rows.len() >= 2,
        "top=0 should return all languages"
    );
}

#[test]
fn lang_fixture_receipt_has_valid_timestamp() {
    let dir = fixture_rust();
    let receipt = lang_workflow(&scan_for(&dir), &LangSettings::default()).unwrap();
    assert!(receipt.generated_at_ms > 1_577_836_800_000);
}

#[test]
fn lang_fixture_receipt_has_tool_info() {
    let dir = fixture_rust();
    let receipt = lang_workflow(&scan_for(&dir), &LangSettings::default()).unwrap();
    assert!(!receipt.tool.name.is_empty());
    assert!(!receipt.tool.version.is_empty());
}

#[test]
fn lang_fixture_serializable_roundtrip() {
    let dir = fixture_rust();
    let receipt = lang_workflow(&scan_for(&dir), &LangSettings::default()).unwrap();

    let json = serde_json::to_string(&receipt).unwrap();
    let back: tokmd_types::LangReceipt = serde_json::from_str(&json).unwrap();
    assert_eq!(receipt.schema_version, back.schema_version);
    assert_eq!(receipt.mode, back.mode);
    assert_eq!(receipt.report.rows.len(), back.report.rows.len());
}

#[test]
fn lang_fixture_deterministic() {
    let dir = fixture_rust();
    let scan = scan_for(&dir);
    let lang = LangSettings::default();

    let r1 = lang_workflow(&scan, &lang).unwrap();
    let r2 = lang_workflow(&scan, &lang).unwrap();

    assert_eq!(r1.report.rows.len(), r2.report.rows.len());
    for (a, b) in r1.report.rows.iter().zip(r2.report.rows.iter()) {
        assert_eq!(a.lang, b.lang);
        assert_eq!(a.code, b.code);
        assert_eq!(a.lines, b.lines);
    }
}

#[test]
fn lang_fixture_children_collapse() {
    let dir = fixture_rust();
    let lang = LangSettings {
        children: tokmd_types::ChildrenMode::Collapse,
        ..Default::default()
    };
    let receipt = lang_workflow(&scan_for(&dir), &lang).unwrap();
    assert_eq!(receipt.mode, "lang");
}

#[test]
fn lang_fixture_children_separate() {
    let dir = fixture_rust();
    let lang = LangSettings {
        children: tokmd_types::ChildrenMode::Separate,
        ..Default::default()
    };
    let receipt = lang_workflow(&scan_for(&dir), &lang).unwrap();
    assert_eq!(receipt.mode, "lang");
}

#[test]
fn lang_fixture_with_files_enabled() {
    let dir = fixture_rust();
    let lang = LangSettings {
        files: true,
        ..Default::default()
    };
    let receipt = lang_workflow(&scan_for(&dir), &lang).unwrap();
    assert!(receipt.args.with_files);
}

// ============================================================================
// module_workflow with fixtures
// ============================================================================

#[test]
fn module_fixture_basic() {
    let dir = fixture_rust();
    let receipt = module_workflow(&scan_for(&dir), &ModuleSettings::default()).unwrap();

    assert_eq!(receipt.mode, "module");
    assert_eq!(receipt.schema_version, tokmd_types::SCHEMA_VERSION);
}

#[test]
fn module_fixture_empty_dir() {
    let dir = fixture_empty();
    let receipt = module_workflow(&scan_for(&dir), &ModuleSettings::default()).unwrap();

    assert!(
        receipt.report.rows.is_empty(),
        "empty dir should produce no module rows"
    );
    assert_eq!(format!("{:?}", receipt.status), "Complete");
}

#[test]
fn module_fixture_nested_detects_modules() {
    let dir = fixture_nested();
    let receipt = module_workflow(&scan_for(&dir), &ModuleSettings::default()).unwrap();

    assert!(
        !receipt.report.rows.is_empty(),
        "nested fixture should produce module rows"
    );
}

#[test]
fn module_fixture_custom_depth() {
    let dir = fixture_nested();
    let module = ModuleSettings {
        module_depth: 1,
        ..Default::default()
    };
    let receipt = module_workflow(&scan_for(&dir), &module).unwrap();
    assert_eq!(receipt.args.module_depth, 1);
}

#[test]
fn module_fixture_serializable_roundtrip() {
    let dir = fixture_rust();
    let receipt = module_workflow(&scan_for(&dir), &ModuleSettings::default()).unwrap();

    let json = serde_json::to_string(&receipt).unwrap();
    let back: tokmd_types::ModuleReceipt = serde_json::from_str(&json).unwrap();
    assert_eq!(receipt.schema_version, back.schema_version);
    assert_eq!(receipt.mode, back.mode);
}

#[test]
fn module_fixture_deterministic() {
    let dir = fixture_rust();
    let scan = scan_for(&dir);
    let module = ModuleSettings::default();

    let r1 = module_workflow(&scan, &module).unwrap();
    let r2 = module_workflow(&scan, &module).unwrap();

    assert_eq!(r1.report.rows.len(), r2.report.rows.len());
}

// ============================================================================
// export_workflow with fixtures
// ============================================================================

#[test]
fn export_fixture_basic() {
    let dir = fixture_rust();
    let receipt = export_workflow(&scan_for(&dir), &ExportSettings::default()).unwrap();

    assert_eq!(receipt.mode, "export");
    assert_eq!(receipt.schema_version, tokmd_types::SCHEMA_VERSION);
    assert!(!receipt.data.rows.is_empty());
}

#[test]
fn export_fixture_empty_dir() {
    let dir = fixture_empty();
    let receipt = export_workflow(&scan_for(&dir), &ExportSettings::default()).unwrap();

    assert!(
        receipt.data.rows.is_empty(),
        "empty dir should produce no export rows"
    );
    assert_eq!(format!("{:?}", receipt.status), "Complete");
}

#[test]
fn export_fixture_paths_forward_slashes() {
    let dir = fixture_rust();
    let receipt = export_workflow(&scan_for(&dir), &ExportSettings::default()).unwrap();

    for row in &receipt.data.rows {
        assert!(
            !row.path.contains('\\'),
            "path must use forward slashes: {}",
            row.path
        );
    }
}

#[test]
fn export_fixture_file_has_correct_lang() {
    let dir = fixture_rust();
    let receipt = export_workflow(&scan_for(&dir), &ExportSettings::default()).unwrap();

    let has_rust = receipt.data.rows.iter().any(|r| r.lang == "Rust");
    assert!(has_rust, "should detect Rust file");
}

#[test]
fn export_fixture_multi_lang_files() {
    let dir = fixture_multi_lang();
    let receipt = export_workflow(&scan_for(&dir), &ExportSettings::default()).unwrap();

    let langs: Vec<&str> = receipt.data.rows.iter().map(|r| r.lang.as_str()).collect();
    assert!(langs.contains(&"Rust"), "should find Rust file");
    assert!(langs.contains(&"Python"), "should find Python file");
}

#[test]
fn export_fixture_min_code_filter() {
    let dir = fixture_rust();

    let all = export_workflow(
        &scan_for(&dir),
        &ExportSettings {
            min_code: 0,
            ..Default::default()
        },
    )
    .unwrap();

    let filtered = export_workflow(
        &scan_for(&dir),
        &ExportSettings {
            min_code: 999_999,
            ..Default::default()
        },
    )
    .unwrap();

    assert!(!all.data.rows.is_empty());
    assert!(
        filtered.data.rows.is_empty(),
        "high min_code should filter all"
    );
}

#[test]
fn export_fixture_max_rows() {
    let dir = fixture_multi_lang();
    let export = ExportSettings {
        max_rows: 1,
        ..Default::default()
    };
    let receipt = export_workflow(&scan_for(&dir), &export).unwrap();
    assert!(
        receipt.data.rows.len() <= 1,
        "max_rows=1 should limit output"
    );
}

#[test]
fn export_fixture_redact_paths() {
    let dir = fixture_rust();
    let export = ExportSettings {
        redact: RedactMode::Paths,
        ..Default::default()
    };
    let receipt = export_workflow(&scan_for(&dir), &export).unwrap();

    for row in &receipt.data.rows {
        assert!(
            !row.path.contains("main.rs"),
            "redacted path should not contain original filename: {}",
            row.path
        );
    }
}

#[test]
fn export_fixture_redact_all() {
    let dir = fixture_rust();
    let export = ExportSettings {
        redact: RedactMode::All,
        ..Default::default()
    };
    let receipt = export_workflow(&scan_for(&dir), &export).unwrap();
    assert!(!receipt.data.rows.is_empty());
}

#[test]
fn export_fixture_redact_none_preserves_paths() {
    let dir = fixture_rust();
    let export = ExportSettings {
        redact: RedactMode::None,
        ..Default::default()
    };
    let receipt = export_workflow(&scan_for(&dir), &export).unwrap();

    let has_main = receipt.data.rows.iter().any(|r| r.path.contains("main.rs"));
    assert!(has_main, "unredacted paths should preserve filenames");
}

#[test]
fn export_fixture_serializable_roundtrip() {
    let dir = fixture_rust();
    let receipt = export_workflow(&scan_for(&dir), &ExportSettings::default()).unwrap();

    let json = serde_json::to_string(&receipt).unwrap();
    let back: tokmd_types::ExportReceipt = serde_json::from_str(&json).unwrap();
    assert_eq!(receipt.schema_version, back.schema_version);
    assert_eq!(receipt.data.rows.len(), back.data.rows.len());
}

#[test]
fn export_fixture_deterministic() {
    let dir = fixture_multi_lang();
    let scan = scan_for(&dir);
    let export = ExportSettings::default();

    let r1 = export_workflow(&scan, &export).unwrap();
    let r2 = export_workflow(&scan, &export).unwrap();

    assert_eq!(r1.data.rows.len(), r2.data.rows.len());
    for (a, b) in r1.data.rows.iter().zip(r2.data.rows.iter()) {
        assert_eq!(a.path, b.path);
        assert_eq!(a.lang, b.lang);
        assert_eq!(a.code, b.code);
    }
}

// ============================================================================
// diff_workflow with fixtures
// ============================================================================

#[test]
fn diff_fixture_self_diff_zero_deltas() {
    let dir = fixture_rust();
    let p = dir.path().display().to_string();
    let settings = DiffSettings {
        from: p.clone(),
        to: p,
    };
    let receipt = tokmd_core::diff_workflow(&settings).unwrap();

    for row in &receipt.diff_rows {
        assert_eq!(
            row.delta_code, 0,
            "self-diff delta_code should be 0 for {}",
            row.lang
        );
    }
    assert_eq!(receipt.totals.delta_code, 0);
}

#[test]
fn diff_fixture_different_dirs() {
    let dir_a = fixture_rust();
    let dir_b = fixture_multi_lang();
    let settings = DiffSettings {
        from: dir_a.path().display().to_string(),
        to: dir_b.path().display().to_string(),
    };
    let receipt = tokmd_core::diff_workflow(&settings).unwrap();

    assert_eq!(receipt.mode, "diff");
    assert_eq!(receipt.schema_version, tokmd_types::SCHEMA_VERSION);
    // Should detect added Python
    let has_python = receipt.diff_rows.iter().any(|r| r.lang == "Python");
    assert!(has_python, "diff should show added Python language");
}

#[test]
fn diff_fixture_empty_to_populated() {
    let empty = fixture_empty();
    let populated = fixture_rust();
    let settings = DiffSettings {
        from: empty.path().display().to_string(),
        to: populated.path().display().to_string(),
    };
    let receipt = tokmd_core::diff_workflow(&settings).unwrap();

    // All delta_code should be positive (added code)
    assert!(
        receipt.totals.delta_code > 0,
        "empty→populated diff should have positive delta"
    );
}

#[test]
fn diff_fixture_populated_to_empty() {
    let populated = fixture_rust();
    let empty = fixture_empty();
    let settings = DiffSettings {
        from: populated.path().display().to_string(),
        to: empty.path().display().to_string(),
    };
    let receipt = tokmd_core::diff_workflow(&settings).unwrap();

    // All delta_code should be negative (removed code)
    assert!(
        receipt.totals.delta_code < 0,
        "populated→empty diff should have negative delta"
    );
}

// ============================================================================
// Non-existent directory error handling
// ============================================================================

#[test]
fn lang_nonexistent_dir_is_error() {
    let scan = ScanSettings::for_paths(vec!["/tmp/__tokmd_nonexistent_42__".to_string()]);
    let result = lang_workflow(&scan, &LangSettings::default());
    // Should either error or return empty results; either is acceptable
    match result {
        Ok(receipt) => assert!(receipt.report.rows.is_empty()),
        Err(_) => {} // error is also acceptable
    }
}

#[test]
fn module_nonexistent_dir_is_error() {
    let scan = ScanSettings::for_paths(vec!["/tmp/__tokmd_nonexistent_42__".to_string()]);
    let result = module_workflow(&scan, &ModuleSettings::default());
    match result {
        Ok(receipt) => assert!(receipt.report.rows.is_empty()),
        Err(_) => {}
    }
}

#[test]
fn export_nonexistent_dir_is_error() {
    let scan = ScanSettings::for_paths(vec!["/tmp/__tokmd_nonexistent_42__".to_string()]);
    let result = export_workflow(&scan, &ExportSettings::default());
    match result {
        Ok(receipt) => assert!(receipt.data.rows.is_empty()),
        Err(_) => {}
    }
}

// ============================================================================
// Cross-workflow consistency
// ============================================================================

#[test]
fn cross_workflow_lang_export_code_totals_match() {
    let dir = fixture_multi_lang();
    let scan = scan_for(&dir);

    let lang_receipt = lang_workflow(&scan, &LangSettings::default()).unwrap();
    let export_receipt = export_workflow(&scan, &ExportSettings::default()).unwrap();

    let lang_total: usize = lang_receipt.report.rows.iter().map(|r| r.code).sum();
    let export_total: usize = export_receipt.data.rows.iter().map(|r| r.code).sum();

    assert_eq!(
        lang_total, export_total,
        "lang total code ({lang_total}) should match export total ({export_total})"
    );
}

#[test]
fn cross_workflow_all_modes_same_schema_version() {
    let dir = fixture_rust();
    let scan = scan_for(&dir);

    let lang = lang_workflow(&scan, &LangSettings::default()).unwrap();
    let module = module_workflow(&scan, &ModuleSettings::default()).unwrap();
    let export = export_workflow(&scan, &ExportSettings::default()).unwrap();

    assert_eq!(lang.schema_version, module.schema_version);
    assert_eq!(lang.schema_version, export.schema_version);
    assert_eq!(lang.schema_version, tokmd_types::SCHEMA_VERSION);
}

#[test]
fn cross_workflow_all_modes_complete_status() {
    let dir = fixture_rust();
    let scan = scan_for(&dir);

    let lang = lang_workflow(&scan, &LangSettings::default()).unwrap();
    let module = module_workflow(&scan, &ModuleSettings::default()).unwrap();
    let export = export_workflow(&scan, &ExportSettings::default()).unwrap();

    assert_eq!(format!("{:?}", lang.status), "Complete");
    assert_eq!(format!("{:?}", module.status), "Complete");
    assert_eq!(format!("{:?}", export.status), "Complete");
}
