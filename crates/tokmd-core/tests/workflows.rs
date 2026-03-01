//! Integration tests for tokmd-core workflows.

#[cfg(feature = "analysis")]
use tokmd_core::{analyze_workflow, settings::AnalyzeSettings};
use tokmd_core::{
    export_workflow, lang_workflow, module_workflow,
    settings::{ExportSettings, LangSettings, ModuleSettings, ScanSettings},
};

// ============================================================================
// Lang workflow
// ============================================================================

#[test]
fn lang_workflow_scans_current_crate() {
    let scan = ScanSettings::for_paths(vec!["src".to_string()]);
    let lang = LangSettings::default();

    let receipt = lang_workflow(&scan, &lang).expect("lang_workflow should succeed");

    assert_eq!(receipt.mode, "lang");
    assert_eq!(receipt.schema_version, tokmd_types::SCHEMA_VERSION);
    assert!(
        !receipt.report.rows.is_empty(),
        "should find some languages"
    );
    // This crate is Rust, so we should find Rust
    assert!(
        receipt.report.rows.iter().any(|r| r.lang == "Rust"),
        "should find Rust in this crate"
    );
}

#[test]
fn lang_workflow_respects_top_setting() {
    let scan = ScanSettings::for_paths(vec!["src".to_string()]);
    let lang = LangSettings {
        top: 1,
        ..Default::default()
    };

    let receipt = lang_workflow(&scan, &lang).expect("lang_workflow should succeed");

    // Should have at most 2 rows (1 real + 1 "Other" if needed)
    assert!(receipt.report.rows.len() <= 2, "should respect top setting");
}

#[test]
fn lang_workflow_with_files_enabled() {
    let scan = ScanSettings::for_paths(vec!["src".to_string()]);
    let lang = LangSettings {
        files: true,
        ..Default::default()
    };

    let receipt = lang_workflow(&scan, &lang).expect("lang_workflow should succeed");
    assert!(receipt.args.with_files);
}

#[test]
fn lang_workflow_receipt_has_complete_status() {
    let scan = ScanSettings::for_paths(vec!["src".to_string()]);
    let lang = LangSettings::default();

    let receipt = lang_workflow(&scan, &lang).expect("lang_workflow should succeed");
    // ScanStatus doesn't implement PartialEq; compare via Debug
    assert_eq!(format!("{:?}", receipt.status), "Complete");
}

#[test]
fn lang_workflow_receipt_has_tool_info() {
    let scan = ScanSettings::for_paths(vec!["src".to_string()]);
    let lang = LangSettings::default();

    let receipt = lang_workflow(&scan, &lang).expect("lang_workflow should succeed");
    assert!(!receipt.tool.name.is_empty());
    assert!(!receipt.tool.version.is_empty());
}

#[test]
fn lang_workflow_receipt_has_valid_timestamp() {
    let scan = ScanSettings::for_paths(vec!["src".to_string()]);
    let lang = LangSettings::default();

    let receipt = lang_workflow(&scan, &lang).expect("lang_workflow should succeed");
    // generated_at_ms should be a timestamp after 2020-01-01
    assert!(receipt.generated_at_ms > 1_577_836_800_000);
}

#[test]
fn lang_workflow_receipt_has_scan_args() {
    let scan = ScanSettings::for_paths(vec!["src".to_string()]);
    let lang = LangSettings::default();

    let receipt = lang_workflow(&scan, &lang).expect("lang_workflow should succeed");
    assert!(
        !receipt.scan.paths.is_empty(),
        "scan.paths should not be empty"
    );
}

#[test]
fn lang_workflow_top_zero_returns_all_languages() {
    let scan = ScanSettings::for_paths(vec!["src".to_string()]);
    let lang = LangSettings {
        top: 0,
        ..Default::default()
    };

    let receipt = lang_workflow(&scan, &lang).expect("lang_workflow should succeed");
    // top=0 means no limit; all languages should be present
    assert!(
        !receipt.report.rows.is_empty(),
        "top=0 should return all languages"
    );
}

