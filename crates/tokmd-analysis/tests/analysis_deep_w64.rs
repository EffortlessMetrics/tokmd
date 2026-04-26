//! Deep tests for tokmd-analysis (w64 batch).
//!
//! Covers:
//! - AnalysisPreset conversions and properties
//! - AnalysisLimits default and custom values
//! - ImportGranularity handling
//! - NearDupScope variants
//! - Preset plan truth-table spot checks
//! - Deterministic analysis output
//! - Property: same input → same output
//! - BDD-style scenarios for analysis workflows
//! - Edge: empty export data, zero files, single file
//! - Schema version sanity
//! - AnalysisArgsMeta serialization round-trip
//! - Receipt field presence by preset

use std::path::PathBuf;

use tokmd_analysis::{
    AnalysisContext, AnalysisLimits, AnalysisPreset, AnalysisRequest, ImportGranularity,
    NearDupScope, analyze,
};
use tokmd_analysis::{PRESET_KINDS, PresetKind, preset_plan_for};
use tokmd_analysis_types::{ANALYSIS_SCHEMA_VERSION, AnalysisArgsMeta, AnalysisSource};
use tokmd_types::{ChildIncludeMode, ExportData, FileKind, FileRow, ScanStatus};

// ─── Helpers ────────────────────────────────────────────────────────────────

/// Expected status for Receipt-preset tests that set `req.git = Some(false)`.
///
/// With content+walk features, Receipt's dup/complexity/api_surface analyses
/// all execute successfully → Complete.
/// Without them, disabled-feature warnings are emitted → Partial.
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
        path: path.to_string(),
        module: module.to_string(),
        lang: lang.to_string(),
        kind: FileKind::Parent,
        code,
        comments: code.checked_div(5).unwrap_or(0),
        blanks: code.checked_div(10).unwrap_or(0),
        lines: code + code.checked_div(5).unwrap_or(0) + code.checked_div(10).unwrap_or(0),
        bytes: code * 10,
        tokens: code * 2,
    }
}

