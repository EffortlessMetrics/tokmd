//! W74 – Unit tests for analysis near-duplicate module enricher.

use std::path::Path;

use crate::near_dup::{NearDupLimits, build_near_dup_report};
use tokmd_analysis_types::NearDupScope;
use tokmd_types::{ChildIncludeMode, ExportData, FileKind, FileRow};

// ---------------------------------------------------------------------------
// Helper: build a minimal ExportData with given file paths
// ---------------------------------------------------------------------------
fn make_export(files: &[(&str, &str, usize)]) -> ExportData {
    let rows = files
        .iter()
        .map(|(path, lang, bytes)| FileRow {
            path: path.to_string(),
            module: "root".to_string(),
            lang: lang.to_string(),
            kind: FileKind::Parent,
            code: 50,
            comments: 5,
            blanks: 5,
            lines: 60,
            bytes: *bytes,
            tokens: 200,
        })
        .collect();
    ExportData {
        rows,
        module_roots: vec!["root".to_string()],
        module_depth: 1,
        children: ChildIncludeMode::Separate,
    }
}

// ── Empty / trivial inputs ────────────────────────────────────────────────

#[test]
fn empty_export_yields_no_pairs() {
    let export = make_export(&[]);
    let limits = NearDupLimits::default();
    let report = build_near_dup_report(
        Path::new("."),
        &export,
        NearDupScope::Global,
        0.5,
        100,
        None,
        &limits,
        &[],
    )
    .unwrap();
    assert!(report.pairs.is_empty());
    assert_eq!(report.files_analyzed, 0);
}

#[test]
fn single_file_yields_no_pairs() {
    let export = make_export(&[("src/main.rs", "Rust", 100)]);
    let limits = NearDupLimits::default();
    let report = build_near_dup_report(
        Path::new("."),
        &export,
        NearDupScope::Global,
        0.5,
        100,
        None,
        &limits,
        &[],
    )
    .unwrap();
    assert!(report.pairs.is_empty());
}

// ── Report structure ──────────────────────────────────────────────────────

#[test]
fn report_params_reflect_inputs() {
    let export = make_export(&[]);
    let limits = NearDupLimits {
        max_bytes: Some(1_000_000),
        max_file_bytes: Some(512_000),
    };
    let report = build_near_dup_report(
        Path::new("."),
        &export,
        NearDupScope::Module,
        0.75,
        50,
        Some(10),
        &limits,
        &[],
    )
    .unwrap();
    assert_eq!(report.params.threshold, 0.75);
    assert_eq!(report.params.max_files, 50);
    assert_eq!(report.params.max_pairs, Some(10));
    assert_eq!(report.params.scope, NearDupScope::Module);
}

#[test]
fn report_not_truncated_when_under_limit() {
    let export = make_export(&[]);
    let limits = NearDupLimits::default();
    let report = build_near_dup_report(
        Path::new("."),
        &export,
        NearDupScope::Global,
        0.5,
        100,
        Some(50),
        &limits,
        &[],
    )
    .unwrap();
    assert!(!report.truncated);
}

// ── Scope variants ────────────────────────────────────────────────────────

#[test]
fn scope_global_is_default_variant() {
    // Just ensure the enum variant is usable
    let scope = NearDupScope::Global;
    assert_eq!(scope, NearDupScope::Global);
}

#[test]
fn scope_module_variant_exists() {
    let scope = NearDupScope::Module;
    assert_eq!(scope, NearDupScope::Module);
}

#[test]
fn scope_lang_variant_exists() {
    let scope = NearDupScope::Lang;
    assert_eq!(scope, NearDupScope::Lang);
}

// ── Limits ────────────────────────────────────────────────────────────────

#[test]
fn default_limits_are_none() {
    let limits = NearDupLimits::default();
    assert!(limits.max_bytes.is_none());
    assert!(limits.max_file_bytes.is_none());
}

#[test]
fn files_exceeding_max_file_bytes_are_excluded() {
    // File with 1MB bytes should be excluded when limit is 512KB
    let export = make_export(&[("big.rs", "Rust", 1_000_000)]);
    let limits = NearDupLimits {
        max_bytes: None,
        max_file_bytes: Some(512_000),
    };
    let report = build_near_dup_report(
        Path::new("."),
        &export,
        NearDupScope::Global,
        0.5,
        100,
        None,
        &limits,
        &[],
    )
    .unwrap();
    // The file exceeds the per-file limit so should not be analyzed
    assert_eq!(report.files_analyzed, 0);
}

#[test]
fn exclude_patterns_filter_files() {
    let export = make_export(&[
        ("src/main.rs", "Rust", 100),
        ("tests/test_main.rs", "Rust", 100),
    ]);
    let limits = NearDupLimits::default();
    let report = build_near_dup_report(
        Path::new("."),
        &export,
        NearDupScope::Global,
        0.5,
        100,
        None,
        &limits,
        &["tests/**".to_string()],
    )
    .unwrap();
    assert_eq!(report.excluded_by_pattern, Some(1));
}