#[test]
fn lang_workflow_receipt_rows_have_code_lines() {
    let scan = ScanSettings::for_paths(vec!["src".to_string()]);
    let lang = LangSettings::default();

    let receipt = lang_workflow(&scan, &lang).expect("lang_workflow should succeed");
    // Every row should have non-negative code count
    for row in &receipt.report.rows {
        assert!(
            row.code > 0 || row.lang == "Other",
            "row should have code lines: {:?}",
            row.lang
        );
    }
}

#[test]
fn lang_workflow_receipt_is_serializable() {
    let scan = ScanSettings::for_paths(vec!["src".to_string()]);
    let lang = LangSettings::default();

    let receipt = lang_workflow(&scan, &lang).expect("lang_workflow should succeed");
    let json = serde_json::to_string(&receipt);
    assert!(json.is_ok(), "receipt should be JSON-serializable");

    // Should round-trip
    let json_str = json.expect("receipt should serialize to JSON");
    let deserialized: Result<tokmd_types::LangReceipt, _> = serde_json::from_str(&json_str);
    assert!(
        deserialized.is_ok(),
        "receipt should round-trip through JSON"
    );
}

// ============================================================================
// Module workflow
// ============================================================================

#[test]
fn module_workflow_scans_current_crate() {
    let scan = ScanSettings::for_paths(vec!["src".to_string()]);
    let module = ModuleSettings::default();

    let receipt = module_workflow(&scan, &module).expect("module_workflow should succeed");

    assert_eq!(receipt.mode, "module");
    assert_eq!(receipt.schema_version, tokmd_types::SCHEMA_VERSION);
    assert!(!receipt.report.rows.is_empty(), "should find some modules");
}

#[test]
fn module_workflow_with_custom_depth() {
    let scan = ScanSettings::for_paths(vec!["src".to_string()]);
    let module = ModuleSettings {
        module_depth: 1,
        ..Default::default()
    };

    let receipt = module_workflow(&scan, &module).expect("module_workflow should succeed");
    assert_eq!(receipt.args.module_depth, 1);
}

#[test]
fn module_workflow_with_custom_roots() {
    let scan = ScanSettings::for_paths(vec!["src".to_string()]);
    let module = ModuleSettings {
        module_roots: vec!["src".to_string()],
        ..Default::default()
    };

    let receipt = module_workflow(&scan, &module).expect("module_workflow should succeed");
    assert!(receipt.args.module_roots.contains(&"src".to_string()));
}

#[test]
fn module_workflow_receipt_has_complete_status() {
    let scan = ScanSettings::for_paths(vec!["src".to_string()]);
    let module = ModuleSettings::default();

    let receipt = module_workflow(&scan, &module).expect("module_workflow should succeed");
    assert_eq!(format!("{:?}", receipt.status), "Complete");
}

#[test]
fn module_workflow_receipt_is_serializable() {
    let scan = ScanSettings::for_paths(vec!["src".to_string()]);
    let module = ModuleSettings::default();

    let receipt = module_workflow(&scan, &module).expect("module_workflow should succeed");
    let json = serde_json::to_string(&receipt);
    assert!(json.is_ok(), "module receipt should be JSON-serializable");
}

#[test]
fn module_workflow_with_top_setting() {
    let scan = ScanSettings::for_paths(vec!["src".to_string()]);
    let module = ModuleSettings {
        top: 1,
        ..Default::default()
    };

    let receipt = module_workflow(&scan, &module).expect("module_workflow should succeed");
    assert_eq!(receipt.args.top, 1);
}

// ============================================================================
// Export workflow
// ============================================================================

#[test]
fn export_workflow_scans_current_crate() {
    let scan = ScanSettings::for_paths(vec!["src".to_string()]);
    let export = ExportSettings::default();

    let receipt = export_workflow(&scan, &export).expect("export_workflow should succeed");

    assert_eq!(receipt.mode, "export");
    assert_eq!(receipt.schema_version, tokmd_types::SCHEMA_VERSION);
    assert!(!receipt.data.rows.is_empty(), "should find some files");
}

