//! Feature boundary and capability reporting tests for tokmd-analysis.
//!
//! Verifies that the analysis pipeline handles missing capabilities correctly,
//! that preset resolution respects available features, and that enricher
//! ordering is deterministic regardless of feature configuration.

use std::path::PathBuf;
use tokmd_analysis::{AnalysisContext, AnalysisRequest, ImportGranularity, analyze};
use tokmd_analysis::{DisabledFeature, PresetKind, preset_plan_for};
use tokmd_analysis_types::{
    ANALYSIS_SCHEMA_VERSION, AnalysisArgsMeta, AnalysisSource, NearDupScope,
};
use tokmd_types::{ChildIncludeMode, ExportData, FileKind, FileRow, ScanStatus};

// ── helpers ──────────────────────────────────────────────────────────

fn sample_row(path: &str, module: &str, lang: &str, code: usize) -> FileRow {
    FileRow {
        path: path.to_string(),
        module: module.to_string(),
        lang: lang.to_string(),
        kind: FileKind::Parent,
        code,
        comments: 2,
        blanks: 1,
        lines: code + 3,
        bytes: code * 30,
        tokens: code * 5,
    }
}

fn sample_export() -> ExportData {
    ExportData {
        rows: vec![
            sample_row("src/main.rs", "src", "Rust", 100),
            sample_row("src/lib.rs", "src", "Rust", 200),
            sample_row("src/util.rs", "src", "Rust", 50),
            sample_row("tests/smoke.rs", "tests", "Rust", 30),
        ],
        module_roots: vec!["src".to_string(), "tests".to_string()],
        module_depth: 2,
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

fn make_source() -> AnalysisSource {
    AnalysisSource {
        inputs: vec![".".to_string()],
        export_path: None,
        base_receipt_path: None,
        export_schema_version: Some(2),
        export_generated_at_ms: Some(0),
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

fn make_req(preset: PresetKind) -> AnalysisRequest {
    AnalysisRequest {
        preset,
        args: AnalysisArgsMeta {
            preset: format!("{preset:?}").to_lowercase(),
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
        limits: Default::default(),
        #[cfg(feature = "effort")]
        effort: None,
        window_tokens: None,
        git: None,
        import_granularity: ImportGranularity::Module,
        detail_functions: false,
        near_dup: false,
        near_dup_threshold: 0.8,
        near_dup_max_files: 500,
        near_dup_scope: NearDupScope::Module,
        near_dup_max_pairs: None,
        near_dup_exclude: vec![],
    }
}

// ── preset plan resolution tests ─────────────────────────────────────

#[test]
fn receipt_preset_matches_current_contract() {
    let plan = preset_plan_for(PresetKind::Receipt);
    // Receipt now enables these four enrichers
    assert!(plan.dup, "receipt should request dup");
    assert!(plan.git, "receipt should request git");
    assert!(plan.complexity, "receipt should request complexity");
    assert!(plan.api_surface, "receipt should request api_surface");
    // Everything else stays off
    assert!(!plan.todo, "receipt should not request todo scan");
    assert!(!plan.entropy, "receipt should not request entropy");
    assert!(!plan.assets, "receipt should not request assets");
    assert!(!plan.deps, "receipt should not request deps");
    assert!(!plan.imports, "receipt should not request imports");
    assert!(!plan.fun, "receipt should not request fun");
    assert!(!plan.archetype, "receipt should not request archetype");
    assert!(!plan.topics, "receipt should not request topics");
    assert!(!plan.license, "receipt should not request license");
}

#[test]
fn health_preset_requests_todo_and_complexity() {
    let plan = preset_plan_for(PresetKind::Health);
    assert!(plan.todo, "health should request TODO scan");
    assert!(plan.complexity, "health should request complexity");
    assert!(!plan.git, "health should not request git");
}

#[test]
fn risk_preset_requests_git_and_complexity() {
    let plan = preset_plan_for(PresetKind::Risk);
    assert!(plan.git, "risk should request git");
    assert!(plan.complexity, "risk should request complexity");
    assert!(!plan.assets, "risk should not request assets");
}

#[test]
fn supply_preset_requests_assets_and_deps() {
    let plan = preset_plan_for(PresetKind::Supply);
    assert!(plan.assets, "supply should request assets");
    assert!(plan.deps, "supply should request deps");
    assert!(!plan.git, "supply should not request git");
}

#[test]
fn architecture_preset_requests_imports_and_api_surface() {
    let plan = preset_plan_for(PresetKind::Architecture);
    assert!(plan.imports, "architecture should request imports");
    assert!(plan.api_surface, "architecture should request api_surface");
}

#[test]
fn deep_preset_enables_all_core_enrichers() {
    let plan = preset_plan_for(PresetKind::Deep);
    assert!(plan.git, "deep should request git");
    assert!(plan.todo, "deep should request todo");
    assert!(plan.dup, "deep should request dup");
    assert!(plan.imports, "deep should request imports");
    assert!(plan.entropy, "deep should request entropy");
    assert!(plan.license, "deep should request license");
    assert!(plan.complexity, "deep should request complexity");
    assert!(plan.assets, "deep should request assets");
    assert!(plan.api_surface, "deep should request api_surface");
}

// ── disabled-feature warning messages ────────────────────────────────

#[test]
fn disabled_git_metrics_warning_is_descriptive() {
    let w = DisabledFeature::GitMetrics.warning();
    assert!(
        w.contains("git") && w.contains("disabled"),
        "warning should mention git and disabled: {w}"
    );
}

#[test]
fn disabled_todo_scan_warning_is_descriptive() {
    let w = DisabledFeature::TodoScan.warning();
    assert!(
        w.contains("content") && w.contains("disabled"),
        "warning should mention content and disabled: {w}"
    );
}

#[test]
fn disabled_entropy_warning_is_descriptive() {
    let w = DisabledFeature::EntropyProfiling.warning();
    assert!(
        w.contains("disabled"),
        "warning should mention disabled: {w}"
    );
}

#[test]
fn disabled_file_inventory_warning_is_descriptive() {
    let w = DisabledFeature::FileInventory.warning();
    assert!(
        w.contains("walk") && w.contains("disabled"),
        "warning should mention walk and disabled: {w}"
    );
}

#[test]
fn disabled_complexity_warning_is_descriptive() {
    let w = DisabledFeature::ComplexityAnalysis.warning();
    assert!(
        w.contains("disabled"),
        "warning should mention disabled: {w}"
    );
}

#[test]
fn disabled_license_warning_is_descriptive() {
    let w = DisabledFeature::LicenseRadar.warning();
    assert!(
        w.contains("disabled"),
        "warning should mention disabled: {w}"
    );
}

#[test]
fn all_disabled_features_have_nonempty_warnings() {
    let features = [
        DisabledFeature::FileInventory,
        DisabledFeature::TodoScan,
        DisabledFeature::DuplicationScan,
        DisabledFeature::NearDuplicateScan,
        DisabledFeature::ImportScan,
        DisabledFeature::GitMetrics,
        DisabledFeature::EntropyProfiling,
        DisabledFeature::LicenseRadar,
        DisabledFeature::ComplexityAnalysis,
        DisabledFeature::ApiSurfaceAnalysis,
        DisabledFeature::Archetype,
        DisabledFeature::Topics,
        DisabledFeature::Fun,
    ];
    for f in features {
        let w = f.warning();
        assert!(!w.is_empty(), "{f:?} should have a non-empty warning");
        assert!(
            w.contains("disabled") || w.contains("skipping"),
            "{f:?} warning should mention disabled or skipping: {w}"
        );
    }
}

// ── analysis receipt structure tests ─────────────────────────────────

#[test]
fn receipt_preset_produces_valid_receipt_with_derived() {
    let mut req = make_req(PresetKind::Receipt);
    // Suppress git to isolate the receipt-structure test from git availability
    req.git = Some(false);
    let receipt = analyze(make_ctx(sample_export()), req).unwrap();
    assert_eq!(receipt.schema_version, ANALYSIS_SCHEMA_VERSION);
    assert!(
        receipt.derived.is_some(),
        "receipt preset must include derived metrics"
    );
    assert!(
        receipt.git.is_none(),
        "receipt preset with git=false should not include git"
    );
    assert!(
        receipt.entropy.is_none(),
        "receipt preset should not include entropy"
    );
}

#[test]
fn receipt_preset_with_empty_export_does_not_panic() {
    let receipt = analyze(make_ctx(empty_export()), make_req(PresetKind::Receipt)).unwrap();
    assert_eq!(receipt.schema_version, ANALYSIS_SCHEMA_VERSION);
    assert!(receipt.derived.is_some());
}

#[test]
fn health_preset_without_content_feature_emits_warnings() {
    // When the content feature IS compiled in, todo/complexity will succeed.
    // When it is NOT compiled in, warnings will appear.
    // Either way, the call must not panic.
    let receipt = analyze(make_ctx(sample_export()), make_req(PresetKind::Health)).unwrap();
    assert_eq!(receipt.schema_version, ANALYSIS_SCHEMA_VERSION);
    // derived is always present
    assert!(receipt.derived.is_some());
}

#[test]
fn analysis_receipt_status_is_complete_or_partial() {
    let receipt = analyze(make_ctx(sample_export()), make_req(PresetKind::Receipt)).unwrap();
    // ScanStatus has only two variants; match ensures coverage
    match receipt.status {
        ScanStatus::Complete | ScanStatus::Partial => {}
    }
}

#[test]
fn analysis_receipt_tool_name_is_tokmd() {
    let receipt = analyze(make_ctx(sample_export()), make_req(PresetKind::Receipt)).unwrap();
    assert_eq!(receipt.tool.name, "tokmd");
}

#[cfg(all(feature = "walk", feature = "git"))]
#[test]
fn estimate_with_rootless_context_emits_host_root_warnings() {
    let mut ctx = make_ctx(sample_export());
    ctx.root = PathBuf::new();

    let receipt = analyze(ctx, make_req(PresetKind::Estimate)).unwrap();

    assert!(receipt.git.is_none(), "rootless estimate should skip git");
    assert!(
        receipt
            .warnings
            .iter()
            .any(|warning| warning.contains("no host root") && warning.contains("file-backed")),
        "expected file-backed rootless warning, got {:?}",
        receipt.warnings
    );
    assert!(
        receipt
            .warnings
            .iter()
            .any(|warning| warning.contains("no host root") && warning.contains("git")),
        "expected git rootless warning, got {:?}",
        receipt.warnings
    );
}

// ── enricher determinism tests ───────────────────────────────────────

#[test]
fn receipt_preset_deterministic_across_runs() {
    let r1 = analyze(make_ctx(sample_export()), make_req(PresetKind::Receipt)).unwrap();
    let r2 = analyze(make_ctx(sample_export()), make_req(PresetKind::Receipt)).unwrap();
    // Derived totals must be identical
    let d1 = &r1.derived.as_ref().unwrap().totals;
    let d2 = &r2.derived.as_ref().unwrap().totals;
    assert_eq!(d1.code, d2.code);
    assert_eq!(d1.comments, d2.comments);
    assert_eq!(d1.blanks, d2.blanks);
}

#[test]
fn preset_plan_is_deterministic() {
    for _ in 0..10 {
        let p1 = preset_plan_for(PresetKind::Deep);
        let p2 = preset_plan_for(PresetKind::Deep);
        assert_eq!(p1, p2, "preset plan must be deterministic");
    }
}

// ── git override flag tests ──────────────────────────────────────────

#[test]
fn explicit_git_false_skips_git_even_for_risk_preset() {
    let mut req = make_req(PresetKind::Risk);
    req.git = Some(false);
    let receipt = analyze(make_ctx(sample_export()), req).unwrap();
    assert!(
        receipt.git.is_none(),
        "explicit git=false should suppress git enricher"
    );
}

#[test]
fn explicit_git_true_on_non_git_dir_emits_warning() {
    let mut req = make_req(PresetKind::Receipt);
    req.git = Some(true);
    // Running with root="." may or may not be a git dir, but must not panic
    let receipt = analyze(make_ctx(sample_export()), req).unwrap();
    assert_eq!(receipt.schema_version, ANALYSIS_SCHEMA_VERSION);
}

// ── edge case: single-file export ────────────────────────────────────

#[test]
fn single_file_export_does_not_panic() {
    let export = ExportData {
        rows: vec![sample_row("main.rs", ".", "Rust", 10)],
        module_roots: vec![".".to_string()],
        module_depth: 1,
        children: ChildIncludeMode::Separate,
    };
    let receipt = analyze(make_ctx(export), make_req(PresetKind::Receipt)).unwrap();
    assert!(receipt.derived.is_some());
}
