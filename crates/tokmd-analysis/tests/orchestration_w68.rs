//! W68 deep tests for analysis orchestration pipeline.
//!
//! Covers preset resolution, enricher plans, analyze() with empty/minimal data,
//! and receipt envelope structure.

use std::path::PathBuf;

use tokmd_analysis::{
    AnalysisContext, AnalysisPreset, AnalysisRequest, ImportGranularity, analyze,
};
use tokmd_analysis_types::{ANALYSIS_SCHEMA_VERSION, AnalysisArgsMeta, AnalysisSource};
use tokmd_analysis_util::AnalysisLimits;
use tokmd_types::{ChildIncludeMode, ExportData, FileKind, FileRow, ScanStatus};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn empty_export() -> ExportData {
    ExportData {
        rows: vec![],
        module_roots: vec![],
        module_depth: 1,
        children: ChildIncludeMode::ParentsOnly,
    }
}

fn minimal_export() -> ExportData {
    ExportData {
        rows: vec![FileRow {
            path: "src/main.rs".to_string(),
            module: "src".to_string(),
            lang: "Rust".to_string(),
            kind: FileKind::Parent,
            code: 100,
            comments: 20,
            blanks: 10,
            lines: 130,
            bytes: 3_200,
            tokens: 800,
        }],
        module_roots: vec!["src".to_string()],
        module_depth: 1,
        children: ChildIncludeMode::ParentsOnly,
    }
}

fn multi_lang_export() -> ExportData {
    ExportData {
        rows: vec![
            FileRow {
                path: "src/main.rs".to_string(),
                module: "src".to_string(),
                lang: "Rust".to_string(),
                kind: FileKind::Parent,
                code: 200,
                comments: 40,
                blanks: 20,
                lines: 260,
                bytes: 6_400,
                tokens: 1_600,
            },
            FileRow {
                path: "src/util.py".to_string(),
                module: "src".to_string(),
                lang: "Python".to_string(),
                kind: FileKind::Parent,
                code: 80,
                comments: 10,
                blanks: 5,
                lines: 95,
                bytes: 2_000,
                tokens: 500,
            },
            FileRow {
                path: "tests/test_main.rs".to_string(),
                module: "tests".to_string(),
                lang: "Rust".to_string(),
                kind: FileKind::Parent,
                code: 50,
                comments: 5,
                blanks: 5,
                lines: 60,
                bytes: 1_200,
                tokens: 300,
            },
        ],
        module_roots: vec!["src".to_string(), "tests".to_string()],
        module_depth: 1,
        children: ChildIncludeMode::ParentsOnly,
    }
}

fn default_source() -> AnalysisSource {
    AnalysisSource {
        inputs: vec![".".to_string()],
        export_path: None,
        base_receipt_path: None,
        export_schema_version: Some(2),
        export_generated_at_ms: Some(0),
        base_signature: None,
        module_roots: vec![],
        module_depth: 1,
        children: "parents_only".to_string(),
    }
}

fn default_args(preset: &str) -> AnalysisArgsMeta {
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
    }
}

fn make_request(preset: AnalysisPreset) -> AnalysisRequest {
    AnalysisRequest {
        preset,
        args: default_args(preset.as_str()),
        limits: AnalysisLimits::default(),
        #[cfg(feature = "effort")]
        effort: None,
        window_tokens: None,
        git: Some(false),
        import_granularity: ImportGranularity::Module,
        detail_functions: false,
        near_dup: false,
        near_dup_threshold: 0.8,
        near_dup_max_files: 500,
        near_dup_scope: tokmd_analysis::NearDupScope::Module,
        near_dup_max_pairs: None,
        near_dup_exclude: vec![],
    }
}

fn make_context(export: ExportData) -> AnalysisContext {
    AnalysisContext {
        export,
        root: PathBuf::from("."),
        source: default_source(),
    }
}

// ---------------------------------------------------------------------------
// Preset resolution tests
// ---------------------------------------------------------------------------

