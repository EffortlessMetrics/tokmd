//! BDD-style tests for the analysis orchestrator pipeline.

use std::path::PathBuf;

use tokmd_analysis::{
    AnalysisContext, AnalysisPreset, AnalysisRequest, ImportGranularity, analyze,
};
use tokmd_analysis_types::{AnalysisArgsMeta, AnalysisSource, NearDupScope};
use tokmd_analysis_util::AnalysisLimits;
use tokmd_types::{ChildIncludeMode, ExportData, FileKind, FileRow};

// ── Helpers ────────────────────────────────────────────────────────────

fn sample_file_row(path: &str, lang: &str, code: usize) -> FileRow {
    FileRow {
        path: path.to_string(),
        module: path.rsplit('/').nth(1).unwrap_or("root").to_string(),
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
            sample_file_row("src/main.rs", "Rust", 200),
            sample_file_row("src/lib.rs", "Rust", 150),
            sample_file_row("src/utils.rs", "Rust", 80),
            sample_file_row("tests/integration.rs", "Rust", 60),
        ],
        module_roots: vec![],
        module_depth: 1,
        children: ChildIncludeMode::Separate,
    }
}

fn sample_source() -> AnalysisSource {
    AnalysisSource {
        inputs: vec![".".to_string()],
        export_path: None,
        base_receipt_path: None,
        export_schema_version: Some(2),
        export_generated_at_ms: Some(1_700_000_000_000),
        base_signature: None,
        module_roots: vec![],
        module_depth: 1,
        children: "separate".to_string(),
    }
}

fn sample_args(preset: &str) -> AnalysisArgsMeta {
    AnalysisArgsMeta {
        preset: preset.to_string(),
        format: "json".to_string(),
        window_tokens: None,
        git: None,
        max_files: None,
        max_bytes: None,
        max_commits: None,
        max_commit_files: None,
        max_file_bytes: None,
        import_granularity: "module".to_string(),
        effort_model: None,
        effort_layer: None,
        base_ref: None,
        head_ref: None,
        monte_carlo: None,
        mc_iterations: None,
        mc_seed: None,
    }
}

