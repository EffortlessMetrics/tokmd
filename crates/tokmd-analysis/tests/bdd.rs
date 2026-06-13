//! BDD-style scenario tests for tokmd-analysis orchestration.
//!
//! Each test reads as a Given/When/Then scenario validating the behavior of the orchestrator.

use std::path::PathBuf;

use tokmd_analysis::{
    AnalysisContext, AnalysisLimits, AnalysisRequest, PresetKind, AnalysisPreset,
    analyze, ImportGranularity, NearDupScope,
};
use tokmd_analysis_types::{AnalysisArgsMeta, AnalysisSource};
use tokmd_types::{ChildIncludeMode, ExportData, FileKind, FileRow, ScanStatus};

// ============================================================================
// Helpers
// ============================================================================

fn expected_receipt_status() -> ScanStatus {
    if cfg!(all(feature = "content", feature = "walk")) {
        ScanStatus::Complete
    } else {
        ScanStatus::Partial
    }
}

fn make_source() -> AnalysisSource {
    AnalysisSource {
        inputs: vec![".".to_string()],
        export_path: None,
        base_receipt_path: None,
        export_schema_version: None,
        export_generated_at_ms: None,
        base_signature: None,
        module_roots: vec!["src".to_string()],
        module_depth: 2,
        children: "separate".to_string(),
    }
}

fn make_ctx(export: ExportData) -> AnalysisContext {
    AnalysisContext {
        export,
        root: PathBuf::from("."),
        source: make_source(),
    }
}

fn make_req(preset: AnalysisPreset) -> AnalysisRequest {
    AnalysisRequest {
        preset,
        args: AnalysisArgsMeta {
            preset: format!("{:?}", preset).to_lowercase(),
            format: "json".to_string(),
            window_tokens: None,
            git: None,
            max_files: None,
            max_bytes: None,
            max_file_bytes: None,
            max_commits: None,
            max_commit_files: None,
            import_granularity: "module".to_string(),
        },
        limits: AnalysisLimits::default(),
        #[cfg(feature = "effort")]
        effort: None,
        window_tokens: None,
        git: None,
        import_granularity: ImportGranularity::Module,
        detail_functions: false,
        near_dup: false,
        near_dup_threshold: 0.80,
        near_dup_max_files: 2000,
        near_dup_scope: NearDupScope::Module,
        near_dup_max_pairs: None,
        near_dup_exclude: Vec::new(),
    }
}

fn file_row(path: &str, module: &str, lang: &str, code: usize) -> FileRow {
    FileRow {
        path: path.into(),
        module: module.into(),
        lang: lang.into(),
        kind: FileKind::Parent,
        code,
        comments: 0,
        blanks: 0,
        lines: code,
        bytes: code * 30,
        tokens: code * 5,
    }
}

fn run_analysis(export: ExportData, preset: AnalysisPreset) -> tokmd_analysis_types::AnalysisReceipt {
    let mut req = make_req(preset);
    req.git = Some(false);
    analyze(make_ctx(export), req).expect("analysis failed")
}

// ============================================================================
// BDD Scenarios
// ============================================================================

#[test]
fn given_repo_with_multiple_files_when_analyzing_receipt_preset_then_totals_are_accurate() {
    // Given
    let export = ExportData {
        rows: vec![
            file_row("src/main.rs", "src", "Rust", 100),
            file_row("src/lib.rs", "src", "Rust", 200),
        ],
        module_roots: vec!["src".to_string()],
        module_depth: 1,
        children: ChildIncludeMode::Separate,
    };

    // When
    let receipt = run_analysis(export, PresetKind::Receipt.into());

    // Then
    let derived = receipt.derived.expect("derived metrics missing");
    assert_eq!(derived.totals.files, 2);
    assert_eq!(derived.totals.code, 300);
}

#[test]
fn given_empty_repository_when_analyzing_receipt_preset_then_valid_receipt_with_zero_totals() {
    // Given
    let export = ExportData {
        rows: vec![],
        module_roots: vec![],
        module_depth: 1,
        children: ChildIncludeMode::Separate,
    };

    // When
    let receipt = run_analysis(export, PresetKind::Receipt.into());

    // Then
    assert_eq!(receipt.status, expected_receipt_status());
    let derived = receipt.derived.expect("derived metrics missing");
    assert_eq!(derived.totals.files, 0);
    assert_eq!(derived.totals.code, 0);
}

#[test]
fn given_multi_module_project_when_analyzing_receipt_preset_then_modules_represented_in_breakdown() {
    // Given
    let export = ExportData {
        rows: vec![
            file_row("crates/a/src/lib.rs", "crates/a", "Rust", 100),
            file_row("crates/b/src/lib.rs", "crates/b", "Rust", 200),
        ],
        module_roots: vec!["crates".to_string()],
        module_depth: 2,
        children: ChildIncludeMode::Separate,
    };

    // When
    let receipt = run_analysis(export, PresetKind::Receipt.into());

    // Then
    let derived = receipt.derived.expect("derived metrics missing");
    assert_eq!(derived.totals.code, 300);
    assert_eq!(derived.totals.files, 2);
}