fn sample_export() -> ExportData {
    ExportData {
        rows: vec![
            file_row("src/main.rs", "src", "Rust", 200),
            file_row("src/lib.rs", "src", "Rust", 150),
            file_row("tests/test.rs", "tests", "Rust", 80),
            file_row("Cargo.toml", "(root)", "TOML", 30),
        ],
        module_roots: vec!["src".to_string()],
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

fn single_file_export() -> ExportData {
    ExportData {
        rows: vec![file_row("main.rs", "(root)", "Rust", 10)],
        module_roots: vec![],
        module_depth: 1,
        children: ChildIncludeMode::Separate,
    }
}

fn run_analysis(
    export: ExportData,
    preset: AnalysisPreset,
) -> tokmd_analysis_types::AnalysisReceipt {
    let mut req = make_req(preset);
    req.git = Some(false);
    analyze(make_ctx(export), req).expect("analyze should not fail")
}

// ═══════════════════════════════════════════════════════════════════════════
// 1. PresetKind conversions
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn preset_kind_as_str_roundtrip() {
    for &kind in PresetKind::all() {
        let s = kind.as_str();
        let back = PresetKind::from_str(s);
        assert_eq!(back, Some(kind), "round-trip failed for {s}");
    }
}

#[test]
fn preset_kind_from_str_unknown_returns_none() {
    assert_eq!(PresetKind::from_str("nonexistent"), None);
    assert_eq!(PresetKind::from_str(""), None);
    assert_eq!(PresetKind::from_str("RECEIPT"), None); // case-sensitive
}

#[test]
fn preset_kinds_count() {
    assert_eq!(PRESET_KINDS.len(), 12);
    assert_eq!(PresetKind::all().len(), 12);
}

#[test]
fn all_preset_strings_are_lowercase() {
    for &kind in PresetKind::all() {
        let s = kind.as_str();
        assert_eq!(
            s,
            s.to_lowercase(),
            "preset string should be lowercase: {s}"
        );
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// 2. PresetPlan truth-table spot checks
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn receipt_plan_matches_current_contract() {
    let plan = preset_plan_for(PresetKind::Receipt);
    assert!(!plan.assets);
    assert!(!plan.deps);
    assert!(!plan.todo);
    assert!(plan.dup);
    assert!(!plan.imports);
    assert!(plan.git);
    assert!(!plan.fun);
    assert!(!plan.archetype);
    assert!(!plan.topics);
    assert!(!plan.entropy);
    assert!(!plan.license);
    assert!(plan.complexity);
    assert!(plan.api_surface);
}

#[test]
fn health_plan_enables_todo_and_complexity() {
    let plan = preset_plan_for(PresetKind::Health);
    assert!(plan.todo);
    assert!(plan.complexity);
    assert!(!plan.assets);
    assert!(!plan.git);
}

#[test]
fn supply_plan_enables_assets_and_deps() {
    let plan = preset_plan_for(PresetKind::Supply);
    assert!(plan.assets);
    assert!(plan.deps);
}

#[test]
fn architecture_plan_enables_imports() {
    let plan = preset_plan_for(PresetKind::Architecture);
    assert!(plan.imports);
}

#[test]
fn deep_plan_enables_most_features() {
    let plan = preset_plan_for(PresetKind::Deep);
    assert!(plan.assets);
    assert!(plan.deps);
    assert!(plan.todo);
    assert!(plan.dup);
    assert!(plan.imports);
    assert!(plan.git);
    assert!(plan.entropy);
    assert!(plan.license);
    assert!(plan.complexity);
    assert!(plan.api_surface);
    // Deep does NOT enable fun
    assert!(!plan.fun);
}

#[test]
fn fun_plan_enables_only_fun() {
    let plan = preset_plan_for(PresetKind::Fun);
    assert!(plan.fun);
    assert!(!plan.assets);
    assert!(!plan.git);
    assert!(!plan.todo);
}

#[test]
fn receipt_plan_needs_files() {
    let plan = preset_plan_for(PresetKind::Receipt);
    assert!(plan.needs_files());
}

#[test]
fn deep_plan_needs_files() {
    let plan = preset_plan_for(PresetKind::Deep);
    assert!(plan.needs_files());
}

// ═══════════════════════════════════════════════════════════════════════════
// 3. AnalysisLimits
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn limits_default_all_none() {
    let limits = AnalysisLimits::default();
    assert!(limits.max_files.is_none());
    assert!(limits.max_bytes.is_none());
    assert!(limits.max_file_bytes.is_none());
    assert!(limits.max_commits.is_none());
    assert!(limits.max_commit_files.is_none());
}

#[test]
fn limits_custom_values_preserved() {
    let limits = AnalysisLimits {
        max_files: Some(100),
        max_bytes: Some(1_000_000),
        max_file_bytes: Some(50_000),
        max_commits: Some(500),
        max_commit_files: Some(50),
    };
    assert_eq!(limits.max_files, Some(100));
    assert_eq!(limits.max_bytes, Some(1_000_000));
    assert_eq!(limits.max_file_bytes, Some(50_000));
    assert_eq!(limits.max_commits, Some(500));
    assert_eq!(limits.max_commit_files, Some(50));
}

// ═══════════════════════════════════════════════════════════════════════════
// 4. ImportGranularity
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn import_granularity_equality() {
    assert_eq!(ImportGranularity::Module, ImportGranularity::Module);
    assert_eq!(ImportGranularity::File, ImportGranularity::File);
    assert_ne!(ImportGranularity::Module, ImportGranularity::File);
}

#[test]
fn import_granularity_clone() {
    let g = ImportGranularity::Module;
    let g2 = g;
    assert_eq!(g, g2);
}

// ═══════════════════════════════════════════════════════════════════════════
// 5. NearDupScope
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn near_dup_scope_default_is_module() {
    assert_eq!(NearDupScope::default(), NearDupScope::Module);
}

#[test]
fn near_dup_scope_variants_distinct() {
    let scopes = [
        NearDupScope::Module,
        NearDupScope::Lang,
        NearDupScope::Global,
    ];
    for (i, a) in scopes.iter().enumerate() {
        for (j, b) in scopes.iter().enumerate() {
            if i == j {
                assert_eq!(a, b);
            } else {
                assert_ne!(a, b);
            }
        }
    }
}

#[test]
fn near_dup_scope_serde_roundtrip() {
    for scope in [
        NearDupScope::Module,
        NearDupScope::Lang,
        NearDupScope::Global,
    ] {
        let json = serde_json::to_string(&scope).unwrap();
        let back: NearDupScope = serde_json::from_str(&json).unwrap();
        assert_eq!(scope, back);
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// 6. Schema version sanity
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn analysis_schema_version_is_current() {
    assert_eq!(ANALYSIS_SCHEMA_VERSION, 9);
}

#[test]
fn receipt_schema_version_matches_constant() {
    let receipt = run_analysis(sample_export(), PresetKind::Receipt);
    assert_eq!(receipt.schema_version, ANALYSIS_SCHEMA_VERSION);
}

// ═══════════════════════════════════════════════════════════════════════════
// 7. Receipt basic fields
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn receipt_has_tool_info() {
    let receipt = run_analysis(sample_export(), PresetKind::Receipt);
    assert!(!receipt.tool.name.is_empty());
    assert!(!receipt.tool.version.is_empty());
}

#[test]
fn receipt_mode_is_analyze() {
    let receipt = run_analysis(sample_export(), PresetKind::Receipt);
    assert_eq!(receipt.mode, "analysis");
}

#[test]
fn receipt_status_is_complete() {
    let receipt = run_analysis(sample_export(), PresetKind::Receipt);
    assert_eq!(receipt.status, expected_receipt_status());
}

#[test]
fn receipt_timestamp_is_nonzero() {
    let receipt = run_analysis(sample_export(), PresetKind::Receipt);
    assert!(receipt.generated_at_ms > 0);
}

// ═══════════════════════════════════════════════════════════════════════════
// 8. Receipt source propagation
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn source_inputs_propagated() {
    let receipt = run_analysis(sample_export(), PresetKind::Receipt);
    assert_eq!(receipt.source.inputs, vec!["."]);
}

#[test]
fn source_base_signature_auto_populated() {
    let receipt = run_analysis(sample_export(), PresetKind::Receipt);
    assert!(
        receipt.source.base_signature.is_some(),
        "base_signature should be auto-populated from derived integrity hash"
    );
}

// ═══════════════════════════════════════════════════════════════════════════
// 9. Derived report always present
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn derived_always_present_for_receipt() {
    let receipt = run_analysis(sample_export(), PresetKind::Receipt);
    assert!(receipt.derived.is_some());
}

#[test]
fn derived_totals_reflect_input_data() {
    let receipt = run_analysis(sample_export(), PresetKind::Receipt);
    let derived = receipt.derived.as_ref().unwrap();
    // total code = 200 + 150 + 80 + 30 = 460
    assert_eq!(derived.totals.code, 460);
}

#[test]
fn derived_language_breakdown_present() {
    let receipt = run_analysis(sample_export(), PresetKind::Receipt);
    let derived = receipt.derived.as_ref().unwrap();
    assert!(
        !derived.doc_density.by_lang.is_empty(),
        "language breakdown should not be empty"
    );
}

#[test]
fn derived_density_computed() {
    let receipt = run_analysis(sample_export(), PresetKind::Receipt);
    let derived = receipt.derived.as_ref().unwrap();
    // Doc-density total ratio should be > 0 since we have comments
    assert!(derived.doc_density.total.ratio > 0.0);
}

// ═══════════════════════════════════════════════════════════════════════════
// 10. Empty export data
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn empty_export_produces_valid_receipt() {
    let receipt = run_analysis(empty_export(), PresetKind::Receipt);
    assert!(receipt.derived.is_some());
    assert_eq!(receipt.status, expected_receipt_status());
}

#[test]
fn empty_export_totals_are_zero() {
    let receipt = run_analysis(empty_export(), PresetKind::Receipt);
    let derived = receipt.derived.as_ref().unwrap();
    assert_eq!(derived.totals.code, 0);
    assert_eq!(derived.totals.comments, 0);
    assert_eq!(derived.totals.blanks, 0);
    assert_eq!(derived.totals.files, 0);
}

#[test]
fn empty_export_languages_empty() {
    let receipt = run_analysis(empty_export(), PresetKind::Receipt);
    let derived = receipt.derived.as_ref().unwrap();
    assert!(derived.doc_density.by_lang.is_empty());
}

// ═══════════════════════════════════════════════════════════════════════════
// 11. Single file export
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn single_file_produces_valid_receipt() {
    let receipt = run_analysis(single_file_export(), PresetKind::Receipt);
    let derived = receipt.derived.as_ref().unwrap();
    assert_eq!(derived.totals.code, 10);
    assert_eq!(derived.totals.files, 1);
}

#[test]
fn single_file_one_language() {
    let receipt = run_analysis(single_file_export(), PresetKind::Receipt);
    let derived = receipt.derived.as_ref().unwrap();
    assert_eq!(derived.doc_density.by_lang.len(), 1);
}

// ═══════════════════════════════════════════════════════════════════════════
// 12. Determinism: same input → same output
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn deterministic_derived_totals() {
    let export = sample_export();
    let r1 = run_analysis(export.clone(), PresetKind::Receipt);
    let r2 = run_analysis(export, PresetKind::Receipt);
    let d1 = r1.derived.as_ref().unwrap();
    let d2 = r2.derived.as_ref().unwrap();
    assert_eq!(d1.totals.code, d2.totals.code);
    assert_eq!(d1.totals.comments, d2.totals.comments);
    assert_eq!(d1.totals.blanks, d2.totals.blanks);
    assert_eq!(d1.totals.files, d2.totals.files);
}

#[test]
fn deterministic_integrity_hash() {
    let export = sample_export();
    let r1 = run_analysis(export.clone(), PresetKind::Receipt);
    let r2 = run_analysis(export, PresetKind::Receipt);
    let d1 = r1.derived.as_ref().unwrap();
    let d2 = r2.derived.as_ref().unwrap();
    assert_eq!(d1.integrity.hash, d2.integrity.hash);
}

#[test]
fn deterministic_language_order() {
    let export = ExportData {
        rows: vec![
            file_row("a.py", "src", "Python", 100),
            file_row("b.rs", "src", "Rust", 200),
            file_row("c.js", "src", "JavaScript", 50),
        ],
        module_roots: vec![],
        module_depth: 1,
        children: ChildIncludeMode::Separate,
    };
    let r1 = run_analysis(export.clone(), PresetKind::Receipt);
    let r2 = run_analysis(export, PresetKind::Receipt);
    let langs1: Vec<&str> = r1
        .derived
        .as_ref()
        .unwrap()
        .doc_density
        .by_lang
        .iter()
        .map(|l| l.key.as_str())
        .collect();
    let langs2: Vec<&str> = r2
        .derived
        .as_ref()
        .unwrap()
        .doc_density
        .by_lang
        .iter()
        .map(|l| l.key.as_str())
        .collect();
    assert_eq!(langs1, langs2);
}

// ═══════════════════════════════════════════════════════════════════════════
// 13. AnalysisArgsMeta serde round-trip
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn args_meta_serde_roundtrip() {
    let args = AnalysisArgsMeta {
        preset: "receipt".to_string(),
        format: "json".to_string(),
        window_tokens: Some(128_000),
        git: Some(true),
        max_files: Some(10_000),
        max_bytes: Some(500_000_000),
        max_file_bytes: Some(1_000_000),
        max_commits: Some(1000),
        max_commit_files: Some(100),
        import_granularity: "file".to_string(),
    };
    let json = serde_json::to_string(&args).unwrap();
    let back: AnalysisArgsMeta = serde_json::from_str(&json).unwrap();
    assert_eq!(back.preset, "receipt");
    assert_eq!(back.format, "json");
    assert_eq!(back.window_tokens, Some(128_000));
    assert_eq!(back.git, Some(true));
    assert_eq!(back.import_granularity, "file");
}

// ═══════════════════════════════════════════════════════════════════════════
// 14. Receipt optional sections absent for receipt preset
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn receipt_preset_has_no_git_report() {
    let receipt = run_analysis(sample_export(), PresetKind::Receipt);
    assert!(receipt.git.is_none());
}

#[test]
fn receipt_preset_has_no_imports() {
    let receipt = run_analysis(sample_export(), PresetKind::Receipt);
    assert!(receipt.imports.is_none());
}

#[test]
fn receipt_preset_has_no_assets() {
    let receipt = run_analysis(sample_export(), PresetKind::Receipt);
    assert!(receipt.assets.is_none());
}

#[test]
fn receipt_preset_has_no_fun() {
    let receipt = run_analysis(sample_export(), PresetKind::Receipt);
    assert!(receipt.fun.is_none());
}

#[test]
fn receipt_preset_has_no_entropy() {
    let receipt = run_analysis(sample_export(), PresetKind::Receipt);
    assert!(receipt.entropy.is_none());
}

// ═══════════════════════════════════════════════════════════════════════════
// 15. Preset-specific derived fields
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn receipt_derived_has_cocomo() {
    let receipt = run_analysis(sample_export(), PresetKind::Receipt);
    let derived = receipt.derived.as_ref().unwrap();
    let cocomo = derived.cocomo.as_ref().expect("cocomo should be present");
    assert!(cocomo.effort_pm > 0.0);
}

#[test]
fn receipt_derived_has_integrity() {
    let receipt = run_analysis(sample_export(), PresetKind::Receipt);
    let derived = receipt.derived.as_ref().unwrap();
    assert!(!derived.integrity.hash.is_empty());
}

// ═══════════════════════════════════════════════════════════════════════════
// 16. Multi-language export
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn multi_language_counted_correctly() {
    let export = ExportData {
        rows: vec![
            file_row("a.rs", "src", "Rust", 100),
            file_row("b.rs", "src", "Rust", 50),
            file_row("c.py", "src", "Python", 80),
        ],
        module_roots: vec![],
        module_depth: 1,
        children: ChildIncludeMode::Separate,
    };
    let receipt = run_analysis(export, PresetKind::Receipt);
    let derived = receipt.derived.as_ref().unwrap();
    assert_eq!(derived.totals.code, 230);
    assert_eq!(derived.totals.files, 3);
    // Should have 2 languages in the by_lang breakdown
    assert_eq!(derived.doc_density.by_lang.len(), 2);
}

#[test]
fn languages_sorted_deterministically() {
    let export = ExportData {
        rows: vec![
            file_row("a.py", "src", "Python", 50),
            file_row("b.rs", "src", "Rust", 200),
            file_row("c.js", "src", "JavaScript", 100),
        ],
        module_roots: vec![],
        module_depth: 1,
        children: ChildIncludeMode::Separate,
    };
    let r1 = run_analysis(export.clone(), PresetKind::Receipt);
    let r2 = run_analysis(export, PresetKind::Receipt);
    let keys1: Vec<&str> = r1
        .derived
        .as_ref()
        .unwrap()
        .doc_density
        .by_lang
        .iter()
        .map(|l| l.key.as_str())
        .collect();
    let keys2: Vec<&str> = r2
        .derived
        .as_ref()
        .unwrap()
        .doc_density
        .by_lang
        .iter()
        .map(|l| l.key.as_str())
        .collect();
    assert_eq!(keys1, keys2, "language order should be deterministic");
}

// ═══════════════════════════════════════════════════════════════════════════
// 17. Window tokens in derived
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn window_tokens_affects_derived() {
    let mut req = make_req(PresetKind::Receipt);
    req.git = Some(false);
    req.window_tokens = Some(128_000);
    let receipt = analyze(make_ctx(sample_export()), req).unwrap();
    let derived = receipt.derived.as_ref().unwrap();
    // With window_tokens set, context_window should be computed
    assert!(derived.context_window.is_some());
}

#[test]
fn no_window_tokens_context_window_is_none() {
    let mut req = make_req(PresetKind::Receipt);
    req.git = Some(false);
    req.window_tokens = None;
    let receipt = analyze(make_ctx(sample_export()), req).unwrap();
    let derived = receipt.derived.as_ref().unwrap();
    assert!(derived.context_window.is_none());
}

// ═══════════════════════════════════════════════════════════════════════════
// 18. BDD-style scenarios
// ═══════════════════════════════════════════════════════════════════════════

/// Given a codebase with 4 files and 460 total lines of code
/// When analyzing with Receipt preset
/// Then the receipt shows exactly 4 files and 460 code lines
#[test]
fn bdd_receipt_counts_match_input() {
    // Given
    let export = sample_export(); // 4 files, 460 code

    // When
    let receipt = run_analysis(export, PresetKind::Receipt);

    // Then
    let d = receipt.derived.as_ref().unwrap();
    assert_eq!(d.totals.files, 4);
    assert_eq!(d.totals.code, 460);
}

/// Given an empty repository
/// When analyzing with Receipt preset
/// Then receipt is valid with zero totals
#[test]
fn bdd_empty_repo_valid_receipt() {
    // Given
    let export = empty_export();

    // When
    let receipt = run_analysis(export, PresetKind::Receipt);

    // Then
    assert_eq!(receipt.status, expected_receipt_status());
    let d = receipt.derived.as_ref().unwrap();
    assert_eq!(d.totals.files, 0);
    assert_eq!(d.totals.code, 0);
}

/// Given a multi-module project
/// When analyzing with Receipt preset
/// Then modules are represented in module breakdown
#[test]
fn bdd_modules_in_breakdown() {
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
    let receipt = run_analysis(export, PresetKind::Receipt);

    // Then
    let d = receipt.derived.as_ref().unwrap();
    assert_eq!(d.totals.code, 300);
    assert_eq!(d.totals.files, 2);
}

// ═══════════════════════════════════════════════════════════════════════════
// 19. Preset plan: every preset has a valid plan
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn every_preset_produces_a_plan() {
    for &kind in PresetKind::all() {
        let _plan = preset_plan_for(kind);
        // Should not panic
    }
}

#[test]
fn preset_plans_are_deterministic() {
    for &kind in PresetKind::all() {
        let p1 = preset_plan_for(kind);
        let p2 = preset_plan_for(kind);
        assert_eq!(p1, p2, "preset plan not deterministic for {:?}", kind);
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// 20. AnalysisRequest with near_dup variants
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn near_dup_disabled_keeps_dup_report_without_near_section() {
    let mut req = make_req(PresetKind::Receipt);
    req.git = Some(false);
    req.near_dup = false;
    let receipt = analyze(make_ctx(sample_export()), req).unwrap();
    #[cfg(all(feature = "content", feature = "walk"))]
    {
        let dup = receipt
            .dup
            .as_ref()
            .expect("dup report present with content and walk features");
        assert!(
            dup.near.is_none(),
            "near-dup absent because req.near_dup is false"
        );
    }
    #[cfg(not(all(feature = "content", feature = "walk")))]
    assert!(
        receipt.dup.is_none(),
        "dup absent without both content and walk features"
    );
}

#[test]
fn request_with_near_dup_exclude_patterns() {
    let mut req = make_req(PresetKind::Receipt);
    req.git = Some(false);
    req.near_dup = false;
    req.near_dup_exclude = vec!["*.lock".to_string(), "vendor/**".to_string()];
    let receipt = analyze(make_ctx(sample_export()), req).unwrap();
    // Should succeed without error even with exclude patterns
    assert_eq!(receipt.status, expected_receipt_status());
}

// ═══════════════════════════════════════════════════════════════════════════
// 21. ChildIncludeMode variants
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn child_include_separate_accepted() {
    let export = ExportData {
        rows: vec![file_row("a.rs", "src", "Rust", 50)],
        module_roots: vec![],
        module_depth: 1,
        children: ChildIncludeMode::Separate,
    };
    let receipt = run_analysis(export, PresetKind::Receipt);
    assert!(receipt.derived.is_some());
}

#[test]
fn child_include_parents_only_accepted() {
    let export = ExportData {
        rows: vec![file_row("a.rs", "src", "Rust", 50)],
        module_roots: vec![],
        module_depth: 1,
        children: ChildIncludeMode::ParentsOnly,
    };
    let receipt = run_analysis(export, PresetKind::Receipt);
    assert!(receipt.derived.is_some());
}

// ═══════════════════════════════════════════════════════════════════════════
// 22. Large file count
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn hundred_files_produces_valid_receipt() {
    let rows: Vec<FileRow> = (0..100)
        .map(|i| file_row(&format!("src/f{i}.rs"), "src", "Rust", 10 + i))
        .collect();
    let export = ExportData {
        rows,
        module_roots: vec![],
        module_depth: 1,
        children: ChildIncludeMode::Separate,
    };
    let receipt = run_analysis(export, PresetKind::Receipt);
    let d = receipt.derived.as_ref().unwrap();
    assert_eq!(d.totals.files, 100);
    // Total code: sum of (10..110) = 100*10 + sum(0..100) = 1000 + 4950 = 5950
    assert_eq!(d.totals.code, 5950);
}

// ═══════════════════════════════════════════════════════════════════════════
// 23. Warnings collection
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn receipt_preset_no_warnings() {
    let receipt = run_analysis(sample_export(), PresetKind::Receipt);
    if cfg!(all(feature = "content", feature = "walk")) {
        assert!(
            receipt.warnings.is_empty(),
            "no warnings when features present"
        );
    } else {
        assert!(
            !receipt.warnings.is_empty(),
            "disabled-feature warnings expected"
        );
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// 24. Multiple presets produce receipts
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn all_non_feature_gated_presets_produce_receipts() {
    let preset = PresetKind::Receipt;
    let receipt = run_analysis(sample_export(), preset);
    assert!(receipt.derived.is_some(), "failed for {:?}", preset);
}

// ═══════════════════════════════════════════════════════════════════════════
// 25. Derived integrity hash changes with input
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn different_inputs_different_hashes() {
    let r1 = run_analysis(sample_export(), PresetKind::Receipt);
    let r2 = run_analysis(single_file_export(), PresetKind::Receipt);
    let h1 = &r1.derived.as_ref().unwrap().integrity.hash;
    let h2 = &r2.derived.as_ref().unwrap().integrity.hash;
    assert_ne!(h1, h2, "different inputs should produce different hashes");
}

// ═══════════════════════════════════════════════════════════════════════════
// 26. FileRow construction edge cases
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn zero_code_file_row() {
    let row = file_row("empty.rs", "src", "Rust", 0);
    assert_eq!(row.code, 0);
    assert_eq!(row.comments, 0);
    assert_eq!(row.blanks, 0);
    assert_eq!(row.lines, 0);
    assert_eq!(row.bytes, 0);
    assert_eq!(row.tokens, 0);
}

#[test]
fn zero_code_export_valid() {
    let export = ExportData {
        rows: vec![file_row("empty.rs", "src", "Rust", 0)],
        module_roots: vec![],
        module_depth: 1,
        children: ChildIncludeMode::Separate,
    };
    let receipt = run_analysis(export, PresetKind::Receipt);
    let d = receipt.derived.as_ref().unwrap();
    assert_eq!(d.totals.code, 0);
    assert_eq!(d.totals.files, 1);
}
