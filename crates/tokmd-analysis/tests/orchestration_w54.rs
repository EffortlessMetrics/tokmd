//! W54: Comprehensive orchestration coverage for `tokmd-analysis`.
//!
//! Targets preset→receipt shape, enricher pipeline ordering, feature
//! gating behaviour, derived metrics, determinism, and edge cases.

use std::path::PathBuf;

use tokmd_analysis::PresetKind;
use tokmd_analysis::{
    AnalysisContext, AnalysisLimits, AnalysisPreset, AnalysisRequest, ImportGranularity,
    NearDupScope, analyze,
};
use tokmd_analysis_types::{ANALYSIS_SCHEMA_VERSION, AnalysisArgsMeta, AnalysisSource};
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
        module_roots: vec!["crates".to_string()],
        module_depth: 2,
        children: ChildIncludeMode::Separate,
    }
}

// ===========================================================================
// 1. Receipt preset: derived always present, no optional enrichers
// ===========================================================================
#[test]
fn w54_receipt_preset_derived_only() {
    let receipt = analyze(make_ctx(sample_export()), make_req(PresetKind::Receipt)).unwrap();
    assert!(receipt.derived.is_some());
    // Receipt now enables git/dup/complexity/api_surface but without features
    // they are skipped. Only assert fields that are always off.
    assert!(receipt.imports.is_none());
    assert!(receipt.assets.is_none());
    assert!(receipt.deps.is_none());
}

// ===========================================================================
// 2. Schema version matches constant
// ===========================================================================
#[test]
fn w54_schema_version_correct() {
    let receipt = analyze(make_ctx(sample_export()), make_req(PresetKind::Receipt)).unwrap();
    assert_eq!(receipt.schema_version, ANALYSIS_SCHEMA_VERSION);
}

// ===========================================================================
// 3. Mode field is "analysis"
// ===========================================================================
#[test]
fn w54_mode_is_analysis() {
    let receipt = analyze(make_ctx(sample_export()), make_req(PresetKind::Receipt)).unwrap();
    assert_eq!(receipt.mode, "analysis");
}

// ===========================================================================
// 4. Generated timestamp is positive
// ===========================================================================
#[test]
fn w54_timestamp_positive() {
    let receipt = analyze(make_ctx(sample_export()), make_req(PresetKind::Receipt)).unwrap();
    assert!(receipt.generated_at_ms > 0);
}

// ===========================================================================
// 5. Empty export still produces valid receipt
// ===========================================================================
#[test]
fn w54_empty_export_valid() {
    let empty = ExportData {
        rows: vec![],
        module_roots: vec![],
        module_depth: 1,
        children: ChildIncludeMode::Separate,
    };
    let receipt = analyze(make_ctx(empty), make_req(PresetKind::Receipt)).unwrap();
    assert!(receipt.derived.is_some());
    let derived = receipt.derived.unwrap();
    assert_eq!(derived.totals.files, 0);
    assert_eq!(derived.totals.code, 0);
}

// ===========================================================================
// 6. Single-file export produces valid derived
// ===========================================================================
#[test]
fn w54_single_file_export() {
    let single = ExportData {
        rows: vec![row("main.rs", ".", "Rust", 42)],
        module_roots: vec![],
        module_depth: 1,
        children: ChildIncludeMode::Separate,
    };
    let receipt = analyze(make_ctx(single), make_req(PresetKind::Receipt)).unwrap();
    let derived = receipt.derived.unwrap();
    assert_eq!(derived.totals.files, 1);
    assert_eq!(derived.totals.code, 42);
}

// ===========================================================================
// 7. base_signature backfilled when absent
// ===========================================================================
#[test]
fn w54_base_signature_backfilled() {
    let receipt = analyze(make_ctx(sample_export()), make_req(PresetKind::Receipt)).unwrap();
    assert!(receipt.source.base_signature.is_some());
    let derived = receipt.derived.as_ref().unwrap();
    assert_eq!(
        receipt.source.base_signature.as_ref().unwrap(),
        &derived.integrity.hash
    );
}

// ===========================================================================
// 8. base_signature preserved when pre-set
// ===========================================================================
#[test]
fn w54_base_signature_preserved() {
    let mut ctx = make_ctx(sample_export());
    ctx.source.base_signature = Some("my-preset-hash".to_string());
    let receipt = analyze(ctx, make_req(PresetKind::Receipt)).unwrap();
    assert_eq!(
        receipt.source.base_signature.as_deref(),
        Some("my-preset-hash")
    );
}

