//! Deep analysis orchestration tests (wave 48).
//!
//! Covers:
//! - Preset resolution for all known presets
//! - Enricher registration and execution order
//! - Analysis receipt structure validation
//! - Empty scan input handling
//! - Feature capability reporting in analysis output

use std::path::PathBuf;

use tokmd_analysis::{
    analyze, AnalysisContext, AnalysisLimits, AnalysisPreset, AnalysisRequest, ImportGranularity,
    NearDupScope,
};
use tokmd_analysis_grid::{preset_plan_for, PresetKind};
use tokmd_analysis_types::{AnalysisArgsMeta, AnalysisSource, ANALYSIS_SCHEMA_VERSION};
use tokmd_types::{ChildIncludeMode, ExportData, FileKind, FileRow, ScanStatus};

// ─── Helpers ────────────────────────────────────────────────────────────────

fn make_source() -> AnalysisSource {
    AnalysisSource {
        inputs: vec![".".to_string()],
        export_path: None,
        base_receipt_path: None,
        export_schema_version: None,
        export_generated_at_ms: None,
        base_signature: None,
        module_roots: vec![],
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
        effort: None,
        window_tokens: None,
        git: Some(false),
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

fn row(path: &str, module: &str, lang: &str, code: usize) -> FileRow {
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
            row("src/main.rs", "src", "Rust", 200),
            row("src/lib.rs", "src", "Rust", 150),
            row("tests/test.rs", "tests", "Rust", 80),
            row("Cargo.toml", "(root)", "TOML", 30),
        ],
        module_roots: vec![],
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

fn run(export: ExportData, preset: AnalysisPreset) -> tokmd_analysis_types::AnalysisReceipt {
    analyze(make_ctx(export), make_req(preset)).expect("analyze should succeed")
}

// ═══════════════════════════════════════════════════════════════════════════
// 1. Preset resolution — every preset produces a valid receipt
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn every_preset_produces_receipt_with_correct_schema_version() {
    for preset in PresetKind::all() {
        let receipt = run(sample_export(), *preset);
        assert_eq!(
            receipt.schema_version, ANALYSIS_SCHEMA_VERSION,
            "schema_version mismatch for {:?}",
            preset
        );
    }
}

#[test]
fn every_preset_produces_mode_analysis() {
    for preset in PresetKind::all() {
        let receipt = run(sample_export(), *preset);
        assert_eq!(receipt.mode, "analysis", "mode mismatch for {:?}", preset);
    }
}

#[test]
fn receipt_preset_has_derived_section() {
    let receipt = run(sample_export(), PresetKind::Receipt);
    assert!(
        receipt.derived.is_some(),
        "Receipt preset must include derived metrics"
    );
}

#[test]
fn receipt_preset_has_no_optional_enrichers() {
    let receipt = run(sample_export(), PresetKind::Receipt);
    assert!(receipt.assets.is_none());
    assert!(receipt.deps.is_none());
    assert!(receipt.git.is_none());
    assert!(receipt.imports.is_none());
    assert!(receipt.fun.is_none());
}

#[test]
fn health_preset_plan_enables_todo_and_complexity() {
    let plan = preset_plan_for(PresetKind::Health);
    assert!(plan.todo);
    assert!(plan.complexity);
    assert!(!plan.git);
    assert!(!plan.assets);
}

#[test]
fn risk_preset_plan_enables_git_and_complexity() {
    let plan = preset_plan_for(PresetKind::Risk);
    assert!(plan.git);
    assert!(plan.complexity);
    assert!(!plan.todo);
}

#[test]
fn supply_preset_plan_enables_assets_and_deps() {
    let plan = preset_plan_for(PresetKind::Supply);
    assert!(plan.assets);
    assert!(plan.deps);
    assert!(!plan.git);
    assert!(!plan.todo);
}

#[test]
fn deep_preset_plan_is_superset_of_receipt() {
    let deep = preset_plan_for(PresetKind::Deep);
    let receipt = preset_plan_for(PresetKind::Receipt);
    // Deep enables everything Receipt enables (Receipt enables nothing)
    // Plus Deep enables all major enrichers
    assert!(deep.assets);
    assert!(deep.deps);
    assert!(deep.todo);
    assert!(deep.dup);
    assert!(deep.imports);
    assert!(deep.git);
    assert!(!receipt.assets);
}

// ═══════════════════════════════════════════════════════════════════════════
// 2. Enricher registration and execution order
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn derived_always_populated_regardless_of_preset() {
    for preset in PresetKind::all() {
        let receipt = run(sample_export(), *preset);
        assert!(
            receipt.derived.is_some(),
            "derived should always be populated for {:?}",
            preset
        );
    }
}

#[test]
fn derived_totals_reflect_input_rows() {
    let receipt = run(sample_export(), PresetKind::Receipt);
    let derived = receipt.derived.unwrap();
    assert_eq!(derived.totals.files, 4);
    // total code = 200 + 150 + 80 + 30 = 460
    assert_eq!(derived.totals.code, 460);
}

#[test]
fn integrity_hash_present_in_derived() {
    let receipt = run(sample_export(), PresetKind::Receipt);
    let derived = receipt.derived.unwrap();
    assert!(!derived.integrity.hash.is_empty());
    assert_eq!(derived.integrity.algo, "blake3");
}

#[test]
fn base_signature_backfilled_when_absent() {
    let receipt = run(sample_export(), PresetKind::Receipt);
    let derived = receipt.derived.as_ref().unwrap();
    assert_eq!(
        receipt.source.base_signature.as_deref(),
        Some(derived.integrity.hash.as_str())
    );
}

#[test]
fn base_signature_preserved_when_provided() {
    let mut ctx = make_ctx(sample_export());
    ctx.source.base_signature = Some("custom-hash".to_string());
    let receipt = analyze(ctx, make_req(PresetKind::Receipt)).unwrap();
    assert_eq!(
        receipt.source.base_signature.as_deref(),
        Some("custom-hash")
    );
}

// ═══════════════════════════════════════════════════════════════════════════
// 3. Analysis receipt structure validation
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn receipt_json_roundtrip_preserves_structure() {
    let receipt = run(sample_export(), PresetKind::Receipt);
    let json = serde_json::to_string(&receipt).unwrap();
    let deserialized: tokmd_analysis_types::AnalysisReceipt = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.schema_version, receipt.schema_version);
    assert_eq!(deserialized.mode, receipt.mode);
    assert_eq!(deserialized.args.preset, receipt.args.preset);
}

#[test]
fn receipt_tool_info_populated() {
    let receipt = run(sample_export(), PresetKind::Receipt);
    assert_eq!(receipt.tool.name, "tokmd");
    assert!(!receipt.tool.version.is_empty());
}

#[test]
fn receipt_source_inputs_preserved() {
    let receipt = run(sample_export(), PresetKind::Receipt);
    assert_eq!(receipt.source.inputs, vec!["."]);
}

#[test]
fn receipt_warnings_empty_for_receipt_preset() {
    let receipt = run(sample_export(), PresetKind::Receipt);
    assert!(
        receipt.warnings.is_empty(),
        "Receipt preset should have no warnings: {:?}",
        receipt.warnings
    );
    assert!(matches!(receipt.status, ScanStatus::Complete));
}

#[test]
fn receipt_generated_at_ms_is_recent() {
    let receipt = run(sample_export(), PresetKind::Receipt);
    let now_approx = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis();
    // Should be within 10 seconds of now
    assert!(receipt.generated_at_ms > now_approx - 10_000);
    assert!(receipt.generated_at_ms <= now_approx + 1_000);
}

// ═══════════════════════════════════════════════════════════════════════════
// 4. Empty scan input handling
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn empty_export_produces_valid_receipt() {
    let receipt = run(empty_export(), PresetKind::Receipt);
    assert_eq!(receipt.schema_version, ANALYSIS_SCHEMA_VERSION);
    assert!(receipt.derived.is_some());
}

#[test]
fn empty_export_derived_totals_are_zero() {
    let receipt = run(empty_export(), PresetKind::Receipt);
    let derived = receipt.derived.unwrap();
    assert_eq!(derived.totals.files, 0);
    assert_eq!(derived.totals.code, 0);
    assert_eq!(derived.totals.comments, 0);
    assert_eq!(derived.totals.blanks, 0);
    assert_eq!(derived.totals.lines, 0);
    assert_eq!(derived.totals.bytes, 0);
    assert_eq!(derived.totals.tokens, 0);
}

#[test]
fn empty_export_all_presets_succeed() {
    for preset in PresetKind::all() {
        let result = analyze(make_ctx(empty_export()), make_req(*preset));
        assert!(
            result.is_ok(),
            "Empty export failed for {:?}: {}",
            preset,
            result.unwrap_err()
        );
    }
}

#[test]
fn empty_export_integrity_hash_still_present() {
    let receipt = run(empty_export(), PresetKind::Receipt);
    let derived = receipt.derived.unwrap();
    assert!(!derived.integrity.hash.is_empty());
    assert_eq!(derived.integrity.entries, 0);
}

// ═══════════════════════════════════════════════════════════════════════════
// 5. Feature capability reporting in analysis output
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn deep_preset_produces_receipt_regardless_of_features() {
    // Deep preset should always produce a receipt, even with reduced features
    let receipt = run(sample_export(), PresetKind::Deep);
    // When optional features (git/content/walk) are disabled, warnings are emitted
    // and status is Partial; when all are enabled, status is Complete.
    // Either way, the receipt is valid.
    assert!(receipt.schema_version > 0);
    assert!(receipt.derived.is_some());
}

#[test]
fn fun_preset_emits_fun_report_or_warning() {
    let receipt = run(sample_export(), PresetKind::Fun);
    // Either fun report is populated (if fun feature enabled) or a warning is emitted
    let has_fun = receipt.fun.is_some();
    let has_warning = receipt
        .warnings
        .iter()
        .any(|w| w.contains("fun") || w.contains("eco-label"));
    assert!(
        has_fun || has_warning,
        "Fun preset should produce fun report or warning"
    );
}

#[test]
fn window_tokens_populates_context_window() {
    let mut req = make_req(PresetKind::Receipt);
    req.window_tokens = Some(100_000);
    let receipt = analyze(make_ctx(sample_export()), req).unwrap();
    let derived = receipt.derived.unwrap();
    assert!(derived.context_window.is_some());
    let ctx = derived.context_window.unwrap();
    assert_eq!(ctx.window_tokens, 100_000);
    assert!(ctx.fits); // 920 tokens < 100k
}

#[test]
fn window_tokens_none_omits_context_window() {
    let mut req = make_req(PresetKind::Receipt);
    req.window_tokens = None;
    let receipt = analyze(make_ctx(sample_export()), req).unwrap();
    let derived = receipt.derived.unwrap();
    assert!(derived.context_window.is_none());
}

#[test]
fn tree_format_populates_tree_field() {
    let mut req = make_req(PresetKind::Receipt);
    req.args.format = "tree".to_string();
    let receipt = analyze(make_ctx(sample_export()), req).unwrap();
    let derived = receipt.derived.unwrap();
    assert!(derived.tree.is_some());
}

#[test]
fn non_tree_format_omits_tree_field() {
    let mut req = make_req(PresetKind::Receipt);
    req.args.format = "json".to_string();
    let receipt = analyze(make_ctx(sample_export()), req).unwrap();
    let derived = receipt.derived.unwrap();
    assert!(derived.tree.is_none());
}
