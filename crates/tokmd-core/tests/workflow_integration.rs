//! Deep workflow integration tests for tokmd-core.
//!
//! These tests complement `workflows.rs` by focusing on:
//! - Structural invariants (sorting, ordering, path normalization)
//! - Cross-workflow consistency
//! - Redaction behavior
//! - Full-repo scanning

use tokmd_core::{
    export_workflow, lang_workflow, module_workflow,
    settings::{ExportSettings, LangSettings, ModuleSettings, ScanSettings},
};
use tokmd_types::RedactMode;

// ============================================================================
// BTreeMap ordering invariants
// ============================================================================

/// Lang rows must be sorted by code descending, then by name ascending.
#[test]
fn lang_rows_sorted_by_code_descending() {
    let scan = ScanSettings::for_paths(vec!["src".to_string()]);
    let lang = LangSettings::default();
    let receipt = lang_workflow(&scan, &lang).unwrap();

    let rows = &receipt.report.rows;
    for window in rows.windows(2) {
        assert!(
            window[0].code >= window[1].code,
            "rows should be sorted by code desc: {} ({}) >= {} ({})",
            window[0].lang,
            window[0].code,
            window[1].lang,
            window[1].code,
        );
    }
}

/// Module rows must be sorted by code descending.
#[test]
fn module_rows_sorted_by_code_descending() {
    let scan = ScanSettings::for_paths(vec!["src".to_string()]);
    let module = ModuleSettings::default();
    let receipt = module_workflow(&scan, &module).unwrap();

    let rows = &receipt.report.rows;
    for window in rows.windows(2) {
        assert!(
            window[0].code >= window[1].code,
            "rows should be sorted by code desc: {} ({}) >= {} ({})",
            window[0].module,
            window[0].code,
            window[1].module,
            window[1].code,
        );
    }
}

/// Export rows must be sorted by code descending.
#[test]
fn export_rows_sorted_by_code_descending() {
    let scan = ScanSettings::for_paths(vec!["src".to_string()]);
    let export = ExportSettings::default();
    let receipt = export_workflow(&scan, &export).unwrap();

    let rows = &receipt.data.rows;
    for window in rows.windows(2) {
        assert!(
            window[0].code >= window[1].code,
            "rows should be sorted by code desc: {} ({}) >= {} ({})",
            window[0].path,
            window[0].code,
            window[1].path,
            window[1].code,
        );
    }
}

// ============================================================================
// Path normalization
// ============================================================================

/// All module keys should use forward slashes only.
#[test]
fn module_rows_use_forward_slash_separators() {
    let scan = ScanSettings::for_paths(vec!["src".to_string()]);
    let module = ModuleSettings::default();
    let receipt = module_workflow(&scan, &module).unwrap();

    for row in &receipt.report.rows {
        assert!(
            !row.module.contains('\\'),
            "module key should use forward slashes: {}",
            row.module
        );
    }
}

/// All export paths should use forward slashes (tested on all rows).
#[test]
fn export_all_paths_normalized() {
    let scan = ScanSettings::for_paths(vec!["src".to_string()]);
    let export = ExportSettings::default();
    let receipt = export_workflow(&scan, &export).unwrap();

    for row in &receipt.data.rows {
        assert!(
            !row.path.contains('\\'),
            "path should be normalized: {}",
            row.path
        );
        assert!(
            !row.module.contains('\\'),
            "module should be normalized: {}",
            row.module
        );
    }
}

// ============================================================================
// Cross-workflow consistency
// ============================================================================

/// Both lang and export workflows find Rust in src/.
#[test]
fn lang_and_export_agree_on_rust_present() {
    let scan = ScanSettings::for_paths(vec!["src".to_string()]);

    let lang_receipt = lang_workflow(&scan, &LangSettings::default()).unwrap();
    let export_receipt = export_workflow(&scan, &ExportSettings::default()).unwrap();

    let lang_has_rust = lang_receipt.report.rows.iter().any(|r| r.lang == "Rust");
    let export_has_rust = export_receipt.data.rows.iter().any(|r| r.lang == "Rust");

    assert!(lang_has_rust, "lang should find Rust");
    assert!(export_has_rust, "export should find Rust");
}

/// Both lang and export workflows produce non-empty results from the same scan.
#[test]
fn lang_and_export_both_find_results() {
    let scan = ScanSettings::for_paths(vec!["src".to_string()]);

    let lang_receipt = lang_workflow(&scan, &LangSettings::default()).unwrap();
    let export_receipt = export_workflow(&scan, &ExportSettings::default()).unwrap();

    assert!(
        !lang_receipt.report.rows.is_empty(),
        "lang should find languages"
    );
    assert!(
        !export_receipt.data.rows.is_empty(),
        "export should find files"
    );

    // Lang total code should be positive
    assert!(lang_receipt.report.total.code > 0);
    // Export sum should be positive
    let export_sum: usize = export_receipt.data.rows.iter().map(|r| r.code).sum();
    assert!(export_sum > 0);
}