#[test]
fn export_workflow_finds_rust_files() {
    let scan = ScanSettings::for_paths(vec!["src".to_string()]);
    let export = ExportSettings::default();

    let receipt = export_workflow(&scan, &export).expect("export_workflow should succeed");
    let has_rust = receipt.data.rows.iter().any(|r| r.lang == "Rust");
    assert!(has_rust, "should find Rust files in src/");
}

#[test]
fn export_workflow_file_rows_have_paths() {
    let scan = ScanSettings::for_paths(vec!["src".to_string()]);
    let export = ExportSettings::default();

    let receipt = export_workflow(&scan, &export).expect("export_workflow should succeed");
    for row in &receipt.data.rows {
        assert!(!row.path.is_empty(), "each file row should have a path");
    }
}

#[test]
fn export_workflow_paths_use_forward_slashes() {
    let scan = ScanSettings::for_paths(vec!["src".to_string()]);
    let export = ExportSettings::default();

    let receipt = export_workflow(&scan, &export).expect("export_workflow should succeed");
    for row in &receipt.data.rows {
        assert!(
            !row.path.contains('\\'),
            "paths should use forward slashes, got: {}",
            row.path
        );
    }
}

#[test]
fn export_workflow_with_min_code_filter() {
    let scan = ScanSettings::for_paths(vec!["src".to_string()]);

    let export_all = ExportSettings {
        min_code: 0,
        ..Default::default()
    };
    let receipt_all = export_workflow(&scan, &export_all).expect("export_workflow should succeed");

    let export_filtered = ExportSettings {
        min_code: 9999,
        ..Default::default()
    };
    let receipt_filtered = export_workflow(&scan, &export_filtered).expect("export_workflow should succeed");

    assert!(
        receipt_filtered.data.rows.len() <= receipt_all.data.rows.len(),
        "min_code filter should reduce rows"
    );
}

#[test]
fn export_workflow_with_max_rows() {
    let scan = ScanSettings::for_paths(vec!["src".to_string()]);
    let export = ExportSettings {
        max_rows: 1,
        ..Default::default()
    };

    let receipt = export_workflow(&scan, &export).expect("export_workflow should succeed");
    assert!(
        receipt.data.rows.len() <= 1,
        "max_rows=1 should limit output, got {}",
        receipt.data.rows.len()
    );
}

#[test]
fn export_workflow_receipt_is_serializable() {
    let scan = ScanSettings::for_paths(vec!["src".to_string()]);
    let export = ExportSettings::default();

    let receipt = export_workflow(&scan, &export).expect("export_workflow should succeed");
    let json = serde_json::to_string(&receipt);
    assert!(json.is_ok(), "export receipt should be JSON-serializable");
}

#[test]
fn export_workflow_receipt_has_complete_status() {
    let scan = ScanSettings::for_paths(vec!["src".to_string()]);
    let export = ExportSettings::default();

    let receipt = export_workflow(&scan, &export).expect("export_workflow should succeed");
    assert_eq!(format!("{:?}", receipt.status), "Complete");
}

// ============================================================================
// ScanSettings
// ============================================================================

#[test]
fn scan_settings_excluded_patterns() {
    let scan = ScanSettings {
        paths: vec!["src".to_string()],
        options: tokmd_core::settings::ScanOptions {
            excluded: vec!["**/tests/**".to_string()],
            ..Default::default()
        },
    };
    let lang = LangSettings::default();

    let receipt = lang_workflow(&scan, &lang).expect("lang_workflow should succeed");
    assert_eq!(receipt.mode, "lang");
}

#[test]
fn scan_settings_current_dir() {
    let settings = ScanSettings::current_dir();
    assert_eq!(settings.paths, vec!["."]);
}