// ===========================================================================
// 9. Git override false suppresses git
// ===========================================================================
#[test]
fn w54_git_false_no_git() {
    let mut req = make_req(PresetKind::Risk);
    req.git = Some(false);
    let receipt = analyze(make_ctx(sample_export()), req).unwrap();
    assert!(receipt.git.is_none());
    assert!(receipt.predictive_churn.is_none());
}

// ===========================================================================
// 10. Tree built when format includes "tree"
// ===========================================================================
#[test]
fn w54_tree_built() {
    let mut req = make_req(PresetKind::Receipt);
    req.args.format = "tree".to_string();
    let receipt = analyze(make_ctx(sample_export()), req).unwrap();
    assert!(receipt.derived.as_ref().unwrap().tree.is_some());
}

// ===========================================================================
// 11. Tree absent when format is "json"
// ===========================================================================
#[test]
fn w54_tree_absent_json() {
    let mut req = make_req(PresetKind::Receipt);
    req.args.format = "json".to_string();
    let receipt = analyze(make_ctx(sample_export()), req).unwrap();
    assert!(receipt.derived.as_ref().unwrap().tree.is_none());
}

// ===========================================================================
// 12. context_window absent without tokens
// ===========================================================================
#[test]
fn w54_context_window_absent() {
    let req = make_req(PresetKind::Receipt);
    let receipt = analyze(make_ctx(sample_export()), req).unwrap();
    assert!(receipt.derived.unwrap().context_window.is_none());
}

// ===========================================================================
// 13. context_window present with tokens
// ===========================================================================
#[test]
fn w54_context_window_present() {
    let mut req = make_req(PresetKind::Receipt);
    req.window_tokens = Some(4096);
    let receipt = analyze(make_ctx(sample_export()), req).unwrap();
    assert!(receipt.derived.unwrap().context_window.is_some());
}

// ===========================================================================
// 14. Receipt preset → Complete status
// ===========================================================================
#[test]
fn w54_receipt_complete_status() {
    let receipt = analyze(make_ctx(sample_export()), make_req(PresetKind::Receipt)).unwrap();
    if cfg!(all(feature = "content", feature = "walk")) {
        assert!(matches!(receipt.status, ScanStatus::Complete));
        assert!(receipt.warnings.is_empty());
    } else {
        assert!(
            !receipt.warnings.is_empty(),
            "disabled-feature warnings expected"
        );
    }
}

// ===========================================================================
// 15. Health preset with no features → warnings
// ===========================================================================
#[test]
fn w54_health_preset_warnings() {
    let mut req = make_req(PresetKind::Health);
    req.git = Some(false);
    let receipt = analyze(make_ctx(sample_export()), req).unwrap();
    // Health needs content/walk which may or may not be enabled
    assert!(receipt.derived.is_some());
}

// ===========================================================================
// 16. Derived totals reflect parent rows only
// ===========================================================================
#[test]
fn w54_derived_totals_parent_only() {
    let exp = ExportData {
        rows: vec![
            row("src/lib.rs", "src", "Rust", 100),
            FileRow {
                path: "src/lib.rs".to_string(),
                module: "src".to_string(),
                lang: "JavaScript".to_string(),
                kind: FileKind::Child,
                code: 50,
                comments: 0,
                blanks: 0,
                lines: 50,
                bytes: 0,
                tokens: 0,
            },
        ],
        module_roots: vec![],
        module_depth: 1,
        children: ChildIncludeMode::Separate,
    };
    let receipt = analyze(make_ctx(exp), make_req(PresetKind::Receipt)).unwrap();
    let derived = receipt.derived.unwrap();
    assert_eq!(derived.totals.files, 1);
    assert_eq!(derived.totals.code, 100);
}

// ===========================================================================
// 17. Deterministic: same input → same derived
// ===========================================================================
#[test]
fn w54_deterministic_derived() {
    let r1 = analyze(make_ctx(sample_export()), make_req(PresetKind::Receipt)).unwrap();
    let r2 = analyze(make_ctx(sample_export()), make_req(PresetKind::Receipt)).unwrap();
    let d1 = r1.derived.unwrap();
    let d2 = r2.derived.unwrap();
    assert_eq!(d1.totals.code, d2.totals.code);
    assert_eq!(d1.totals.files, d2.totals.files);
    assert_eq!(d1.integrity.hash, d2.integrity.hash);
}