// ============================================================================
// Redaction
// ============================================================================

/// Export with RedactMode::Paths produces different paths than without.
#[test]
fn export_redact_paths_changes_output() {
    let scan = ScanSettings::for_paths(vec!["src".to_string()]);

    let plain = export_workflow(&scan, &ExportSettings::default()).unwrap();
    let redacted = export_workflow(
        &scan,
        &ExportSettings {
            redact: RedactMode::Paths,
            ..Default::default()
        },
    )
    .unwrap();

    assert_eq!(
        plain.data.rows.len(),
        redacted.data.rows.len(),
        "redaction should not change row count"
    );

    // At least one path should differ (unless the crate has no files, which it does)
    if !plain.data.rows.is_empty() {
        let any_different = plain
            .data
            .rows
            .iter()
            .zip(redacted.data.rows.iter())
            .any(|(p, r)| p.path != r.path);
        assert!(
            any_different,
            "redacted paths should differ from plain paths"
        );
    }
}

/// Export with RedactMode::All also redacts module names.
#[test]
fn export_redact_all_changes_modules() {
    let scan = ScanSettings::for_paths(vec!["src".to_string()]);

    let plain = export_workflow(&scan, &ExportSettings::default()).unwrap();
    let redacted = export_workflow(
        &scan,
        &ExportSettings {
            redact: RedactMode::All,
            ..Default::default()
        },
    )
    .unwrap();

    if !plain.data.rows.is_empty() {
        let any_module_different = plain
            .data
            .rows
            .iter()
            .zip(redacted.data.rows.iter())
            .any(|(p, r)| p.module != r.module);
        assert!(
            any_module_different,
            "RedactMode::All should change module names"
        );
    }
}

// ============================================================================
// Diff workflow structural invariants
// ============================================================================

/// Diff receipt has correct from_source and to_source.
#[test]
fn diff_receipt_records_sources() {
    let settings = tokmd_core::settings::DiffSettings {
        from: "src".to_string(),
        to: "src".to_string(),
    };

    let receipt = tokmd_core::diff_workflow(&settings).unwrap();
    assert_eq!(receipt.from_source, "src");
    assert_eq!(receipt.to_source, "src");
}

/// Diff receipt has valid schema version and mode.
#[test]
fn diff_receipt_has_standard_metadata() {
    let settings = tokmd_core::settings::DiffSettings {
        from: "src".to_string(),
        to: "src".to_string(),
    };

    let receipt = tokmd_core::diff_workflow(&settings).unwrap();
    assert_eq!(receipt.schema_version, tokmd_types::SCHEMA_VERSION);
    assert_eq!(receipt.mode, "diff");
    assert!(receipt.generated_at_ms > 1_577_836_800_000);
    assert!(!receipt.tool.name.is_empty());
    assert!(!receipt.tool.version.is_empty());
}

/// Diff receipt totals match row sums.
#[test]
fn diff_totals_match_row_sums() {
    let settings = tokmd_core::settings::DiffSettings {
        from: "src".to_string(),
        to: "src".to_string(),
    };

    let receipt = tokmd_core::diff_workflow(&settings).unwrap();

    let sum_old_code: usize = receipt.diff_rows.iter().map(|r| r.old_code).sum();
    let sum_new_code: usize = receipt.diff_rows.iter().map(|r| r.new_code).sum();
    let sum_delta: i64 = receipt.diff_rows.iter().map(|r| r.delta_code).sum();

    assert_eq!(receipt.totals.old_code, sum_old_code);
    assert_eq!(receipt.totals.new_code, sum_new_code);
    assert_eq!(receipt.totals.delta_code, sum_delta);
}

/// Diff receipt is JSON-serializable and round-trippable.
#[test]
fn diff_receipt_roundtrips_through_json() {
    let settings = tokmd_core::settings::DiffSettings {
        from: "src".to_string(),
        to: "src".to_string(),
    };

    let receipt = tokmd_core::diff_workflow(&settings).unwrap();
    let json = serde_json::to_string(&receipt).expect("should serialize");
    let deserialized: tokmd_types::DiffReceipt =
        serde_json::from_str(&json).expect("should deserialize");
    assert_eq!(deserialized.mode, "diff");
    assert_eq!(deserialized.schema_version, receipt.schema_version);
    assert_eq!(deserialized.diff_rows.len(), receipt.diff_rows.len());
}

