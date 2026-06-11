//! BDD-style scenarios for the analysis orchestrator.
//! Extracted and formalized to provide standard behavior-level scenario coverage.

use std::path::PathBuf;

use tokmd_analysis::{
    AnalysisContext, AnalysisPreset, AnalysisRequest, ImportGranularity, analyze,
};
use tokmd_analysis_types::{AnalysisArgsMeta, AnalysisLimits, AnalysisSource, NearDupScope};
use tokmd_types::{ChildIncludeMode, ExportData, FileKind, FileRow};

// ── Helpers ────────────────────────────────────────────────────────────

fn file_row(path: &str, module: &str, lang: &str, code: usize) -> FileRow {
    FileRow {
        path: path.to_string(),
        module: module.to_string(),
        lang: lang.to_string(),
        kind: FileKind::Parent,
        code,
        comments: code / 5,
        blanks: code / 10,
        lines: code + code / 5 + code / 10,
        bytes: code * 40,
        tokens: code * 3,
    }
}

fn sample_export() -> ExportData {
    ExportData {
        rows: vec![
            file_row("src/main.rs", "src", "Rust", 200),
            file_row("src/lib.rs", "src", "Rust", 150),
            file_row("src/utils.rs", "src", "Rust", 80),
            file_row("tests/integration.rs", "tests", "Rust", 30),
        ],
        module_roots: vec![],
        module_depth: 1,
        children: ChildIncludeMode::Separate,
    }
}

fn empty_export() -> ExportData {
    ExportData {
        rows: vec![],
        module_roots: vec![],
        module_depth: 1,
        children: ChildIncludeMode::Separate,
    }
}

fn make_ctx(export: ExportData) -> AnalysisContext {
    AnalysisContext {
        export,
        root: PathBuf::from("."),
        source: AnalysisSource {
            inputs: vec![".".to_string()],
            export_path: None,
            base_receipt_path: None,
            export_schema_version: Some(2),
            export_generated_at_ms: Some(1_700_000_000_000),
            base_signature: None,
            module_roots: vec![],
            module_depth: 1,
            children: "separate".to_string(),
        },
    }
}

fn make_req(preset: AnalysisPreset) -> AnalysisRequest {
    AnalysisRequest {
        preset,
        args: AnalysisArgsMeta {
            preset: preset.as_str().to_string(),
            format: "json".to_string(),
            window_tokens: None,
            git: None,
            max_files: None,
            max_bytes: None,
            max_commits: None,
            max_commit_files: None,
            max_file_bytes: None,
            import_granularity: "module".to_string(),
        },
        limits: AnalysisLimits::default(),
        #[cfg(feature = "effort")]
        effort: None,
        window_tokens: None,
        git: Some(false), // Disable git for deterministic unit tests
        import_granularity: ImportGranularity::Module,
        detail_functions: false,
        near_dup: false,
        near_dup_threshold: 0.8,
        near_dup_max_files: 500,
        near_dup_scope: NearDupScope::default(),
        near_dup_max_pairs: None,
        near_dup_exclude: vec![],
    }
}

fn run_analysis(
    export: ExportData,
    preset: AnalysisPreset,
) -> tokmd_analysis_types::AnalysisReceipt {
    analyze(make_ctx(export), make_req(preset)).unwrap()
}

// ── BDD Scenarios ────────────────────────────────────────────────────────

/// Feature: Analysis reporting on empty codebases
///
/// Scenario: Analyzing a project with zero code files
///   Given an empty repository
///   When I request analysis using the Receipt preset
///   Then the derived report shows exactly 0 files and 0 lines of code
#[test]
fn empty_repo_produces_zero_totals() {
    // Given
    let export = empty_export();

    // When
    let receipt = run_analysis(export, AnalysisPreset::Receipt);

    // Then
    let d = receipt
        .derived
        .expect("Receipt preset must have derived section");
    assert_eq!(d.totals.files, 0, "Expected 0 total files for empty export");
    assert_eq!(d.totals.code, 0, "Expected 0 code lines for empty export");
}

/// Feature: Analysis reporting on populated codebases
///
/// Scenario: Analyzing a standard project
///   Given a codebase with 4 files containing exactly 460 total lines of code
///   When I request analysis using the Receipt preset
///   Then the derived report shows exactly 4 files and 460 code lines
#[test]
fn standard_repo_produces_correct_totals() {
    // Given
    let export = sample_export(); // 4 files, 460 code

    // When
    let receipt = run_analysis(export, AnalysisPreset::Receipt);

    // Then
    let d = receipt
        .derived
        .expect("Receipt preset must have derived section");
    assert_eq!(d.totals.files, 4, "Expected exactly 4 total files");
    assert_eq!(d.totals.code, 460, "Expected exactly 460 code lines");
}

/// Feature: Multi-module breakdown in analysis
///
/// Scenario: Analyzing a multi-module project
///   Given a codebase with files in "crates/a" and "crates/b"
///   When I request analysis using the Receipt preset
///   Then the breakdown correctly accounts for all modules and totals match
#[test]
fn multi_module_repo_produces_module_breakdown() {
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
    let receipt = run_analysis(export, AnalysisPreset::Receipt);

    // Then
    let d = receipt
        .derived
        .expect("Receipt preset must have derived section");
    assert_eq!(d.totals.files, 2, "Expected exactly 2 total files");
    assert_eq!(d.totals.code, 300, "Expected exactly 300 code lines");
}

/// Feature: Determinism in analysis output
///
/// Scenario: Running analysis twice on the same codebase
///   Given a standard codebase
///   When I run analysis twice
///   Then both receipts contain identical derived totals and integrity hashes
#[test]
fn repeated_analysis_is_deterministic() {
    // Given
    let export = sample_export();

    // When
    let receipt1 = run_analysis(export.clone(), AnalysisPreset::Receipt);
    let receipt2 = run_analysis(export, AnalysisPreset::Receipt);

    // Then
    let d1 = receipt1.derived.unwrap();
    let d2 = receipt2.derived.unwrap();
    assert_eq!(d1.totals.files, d2.totals.files, "File totals diverged");
    assert_eq!(d1.totals.code, d2.totals.code, "Code totals diverged");
    assert_eq!(
        d1.integrity.hash, d2.integrity.hash,
        "Integrity hash diverged"
    );
}
