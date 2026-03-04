//! Cross-crate integration tests for the scan → model → format pipeline.
//!
//! These tests exercise the full pipeline through `tokmd-core` workflow
//! functions and verify that crate boundaries preserve determinism,
//! correct sorting, and valid output.

use tokmd_core::{
    export_workflow, lang_workflow, module_workflow,
    settings::{ExportSettings, LangSettings, ModuleSettings, ScanSettings},
};
use tokmd_types::SCHEMA_VERSION;

// ============================================================================
// Helpers
// ============================================================================

/// Scan settings pointing at the tokmd-core `src/` directory.
fn src_scan() -> ScanSettings {
    ScanSettings::for_paths(vec!["src".to_string()])
}

// ============================================================================
// lang_workflow – determinism
// ============================================================================

#[test]
fn lang_workflow_deterministic_across_two_runs() {
    let scan = src_scan();
    let lang = LangSettings::default();

    let r1 = lang_workflow(&scan, &lang).expect("first run");
    let r2 = lang_workflow(&scan, &lang).expect("second run");

    // Row content must be identical (generated_at_ms may differ).
    assert_eq!(r1.report.rows.len(), r2.report.rows.len());
    for (a, b) in r1.report.rows.iter().zip(r2.report.rows.iter()) {
        assert_eq!(a.lang, b.lang, "languages must match across runs");
        assert_eq!(a.code, b.code, "code counts must match across runs");
    }
}

#[test]
fn lang_workflow_schema_version_matches_constant() {
    let receipt = lang_workflow(&src_scan(), &LangSettings::default()).unwrap();
    assert_eq!(receipt.schema_version, SCHEMA_VERSION);
}

#[test]
fn lang_workflow_mode_is_lang() {
    let receipt = lang_workflow(&src_scan(), &LangSettings::default()).unwrap();
    assert_eq!(receipt.mode, "lang");
}

#[test]
fn lang_workflow_finds_rust() {
    let receipt = lang_workflow(&src_scan(), &LangSettings::default()).unwrap();
    assert!(
        receipt.report.rows.iter().any(|r| r.lang == "Rust"),
        "should find Rust in tokmd-core/src"
    );
}

// ============================================================================
// module_workflow – sorted output
// ============================================================================

#[test]
fn module_workflow_rows_sorted_descending_by_code() {
    let scan = src_scan();
    let module = ModuleSettings::default();

    let receipt = module_workflow(&scan, &module).expect("module_workflow");

    // Rows should be sorted descending by code (the project convention).
    for pair in receipt.report.rows.windows(2) {
        assert!(
            pair[0].code >= pair[1].code,
            "rows must be sorted descending by code: {} ({}) vs {} ({})",
            pair[0].module,
            pair[0].code,
            pair[1].module,
            pair[1].code,
        );
    }
}

#[test]
fn module_workflow_schema_version_matches() {
    let receipt = module_workflow(&src_scan(), &ModuleSettings::default()).unwrap();
    assert_eq!(receipt.schema_version, SCHEMA_VERSION);
}

#[test]
fn module_workflow_mode_is_module() {
    let receipt = module_workflow(&src_scan(), &ModuleSettings::default()).unwrap();
    assert_eq!(receipt.mode, "module");
}

// ============================================================================
// export_workflow – valid JSON/CSV output
// ============================================================================

#[test]
fn export_workflow_produces_nonempty_rows() {
    let receipt = export_workflow(&src_scan(), &ExportSettings::default()).unwrap();
    assert!(!receipt.data.rows.is_empty(), "should produce file rows");
}

#[test]
fn export_workflow_json_round_trips() {
    let receipt = export_workflow(&src_scan(), &ExportSettings::default()).unwrap();

    // Serialize to JSON and parse back – verifies serde round-trip.
    let json = serde_json::to_string(&receipt).expect("serialize");
    let parsed: serde_json::Value = serde_json::from_str(&json).expect("parse");

    assert_eq!(parsed["mode"], "export");
    assert_eq!(
        parsed["schema_version"].as_u64().unwrap() as u32,
        SCHEMA_VERSION
    );
    assert!(parsed["data"]["rows"].is_array());
}

#[test]
fn export_workflow_rows_have_required_fields() {
    let receipt = export_workflow(&src_scan(), &ExportSettings::default()).unwrap();

    for row in &receipt.data.rows {
        assert!(!row.path.is_empty(), "path must not be empty");
        assert!(!row.lang.is_empty(), "lang must not be empty");
        assert!(row.lines > 0, "lines must be > 0 for real files");
    }
}

// ============================================================================
// Scan options flow-through
// ============================================================================

#[test]
fn scan_exclusion_filters_files() {
    let mut scan = src_scan();
    // Exclude all .rs files – should find nothing (or very little).
    scan.options.excluded = vec!["*.rs".to_string()];

    let receipt = lang_workflow(&scan, &LangSettings::default()).unwrap();

    // Should NOT find Rust since we excluded *.rs.
    let has_rust = receipt.report.rows.iter().any(|r| r.lang == "Rust");
    assert!(!has_rust, "Rust should be excluded by *.rs glob");
}

#[test]
fn top_setting_limits_lang_rows() {
    let lang = LangSettings {
        top: 1,
        ..Default::default()
    };
    let receipt = lang_workflow(&src_scan(), &lang).unwrap();

    // top=1 ⇒ at most 2 rows (1 real + possible "Other").
    assert!(
        receipt.report.rows.len() <= 2,
        "top=1 should cap rows, got {}",
        receipt.report.rows.len()
    );
}

// ============================================================================
// Error handling at pipeline boundaries
// ============================================================================

#[test]
fn lang_workflow_nonexistent_path_returns_error() {
    let scan = ScanSettings::for_paths(vec![
        "this_path_definitely_does_not_exist_xyz_42".to_string(),
    ]);
    let result = lang_workflow(&scan, &LangSettings::default());

    // The scan layer should propagate an error for a missing path.
    // tokei may return Ok with empty results rather than Err.
    // Either outcome is acceptable – but it must not panic.
    match result {
        Ok(receipt) => {
            // Empty results are valid: tokei silently skips missing paths.
            assert!(
                receipt.report.rows.is_empty(),
                "nonexistent path should yield no rows"
            );
        }
        Err(_) => {
            // Error propagation is also acceptable.
        }
    }
}

#[test]
fn export_workflow_nonexistent_path_handles_gracefully() {
    let scan = ScanSettings::for_paths(vec!["nonexistent_path_abc_99".to_string()]);
    let result = export_workflow(&scan, &ExportSettings::default());

    match result {
        Ok(receipt) => {
            assert!(
                receipt.data.rows.is_empty(),
                "nonexistent path should yield no rows"
            );
        }
        Err(_) => {}
    }
}

#[test]
fn module_workflow_nonexistent_path_handles_gracefully() {
    let scan = ScanSettings::for_paths(vec!["nonexistent_path_mod_77".to_string()]);
    let result = module_workflow(&scan, &ModuleSettings::default());

    match result {
        Ok(receipt) => {
            assert!(
                receipt.report.rows.is_empty(),
                "nonexistent path should yield no rows"
            );
        }
        Err(_) => {}
    }
}