#[test]
fn scan_settings_for_multiple_paths() {
    let settings = ScanSettings::for_paths(vec!["src".to_string(), "tests".to_string()]);
    assert_eq!(settings.paths.len(), 2);
}

// ============================================================================
// Version helper
// ============================================================================

#[test]
fn version_not_empty() {
    let v = tokmd_core::version();
    assert!(!v.is_empty());
    assert!(v.contains('.'), "version should look like semver");
}

#[test]
fn core_schema_version_matches() {
    assert_eq!(tokmd_core::CORE_SCHEMA_VERSION, tokmd_types::SCHEMA_VERSION);
}

// ============================================================================
// Diff workflow
// ============================================================================

#[test]
fn diff_workflow_same_directory() {
    use tokmd_core::settings::DiffSettings;

    let settings = DiffSettings {
        from: "src".to_string(),
        to: "src".to_string(),
    };

    let receipt = tokmd_core::diff_workflow(&settings).expect("diff_workflow should succeed");
    // Self-diff should have zero-delta rows
    for row in &receipt.diff_rows {
        assert_eq!(
            row.delta_code, 0,
            "self-diff should have zero delta_code for {}",
            row.lang
        );
    }
}

#[test]
fn diff_workflow_receipt_has_totals() {
    use tokmd_core::settings::DiffSettings;

    let settings = DiffSettings {
        from: "src".to_string(),
        to: "src".to_string(),
    };

    let receipt = tokmd_core::diff_workflow(&settings).expect("diff_workflow should succeed");
    // Totals should exist
    assert_eq!(receipt.totals.delta_code, 0);
}

// ============================================================================
// Analyze workflow (feature-gated)
// ============================================================================

#[test]
#[cfg(feature = "analysis")]
fn analyze_workflow_runs_with_receipt_preset() {
    let scan = ScanSettings::for_paths(vec!["src".to_string()]);
    let analyze = AnalyzeSettings::default();

    let receipt = analyze_workflow(&scan, &analyze).expect("analyze_workflow should succeed");

    assert_eq!(receipt.mode, "analysis");
    assert!(
        receipt.derived.is_some(),
        "receipt preset should include derived metrics"
    );
}

// ============================================================================
// Property tests
// ============================================================================

mod properties {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn lang_workflow_top_never_exceeds_limit(top in 1usize..20) {
            let scan = ScanSettings::for_paths(vec!["src".to_string()]);
            let lang = LangSettings {
                top,
                ..Default::default()
            };

            let receipt = lang_workflow(&scan, &lang).expect("lang_workflow should succeed");
            // rows should be at most top + 1 (for possible "Other" row)
            prop_assert!(
                receipt.report.rows.len() <= top + 1,
                "top={} but got {} rows",
                top,
                receipt.report.rows.len()
            );
        }

        #[test]
        fn export_max_rows_respected(max_rows in 1usize..10) {
            let scan = ScanSettings::for_paths(vec!["src".to_string()]);
            let export = ExportSettings {
                max_rows,
                ..Default::default()
            };

            let receipt = export_workflow(&scan, &export).expect("export_workflow should succeed");
            prop_assert!(
                receipt.data.rows.len() <= max_rows,
                "max_rows={} but got {} rows",
                max_rows,
                receipt.data.rows.len()
            );
        }

        #[test]
        fn lang_workflow_always_produces_complete_status(
            top in 0usize..5,
            files in proptest::bool::ANY,
        ) {
            let scan = ScanSettings::for_paths(vec!["src".to_string()]);
            let lang = LangSettings {
                top,
                files,
                ..Default::default()
            };

            let receipt = lang_workflow(&scan, &lang).expect("lang_workflow should succeed");
            prop_assert_eq!(format!("{:?}", receipt.status), "Complete");
            prop_assert_eq!(receipt.mode.as_str(), "lang");
            prop_assert_eq!(receipt.schema_version, tokmd_types::SCHEMA_VERSION);
        }
    }
}