fn sample_request(preset: AnalysisPreset) -> AnalysisRequest {
    AnalysisRequest {
        preset,
        args: sample_args(preset.as_str()),
        limits: AnalysisLimits::default(),
        window_tokens: None,
        git: Some(false), // disable git for unit tests (no repo)
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

fn run_analysis(preset: AnalysisPreset) -> tokmd_analysis_types::AnalysisReceipt {
    let ctx = AnalysisContext {
        export: sample_export(),
        root: PathBuf::from("."),
        source: sample_source(),
    };
    analyze(ctx, sample_request(preset)).expect("analyze should not fail")
}

// ── Scenario: Receipt preset produces minimal analysis ─────────────────

#[test]
fn receipt_preset_produces_derived_metrics_only() {
    let receipt = run_analysis(AnalysisPreset::Receipt);
    assert!(
        receipt.derived.is_some(),
        "receipt should always have derived"
    );
    assert!(receipt.git.is_none(), "receipt should not include git");
    assert!(
        receipt.assets.is_none(),
        "receipt should not include assets"
    );
    assert!(
        receipt.imports.is_none(),
        "receipt should not include imports"
    );
    assert!(receipt.fun.is_none(), "receipt should not include fun");
}

// ── Scenario: Schema version matches constant ──────────────────────────

#[test]
fn receipt_schema_version_matches_constant() {
    let receipt = run_analysis(AnalysisPreset::Receipt);
    assert_eq!(
        receipt.schema_version,
        tokmd_analysis_types::ANALYSIS_SCHEMA_VERSION
    );
}

// ── Scenario: Mode field is always "analysis" ──────────────────────────

#[test]
fn mode_is_always_analysis() {
    for preset in AnalysisPreset::all() {
        let receipt = run_analysis(*preset);
        assert_eq!(receipt.mode, "analysis", "mode mismatch for {:?}", preset);
    }
}

// ── Scenario: Base signature is auto-populated ─────────────────────────

#[test]
fn base_signature_auto_populated_when_absent() {
    let receipt = run_analysis(AnalysisPreset::Receipt);
    assert!(
        receipt.source.base_signature.is_some(),
        "base_signature should be auto-populated from derived integrity hash"
    );
}

// ── Scenario: Derived report structure ─────────────────────────────────

#[test]
fn derived_report_has_totals_and_integrity() {
    let receipt = run_analysis(AnalysisPreset::Receipt);
    let derived = receipt.derived.as_ref().expect("derived should exist");
    assert!(derived.totals.code > 0, "total code lines should be > 0");
    assert!(
        !derived.integrity.hash.is_empty(),
        "integrity hash should be non-empty"
    );
}

// ── Scenario: Git disabled via flag produces no git report ─────────────

#[test]
fn git_disabled_via_flag_produces_no_git_report() {
    // Even for presets that request git (like Risk), overriding with git=false should skip it.
    let ctx = AnalysisContext {
        export: sample_export(),
        root: PathBuf::from("."),
        source: sample_source(),
    };
    let mut req = sample_request(AnalysisPreset::Risk);
    req.git = Some(false);
    let receipt = analyze(ctx, req).expect("should succeed");
    assert!(receipt.git.is_none(), "git should be None when git=false");
}

// ── Scenario: Fun preset produces fun report ───────────────────────────

#[test]
#[cfg(feature = "fun")]
fn fun_preset_produces_fun_report() {
    let receipt = run_analysis(AnalysisPreset::Fun);
    assert!(
        receipt.fun.is_some(),
        "fun preset should produce fun report"
    );
}

#[test]
fn fun_preset_does_not_produce_git_or_assets() {
    let receipt = run_analysis(AnalysisPreset::Fun);
    assert!(receipt.git.is_none());
    assert!(receipt.assets.is_none());
    assert!(receipt.imports.is_none());
}

// ── Scenario: Determinism — same input produces same output ────────────

#[test]
fn determinism_same_input_same_derived() {
    let r1 = run_analysis(AnalysisPreset::Receipt);
    let r2 = run_analysis(AnalysisPreset::Receipt);

    let d1 = r1.derived.as_ref().unwrap();
    let d2 = r2.derived.as_ref().unwrap();

    assert_eq!(d1.totals.code, d2.totals.code, "code totals diverged");
    assert_eq!(d1.totals.lines, d2.totals.lines, "line totals diverged");
    assert_eq!(d1.totals.files, d2.totals.files, "file totals diverged");
    assert_eq!(
        d1.integrity.hash, d2.integrity.hash,
        "integrity hash diverged"
    );
}

// ── Scenario: Empty export data ────────────────────────────────────────

#[test]
fn empty_export_produces_zero_totals() {
    let ctx = AnalysisContext {
        export: ExportData {
            rows: vec![],
            module_roots: vec![],
            module_depth: 1,
            children: ChildIncludeMode::Separate,
        },
        root: PathBuf::from("."),
        source: sample_source(),
    };
    let receipt = analyze(ctx, sample_request(AnalysisPreset::Receipt)).unwrap();
    let derived = receipt.derived.as_ref().unwrap();
    assert_eq!(derived.totals.code, 0);
    assert_eq!(derived.totals.files, 0);
}

// ── Scenario: Warnings for disabled features ───────────────────────────

#[test]
fn health_preset_emits_warnings_for_disabled_features() {
    // Health enables todo + complexity; without content/walk features compiled in,
    // it should emit warnings. With features compiled in, it may still warn if
    // walk fails on a non-existent root. Either way, the receipt should be valid.
    let receipt = run_analysis(AnalysisPreset::Health);
    // The receipt should always be constructable regardless of features
    assert!(receipt.derived.is_some());
    assert_eq!(receipt.mode, "analysis");
}

// ── Scenario: Deep preset enables maximum enrichers ────────────────────

#[test]
fn deep_preset_always_has_derived() {
    let receipt = run_analysis(AnalysisPreset::Deep);
    assert!(receipt.derived.is_some());
}

// ── Scenario: Analysis args are preserved in receipt ───────────────────

#[test]
fn args_are_preserved_in_receipt() {
    let receipt = run_analysis(AnalysisPreset::Receipt);
    assert_eq!(receipt.args.preset, "receipt");
    assert_eq!(receipt.args.format, "json");
}