#[test]
fn preset_receipt_exists() {
    use tokmd_analysis_grid::preset_plan_for;
    let plan = preset_plan_for(AnalysisPreset::Receipt);
    // Receipt preset should have everything disabled
    assert!(!plan.assets);
    assert!(!plan.deps);
    assert!(!plan.todo);
    assert!(!plan.dup);
    assert!(!plan.imports);
    assert!(!plan.git);
    assert!(!plan.fun);
    assert!(!plan.complexity);
}

#[test]
fn preset_health_enables_todo_and_complexity() {
    use tokmd_analysis_grid::preset_plan_for;
    let plan = preset_plan_for(AnalysisPreset::Health);
    assert!(plan.todo);
    assert!(plan.complexity);
    assert!(!plan.assets);
    assert!(!plan.deps);
    assert!(!plan.imports);
}

#[test]
fn preset_risk_enables_git_and_complexity() {
    use tokmd_analysis_grid::preset_plan_for;
    let plan = preset_plan_for(AnalysisPreset::Risk);
    assert!(plan.git);
    assert!(plan.complexity);
    assert!(!plan.assets);
    assert!(!plan.deps);
}

#[test]
fn preset_supply_enables_assets_and_deps() {
    use tokmd_analysis_grid::preset_plan_for;
    let plan = preset_plan_for(AnalysisPreset::Supply);
    assert!(plan.assets);
    assert!(plan.deps);
    assert!(!plan.todo);
    assert!(!plan.dup);
    assert!(!plan.imports);
    assert!(!plan.git);
}

#[test]
fn preset_architecture_enables_imports_and_api_surface() {
    use tokmd_analysis_grid::preset_plan_for;
    let plan = preset_plan_for(AnalysisPreset::Architecture);
    assert!(plan.imports);
    assert!(plan.api_surface);
    assert!(!plan.assets);
    assert!(!plan.deps);
    assert!(!plan.git);
}

#[test]
fn preset_deep_enables_everything_except_fun() {
    use tokmd_analysis_grid::preset_plan_for;
    let plan = preset_plan_for(AnalysisPreset::Deep);
    assert!(plan.assets);
    assert!(plan.deps);
    assert!(plan.todo);
    assert!(plan.dup);
    assert!(plan.imports);
    assert!(plan.git);
    assert!(plan.archetype);
    assert!(plan.topics);
    assert!(plan.entropy);
    assert!(plan.license);
    assert!(plan.complexity);
    assert!(plan.api_surface);
    assert!(!plan.fun);
}

#[test]
fn preset_fun_enables_only_fun() {
    use tokmd_analysis_grid::preset_plan_for;
    let plan = preset_plan_for(AnalysisPreset::Fun);
    assert!(plan.fun);
    assert!(!plan.assets);
    assert!(!plan.deps);
    assert!(!plan.todo);
    assert!(!plan.dup);
    assert!(!plan.imports);
    assert!(!plan.git);
    assert!(!plan.complexity);
}

#[test]
fn preset_topics_enables_only_topics() {
    use tokmd_analysis_grid::preset_plan_for;
    let plan = preset_plan_for(AnalysisPreset::Topics);
    assert!(plan.topics);
    assert!(!plan.assets);
    assert!(!plan.deps);
    assert!(!plan.git);
    assert!(!plan.fun);
}

#[test]
fn preset_security_enables_entropy_and_license() {
    use tokmd_analysis_grid::preset_plan_for;
    let plan = preset_plan_for(AnalysisPreset::Security);
    assert!(plan.entropy);
    assert!(plan.license);
    assert!(!plan.git);
    assert!(!plan.assets);
    assert!(!plan.fun);
}

#[test]
fn preset_identity_enables_archetype_and_git() {
    use tokmd_analysis_grid::preset_plan_for;
    let plan = preset_plan_for(AnalysisPreset::Identity);
    assert!(plan.archetype);
    assert!(plan.git);
    assert!(!plan.assets);
    assert!(!plan.todo);
}

#[test]
fn preset_git_enables_git() {
    use tokmd_analysis_grid::preset_plan_for;
    let plan = preset_plan_for(AnalysisPreset::Git);
    assert!(plan.git);
    assert!(!plan.assets);
    assert!(!plan.deps);
    assert!(!plan.fun);
}