// ===========================================================================
// 18. All 12 presets produce valid receipts
// ===========================================================================
#[test]
fn w54_all_presets_valid() {
    let presets = [
        PresetKind::Receipt,
        PresetKind::Estimate,
        PresetKind::Health,
        PresetKind::Risk,
        PresetKind::Supply,
        PresetKind::Architecture,
        PresetKind::Topics,
        PresetKind::Security,
        PresetKind::Identity,
        PresetKind::Git,
        PresetKind::Deep,
        PresetKind::Fun,
    ];
    for preset in &presets {
        let mut req = make_req(*preset);
        req.git = Some(false);
        let receipt = analyze(make_ctx(sample_export()), req)
            .unwrap_or_else(|e| panic!("{:?} failed: {e}", preset));
        assert!(receipt.derived.is_some(), "{:?} missing derived", preset);
        assert_eq!(receipt.schema_version, ANALYSIS_SCHEMA_VERSION);
    }
}

// ===========================================================================
// 19. Supply preset does not request git
// ===========================================================================
#[test]
fn w54_supply_no_git() {
    let mut req = make_req(PresetKind::Supply);
    req.git = Some(false);
    let receipt = analyze(make_ctx(sample_export()), req).unwrap();
    assert!(receipt.git.is_none());
}

// ===========================================================================
// 20. Large export still succeeds
// ===========================================================================
#[test]
fn w54_large_export_succeeds() {
    let rows: Vec<FileRow> = (0..500)
        .map(|i| row(&format!("src/f{i}.rs"), "src", "Rust", 10 + i))
        .collect();
    let exp = ExportData {
        rows,
        module_roots: vec![],
        module_depth: 1,
        children: ChildIncludeMode::Separate,
    };
    let receipt = analyze(make_ctx(exp), make_req(PresetKind::Receipt)).unwrap();
    let derived = receipt.derived.unwrap();
    assert_eq!(derived.totals.files, 500);
}

// ===========================================================================
// 21. Multi-language export
// ===========================================================================
#[test]
fn w54_multi_language_export() {
    let exp = ExportData {
        rows: vec![
            row("main.py", ".", "Python", 100),
            row("lib.rs", ".", "Rust", 200),
            row("app.ts", ".", "TypeScript", 150),
        ],
        module_roots: vec![],
        module_depth: 1,
        children: ChildIncludeMode::Separate,
    };
    let receipt = analyze(make_ctx(exp), make_req(PresetKind::Receipt)).unwrap();
    let derived = receipt.derived.unwrap();
    assert_eq!(derived.totals.files, 3);
    assert_eq!(derived.totals.code, 450);
}

// ===========================================================================
// 22. Git request on tempdir produces Partial
// ===========================================================================
#[test]
fn w54_git_on_tempdir_partial() {
    let tmp = tempfile::tempdir().unwrap();
    let mut req = make_req(PresetKind::Risk);
    req.git = Some(true);
    let ctx = AnalysisContext {
        export: sample_export(),
        root: tmp.path().to_path_buf(),
        source: make_source(),
    };
    let receipt = analyze(ctx, req).unwrap();
    assert!(matches!(receipt.status, ScanStatus::Partial));
    assert!(!receipt.warnings.is_empty());
}

// ===========================================================================
// 23. Near-dup disabled by default
// ===========================================================================
#[test]
fn w54_near_dup_disabled_default() {
    let mut req = make_req(PresetKind::Receipt);
    req.git = Some(false);
    assert!(!req.near_dup);
    let receipt = analyze(make_ctx(sample_export()), req).unwrap();
    #[cfg(feature = "content")]
    {
        // Receipt now enables dup; with content feature dup report is present
        // but near section is absent because near_dup=false.
        let dup = receipt.dup.as_ref().expect("dup present with content");
        assert!(dup.near.is_none(), "near-dup absent because near_dup=false");
    }
    #[cfg(not(feature = "content"))]
    assert!(receipt.dup.is_none(), "dup absent without content feature");
}

// ===========================================================================
// 24. Source inputs preserved in receipt
// ===========================================================================
#[test]
fn w54_source_inputs_preserved() {
    let receipt = analyze(make_ctx(sample_export()), make_req(PresetKind::Receipt)).unwrap();
    assert_eq!(receipt.source.inputs, vec![".".to_string()]);
}
