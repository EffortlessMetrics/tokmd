//! Cross-crate enricher pipeline integration tests.
//!
//! Verifies preset expansion, determinism, schema versioning, and
//! empty-input handling across the analysis enricher pipeline.

use std::path::PathBuf;

use tokmd_analysis::{
    AnalysisContext, AnalysisLimits, AnalysisPreset, AnalysisRequest, ImportGranularity,
    NearDupScope, analyze,
};
use tokmd_analysis::{PresetKind, preset_plan_for};
use tokmd_analysis_types::{ANALYSIS_SCHEMA_VERSION, AnalysisArgsMeta, AnalysisSource};
use tokmd_types::{ChildIncludeMode, ExportData, FileKind, FileRow, ScanStatus};

// ============================================================================
// Helpers
// ============================================================================

fn make_source() -> AnalysisSource {
    AnalysisSource {
        inputs: vec![".".to_string()],
        export_path: None,
        base_receipt_path: None,
        export_schema_version: None,
        export_generated_at_ms: None,
        base_signature: None,
        module_roots: vec!["crates".to_string()],
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
            preset: preset.as_str().to_string(),
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

fn sample_row(path: &str, module: &str, lang: &str, code: usize) -> FileRow {
    FileRow {
        path: path.to_string(),
        module: module.to_string(),
        lang: lang.to_string(),
        kind: FileKind::Parent,
        code,
        comments: code / 5,
        blanks: code / 10,
        lines: code + code / 5 + code / 10,
        bytes: code * 10,
        tokens: code * 2,
    }
}

fn sample_export() -> ExportData {
    ExportData {
        rows: vec![
            sample_row("src/main.rs", "src", "Rust", 200),
            sample_row("src/lib.rs", "src", "Rust", 150),
            sample_row("tests/test.rs", "tests", "Rust", 80),
            sample_row("Cargo.toml", "(root)", "TOML", 30),
        ],
        module_roots: vec!["crates".to_string()],
        module_depth: 2,
        children: ChildIncludeMode::Separate,
    }
}

fn empty_export() -> ExportData {
    ExportData {
        rows: vec![],
        module_roots: vec![],
        module_depth: 2,
        children: ChildIncludeMode::Separate,
    }
}

// ============================================================================
// Preset expansion
// ============================================================================

#[test]
fn receipt_preset_enables_only_derived() {
    let plan = preset_plan_for(PresetKind::Receipt);
    // Receipt now enables dup, git, complexity, api_surface
    assert!(plan.dup, "receipt should request dup");
    assert!(plan.git, "receipt should request git");
    assert!(plan.complexity, "receipt should request complexity");
    assert!(plan.api_surface, "receipt should request api_surface");
    // Everything else stays off
    assert!(!plan.assets);
    assert!(!plan.deps);
    assert!(!plan.todo);
    assert!(!plan.imports);
    assert!(!plan.fun);
}

#[test]
fn health_preset_enables_todo_and_complexity() {
    let plan = preset_plan_for(PresetKind::Health);
    assert!(plan.todo, "health should enable todo scanning");
    assert!(plan.complexity, "health should enable complexity");
}

#[test]
fn risk_preset_enables_git() {
    let plan = preset_plan_for(PresetKind::Risk);
    assert!(plan.git, "risk should enable git metrics");
    assert!(plan.complexity, "risk should enable complexity");
}

#[test]
fn deep_preset_enables_most_enrichers() {
    let plan = preset_plan_for(PresetKind::Deep);
    assert!(plan.assets, "deep should enable assets");
    assert!(plan.deps, "deep should enable deps");
    assert!(plan.todo, "deep should enable todo");
    assert!(plan.dup, "deep should enable dup");
    assert!(plan.imports, "deep should enable imports");
    assert!(plan.git, "deep should enable git");
    assert!(plan.archetype, "deep should enable archetype");
    assert!(plan.entropy, "deep should enable entropy");
    assert!(plan.license, "deep should enable license");
    assert!(plan.complexity, "deep should enable complexity");
}

#[test]
fn fun_preset_enables_fun_only() {
    let plan = preset_plan_for(PresetKind::Fun);
    assert!(plan.fun, "fun should enable fun");
    assert!(!plan.git, "fun should not enable git");
    assert!(!plan.assets, "fun should not enable assets");
}

// ============================================================================
// Deterministic output
// ============================================================================

#[test]
fn analysis_receipt_deterministic_across_runs() {
    let export = sample_export();

    let r1 = analyze(make_ctx(export.clone()), make_req(AnalysisPreset::Receipt)).unwrap();
    let r2 = analyze(make_ctx(export), make_req(AnalysisPreset::Receipt)).unwrap();

    let d1 = r1.derived.as_ref().expect("derived present");
    let d2 = r2.derived.as_ref().expect("derived present");

    assert_eq!(d1.totals.code, d2.totals.code);
    assert_eq!(d1.totals.files, d2.totals.files);
    assert_eq!(d1.totals.lines, d2.totals.lines);
    assert_eq!(d1.integrity.hash, d2.integrity.hash);
}

#[test]
fn integrity_hash_changes_with_different_input() {
    let mut export_a = sample_export();
    let export_b = sample_export();

    // Modify one row.
    export_a.rows[0].code = 999;
    export_a.rows[0].lines = 1199;

    let r1 = analyze(make_ctx(export_a), make_req(AnalysisPreset::Receipt)).unwrap();
    let r2 = analyze(make_ctx(export_b), make_req(AnalysisPreset::Receipt)).unwrap();

    let h1 = &r1.derived.as_ref().unwrap().integrity.hash;
    let h2 = &r2.derived.as_ref().unwrap().integrity.hash;
    assert_ne!(h1, h2, "different inputs must produce different hashes");
}

// ============================================================================
// Schema version
// ============================================================================

#[test]
fn analysis_receipt_schema_version_matches_constant() {
    let receipt = analyze(make_ctx(sample_export()), make_req(AnalysisPreset::Receipt)).unwrap();
    assert_eq!(receipt.schema_version, ANALYSIS_SCHEMA_VERSION);
}

#[test]
fn analysis_receipt_mode_is_analysis() {
    let receipt = analyze(make_ctx(sample_export()), make_req(AnalysisPreset::Receipt)).unwrap();
    assert_eq!(receipt.mode, "analysis");
}

#[test]
fn analysis_receipt_generated_at_ms_positive() {
    let receipt = analyze(make_ctx(sample_export()), make_req(AnalysisPreset::Receipt)).unwrap();
    assert!(receipt.generated_at_ms > 0);
}

// ============================================================================
// Empty input handling
// ============================================================================

#[test]
fn empty_export_receipt_preset_succeeds() {
    let receipt = analyze(make_ctx(empty_export()), make_req(AnalysisPreset::Receipt)).unwrap();

    let derived = receipt.derived.expect("derived should be present");
    assert_eq!(derived.totals.files, 0);
    assert_eq!(derived.totals.code, 0);
    assert_eq!(derived.totals.lines, 0);
}

#[test]
fn empty_export_health_preset_succeeds() {
    let receipt = analyze(make_ctx(empty_export()), make_req(AnalysisPreset::Health)).unwrap();

    // Health adds TODO scanning; with empty input it should still succeed.
    let derived = receipt.derived.expect("derived should be present");
    assert_eq!(derived.totals.files, 0);
}

#[test]
fn empty_export_has_complete_status() {
    let receipt = analyze(make_ctx(empty_export()), make_req(AnalysisPreset::Receipt)).unwrap();
    if cfg!(all(feature = "content", feature = "walk")) {
        assert!(
            matches!(receipt.status, ScanStatus::Complete),
            "empty receipt preset should be Complete, got {:?}",
            receipt.status
        );
    } else {
        // Receipt now requests dup/complexity/api_surface which need content+walk
        assert!(
            matches!(receipt.status, ScanStatus::Partial),
            "expected Partial without features, got {:?}",
            receipt.status
        );
    }
}

#[test]
fn empty_export_json_round_trips() {
    let receipt = analyze(make_ctx(empty_export()), make_req(AnalysisPreset::Receipt)).unwrap();

    let json = serde_json::to_string(&receipt).expect("serialize");
    let parsed: serde_json::Value = serde_json::from_str(&json).expect("parse");

    assert_eq!(parsed["mode"], "analysis");
    assert_eq!(
        parsed["schema_version"].as_u64().unwrap() as u32,
        ANALYSIS_SCHEMA_VERSION
    );
}