// ============================================================================
// Structural invariants across workflows
// ============================================================================

/// Every export row has non-negative code and lines.
#[test]
fn export_rows_have_non_negative_counts() {
    let scan = ScanSettings::for_paths(vec!["src".to_string()]);
    let receipt = export_workflow(&scan, &ExportSettings::default()).unwrap();

    for row in &receipt.data.rows {
        // usize is inherently non-negative, but verify via JSON round-trip
        let json = serde_json::to_value(row).unwrap();
        assert!(
            json["code"].as_u64().is_some(),
            "code should serialize as u64"
        );
        assert!(
            json["lines"].as_u64().is_some(),
            "lines should serialize as u64"
        );
    }
}

/// Every lang row has lines >= code (code is a subset of lines).
#[test]
fn lang_rows_lines_ge_code() {
    let scan = ScanSettings::for_paths(vec!["src".to_string()]);
    let receipt = lang_workflow(&scan, &LangSettings::default()).unwrap();

    for row in &receipt.report.rows {
        assert!(
            row.lines >= row.code,
            "{}: lines ({}) should be >= code ({})",
            row.lang,
            row.lines,
            row.code,
        );
    }
}

/// Every export row has lines >= code.
#[test]
fn export_rows_lines_ge_code() {
    let scan = ScanSettings::for_paths(vec!["src".to_string()]);
    let receipt = export_workflow(&scan, &ExportSettings::default()).unwrap();

    for row in &receipt.data.rows {
        assert!(
            row.lines >= row.code,
            "{}: lines ({}) >= code ({})",
            row.path,
            row.lines,
            row.code,
        );
    }
}

/// Lang report total matches sum of individual row values.
#[test]
fn lang_total_matches_row_sums() {
    let scan = ScanSettings::for_paths(vec!["src".to_string()]);
    let receipt = lang_workflow(&scan, &LangSettings::default()).unwrap();

    let sum_code: usize = receipt.report.rows.iter().map(|r| r.code).sum();
    let sum_lines: usize = receipt.report.rows.iter().map(|r| r.lines).sum();
    let sum_files: usize = receipt.report.rows.iter().map(|r| r.files).sum();

    assert_eq!(receipt.report.total.code, sum_code, "total.code mismatch");
    assert_eq!(
        receipt.report.total.lines, sum_lines,
        "total.lines mismatch"
    );
    assert_eq!(
        receipt.report.total.files, sum_files,
        "total.files mismatch"
    );
}

// ============================================================================
// Full repo scan (scans tokmd repo root)
// ============================================================================

/// Scanning the repo root finds multiple languages.
#[test]
fn full_repo_scan_finds_many_languages() {
    let scan = ScanSettings::for_paths(vec![".".to_string()]);
    let lang = LangSettings::default();
    let receipt = lang_workflow(&scan, &lang).unwrap();

    // The tokmd repo has Rust, TOML, Markdown at minimum
    assert!(
        receipt.report.rows.len() >= 3,
        "full repo should have at least 3 languages, got {}",
        receipt.report.rows.len()
    );

    let langs: Vec<&str> = receipt
        .report
        .rows
        .iter()
        .map(|r| r.lang.as_str())
        .collect();
    assert!(langs.contains(&"Rust"), "should find Rust");
    assert!(langs.contains(&"TOML"), "should find TOML");
}

/// Module workflow on repo root produces modules with forward slashes.
#[test]
fn full_repo_module_scan_uses_forward_slashes() {
    let scan = ScanSettings::for_paths(vec![".".to_string()]);
    let module = ModuleSettings::default();
    let receipt = module_workflow(&scan, &module).unwrap();

    assert!(!receipt.report.rows.is_empty());
    for row in &receipt.report.rows {
        assert!(
            !row.module.contains('\\'),
            "module should use forward slashes: {}",
            row.module
        );
    }
}

// ============================================================================
// Children mode behavior
// ============================================================================

/// Collapse children mode should not produce "(embedded)" rows.
#[test]
fn lang_collapse_children_no_embedded_rows() {
    let scan = ScanSettings::for_paths(vec![".".to_string()]);
    let lang = LangSettings {
        children: tokmd_core::settings::ChildrenMode::Collapse,
        ..Default::default()
    };
    let receipt = lang_workflow(&scan, &lang).unwrap();

    for row in &receipt.report.rows {
        assert!(
            !row.lang.contains("(embedded)"),
            "collapse mode should not have embedded rows: {}",
            row.lang
        );
    }
}