#[test]
fn all_presets_have_plans() {
    use tokmd_analysis_grid::{PresetKind, preset_plan_for};
    for preset in PresetKind::all() {
        let _plan = preset_plan_for(*preset);
    }
}

#[test]
fn preset_from_str_roundtrip() {
    use tokmd_analysis_grid::PresetKind;
    for preset in PresetKind::all() {
        let s = preset.as_str();
        let parsed = PresetKind::from_str(s);
        assert_eq!(parsed, Some(*preset), "Roundtrip failed for {s}");
    }
}

#[test]
fn preset_from_str_unknown_returns_none() {
    use tokmd_analysis_grid::PresetKind;
    assert_eq!(PresetKind::from_str("nonexistent"), None);
    assert_eq!(PresetKind::from_str(""), None);
}

// ---------------------------------------------------------------------------
// analyze() with empty input
// ---------------------------------------------------------------------------

#[test]
fn analyze_empty_export_receipt_preset() {
    let ctx = make_context(empty_export());
    let req = make_request(AnalysisPreset::Receipt);
    let receipt = analyze(ctx, req).expect("analyze should succeed on empty data");

    assert_eq!(receipt.schema_version, ANALYSIS_SCHEMA_VERSION);
    assert_eq!(receipt.mode, "analysis");
    let derived = receipt.derived.expect("derived should be present");
    assert_eq!(derived.totals.files, 0);
    assert_eq!(derived.totals.code, 0);
    assert_eq!(derived.totals.lines, 0);
}

#[test]
fn analyze_empty_export_has_zero_cocomo() {
    let ctx = make_context(empty_export());
    let req = make_request(AnalysisPreset::Receipt);
    let receipt = analyze(ctx, req).unwrap();
    let derived = receipt.derived.unwrap();
    assert!(
        derived.cocomo.is_none(),
        "COCOMO should be None for zero code"
    );
}

#[test]
fn analyze_empty_export_has_zero_distribution() {
    let ctx = make_context(empty_export());
    let req = make_request(AnalysisPreset::Receipt);
    let receipt = analyze(ctx, req).unwrap();
    let derived = receipt.derived.unwrap();
    assert_eq!(derived.distribution.count, 0);
    assert_eq!(derived.distribution.min, 0);
    assert_eq!(derived.distribution.max, 0);
}

// ---------------------------------------------------------------------------
// analyze() with minimal data
// ---------------------------------------------------------------------------

#[test]
fn analyze_minimal_export_has_derived() {
    let ctx = make_context(minimal_export());
    let req = make_request(AnalysisPreset::Receipt);
    let receipt = analyze(ctx, req).unwrap();

    let derived = receipt.derived.expect("derived should be present");
    assert_eq!(derived.totals.files, 1);
    assert_eq!(derived.totals.code, 100);
    assert_eq!(derived.totals.comments, 20);
    assert_eq!(derived.totals.blanks, 10);
    assert_eq!(derived.totals.lines, 130);
}

#[test]
fn analyze_minimal_export_has_cocomo() {
    let ctx = make_context(minimal_export());
    let req = make_request(AnalysisPreset::Receipt);
    let receipt = analyze(ctx, req).unwrap();
    let derived = receipt.derived.unwrap();
    let cocomo = derived
        .cocomo
        .expect("COCOMO should be present for nonzero code");
    assert_eq!(cocomo.mode, "organic");
    assert!(cocomo.kloc > 0.0);
    assert!(cocomo.effort_pm > 0.0);
    assert!(cocomo.duration_months > 0.0);
}

#[test]
fn analyze_minimal_export_doc_density() {
    let ctx = make_context(minimal_export());
    let req = make_request(AnalysisPreset::Receipt);
    let receipt = analyze(ctx, req).unwrap();
    let derived = receipt.derived.unwrap();
    // 20 comments / (100 code + 20 comments) = 0.1667
    let ratio = derived.doc_density.total.ratio;
    assert!(ratio > 0.16 && ratio < 0.17, "doc_density ratio={ratio}");
}

// ---------------------------------------------------------------------------
// Receipt envelope structure
// ---------------------------------------------------------------------------

#[test]
fn receipt_schema_version_matches_constant() {
    let ctx = make_context(minimal_export());
    let req = make_request(AnalysisPreset::Receipt);
    let receipt = analyze(ctx, req).unwrap();
    assert_eq!(receipt.schema_version, ANALYSIS_SCHEMA_VERSION);
}

#[test]
fn receipt_has_tool_info() {
    let ctx = make_context(minimal_export());
    let req = make_request(AnalysisPreset::Receipt);
    let receipt = analyze(ctx, req).unwrap();
    assert_eq!(receipt.tool.name, "tokmd");
    assert!(!receipt.tool.version.is_empty());
}

#[test]
fn receipt_mode_is_analysis() {
    let ctx = make_context(minimal_export());
    let req = make_request(AnalysisPreset::Receipt);
    let receipt = analyze(ctx, req).unwrap();
    assert_eq!(receipt.mode, "analysis");
}

#[test]
fn receipt_status_complete_for_receipt_preset() {
    let ctx = make_context(minimal_export());
    let req = make_request(AnalysisPreset::Receipt);
    let receipt = analyze(ctx, req).unwrap();
    assert!(matches!(receipt.status, ScanStatus::Complete));
}

#[test]
fn receipt_generated_at_ms_is_nonzero() {
    let ctx = make_context(minimal_export());
    let req = make_request(AnalysisPreset::Receipt);
    let receipt = analyze(ctx, req).unwrap();
    assert!(receipt.generated_at_ms > 0);
}

#[test]
fn receipt_base_signature_populated_from_derived() {
    let ctx = make_context(minimal_export());
    let req = make_request(AnalysisPreset::Receipt);
    let receipt = analyze(ctx, req).unwrap();
    assert!(
        receipt.source.base_signature.is_some(),
        "base_signature should be populated from integrity hash"
    );
}

// ---------------------------------------------------------------------------
// Multi-lang analysis
// ---------------------------------------------------------------------------

#[test]
fn analyze_multi_lang_counts_all_files() {
    let ctx = make_context(multi_lang_export());
    let req = make_request(AnalysisPreset::Receipt);
    let receipt = analyze(ctx, req).unwrap();
    let derived = receipt.derived.unwrap();
    assert_eq!(derived.totals.files, 3);
    assert_eq!(derived.totals.code, 330); // 200 + 80 + 50
}

#[test]
fn analyze_multi_lang_doc_density_by_lang() {
    let ctx = make_context(multi_lang_export());
    let req = make_request(AnalysisPreset::Receipt);
    let receipt = analyze(ctx, req).unwrap();
    let derived = receipt.derived.unwrap();
    assert!(!derived.doc_density.by_lang.is_empty());
}

#[test]
fn analyze_multi_lang_polyglot_reports_two_langs() {
    let ctx = make_context(multi_lang_export());
    let req = make_request(AnalysisPreset::Receipt);
    let receipt = analyze(ctx, req).unwrap();
    let derived = receipt.derived.unwrap();
    assert_eq!(derived.polyglot.lang_count, 2); // Rust and Python
    assert_eq!(derived.polyglot.dominant_lang, "Rust");
}

#[test]
fn analyze_multi_lang_distribution() {
    let ctx = make_context(multi_lang_export());
    let req = make_request(AnalysisPreset::Receipt);
    let receipt = analyze(ctx, req).unwrap();
    let derived = receipt.derived.unwrap();
    assert_eq!(derived.distribution.count, 3);
    assert_eq!(derived.distribution.min, 60);
    assert_eq!(derived.distribution.max, 260);
}

// ---------------------------------------------------------------------------
// Determinism
// ---------------------------------------------------------------------------

#[test]
fn analyze_is_deterministic() {
    let run = || {
        let ctx = make_context(multi_lang_export());
        let req = make_request(AnalysisPreset::Receipt);
        let receipt = analyze(ctx, req).unwrap();
        let derived = receipt.derived.unwrap();
        (
            derived.totals.code,
            derived.totals.files,
            derived.doc_density.total.ratio,
            derived.distribution.count,
            derived.polyglot.lang_count,
        )
    };
    let a = run();
    let b = run();
    assert_eq!(a, b, "Two runs should produce identical derived metrics");
}
