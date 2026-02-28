//! Tests for enricher orchestration, preset execution, and result merging.
//!
//! Covers areas not addressed by existing test files:
//! - Preset→receipt shape (which enrichers fire per preset)
//! - base_signature backfill logic
//! - ScanStatus Complete vs Partial
//! - Git override flag (`req.git`)
//! - Tree building via format string
//! - Child row filtering
//! - context_window absent when no window_tokens
//! - schema_version and mode constants
//! - Property-based invariants on `analyze`

use std::path::PathBuf;

use proptest::prelude::*;
use tokmd_analysis::{
    AnalysisContext, AnalysisLimits, AnalysisPreset, AnalysisRequest, ImportGranularity,
    NearDupScope, analyze,
};
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

fn child_row(path: &str, module: &str, lang: &str, code: usize) -> FileRow {
    FileRow {
        path: path.to_string(),
        module: module.to_string(),
        lang: lang.to_string(),
        kind: FileKind::Child,
        code,
        comments: 0,
        blanks: 0,
        lines: code,
        bytes: 0,
        tokens: 0,
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

// ═══════════════════════════════════════════════════════════════════════════
// Scenario: Receipt envelope metadata
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn receipt_has_correct_schema_version() {
    // Given: any export
    // When: analyze returns a receipt
    // Then: schema_version matches ANALYSIS_SCHEMA_VERSION constant
    let receipt = analyze(make_ctx(sample_export()), make_req(AnalysisPreset::Receipt)).unwrap();
    assert_eq!(receipt.schema_version, ANALYSIS_SCHEMA_VERSION);
}

#[test]
fn receipt_mode_is_analysis() {
    let receipt = analyze(make_ctx(sample_export()), make_req(AnalysisPreset::Receipt)).unwrap();
    assert_eq!(receipt.mode, "analysis");
}

#[test]
fn receipt_generated_at_ms_is_nonzero() {
    let receipt = analyze(make_ctx(sample_export()), make_req(AnalysisPreset::Receipt)).unwrap();
    assert!(receipt.generated_at_ms > 0);
}

// ═══════════════════════════════════════════════════════════════════════════
// Scenario: ScanStatus – Complete vs Partial
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn receipt_preset_with_no_warnings_is_complete() {
    // Given: Receipt preset requests only derived metrics (no file walk / git)
    // When: analyze succeeds without warnings
    // Then: status is Complete
    let receipt = analyze(make_ctx(sample_export()), make_req(AnalysisPreset::Receipt)).unwrap();
    assert!(
        matches!(receipt.status, ScanStatus::Complete),
        "Receipt preset should be Complete, got {:?}",
        receipt.status
    );
    assert!(receipt.warnings.is_empty());
}

#[test]
fn git_request_without_repo_produces_partial_status() {
    // Given: request with git explicitly enabled
    // When: the root is not a git repo
    // Then: status is Partial and warning is emitted (feature-gated)
    let mut req = make_req(AnalysisPreset::Risk);
    req.git = Some(true);

    let receipt = analyze(make_ctx(sample_export()), req).unwrap();
    // Without git feature, a warning about missing feature gate is emitted;
    // with git feature, a warning about not being a git repo is emitted.
    // Either way: warnings should exist → Partial.
    assert!(
        matches!(receipt.status, ScanStatus::Partial),
        "Should be Partial when git is requested on non-repo"
    );
    assert!(!receipt.warnings.is_empty());
}

// ═══════════════════════════════════════════════════════════════════════════
// Scenario: base_signature backfill
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn base_signature_is_backfilled_from_integrity_hash() {
    // Given: source has no base_signature
    // When: analyze runs
    // Then: source.base_signature is set to the derived integrity hash
    let receipt = analyze(make_ctx(sample_export()), make_req(AnalysisPreset::Receipt)).unwrap();
    let derived = receipt.derived.as_ref().unwrap();
    let sig = receipt.source.base_signature.as_ref().unwrap();
    assert_eq!(sig, &derived.integrity.hash);
}

#[test]
fn base_signature_is_preserved_when_already_set() {
    // Given: source already has a base_signature
    // When: analyze runs
    // Then: the pre-existing base_signature is kept
    let mut ctx = make_ctx(sample_export());
    ctx.source.base_signature = Some("pre-existing-hash".to_string());

    let receipt = analyze(ctx, make_req(AnalysisPreset::Receipt)).unwrap();
    assert_eq!(
        receipt.source.base_signature.as_deref(),
        Some("pre-existing-hash")
    );
}

// ═══════════════════════════════════════════════════════════════════════════
// Scenario: Git override flag
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn git_false_override_suppresses_git_enrichment() {
    // Given: a preset that normally wants git (Risk)
    // When: req.git = Some(false) overrides the plan
    // Then: no git report, no churn, no fingerprint
    let mut req = make_req(AnalysisPreset::Risk);
    req.git = Some(false);

    let receipt = analyze(make_ctx(sample_export()), req).unwrap();
    assert!(receipt.git.is_none());
    assert!(receipt.predictive_churn.is_none());
    assert!(receipt.corporate_fingerprint.is_none());
}

#[test]
fn git_none_defers_to_preset_plan() {
    // Given: a preset that does NOT want git (Receipt)
    // When: req.git = None (defer to plan)
    // Then: no git sections
    let req = make_req(AnalysisPreset::Receipt);
    let receipt = analyze(make_ctx(sample_export()), req).unwrap();
    assert!(receipt.git.is_none());
    assert!(receipt.predictive_churn.is_none());
}

// ═══════════════════════════════════════════════════════════════════════════
// Scenario: Tree building via format string
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn tree_is_built_when_format_contains_tree() {
    // Given: args.format includes "tree"
    // When: analyze runs
    // Then: derived.tree is Some
    let mut req = make_req(AnalysisPreset::Receipt);
    req.args.format = "tree".to_string();

    let receipt = analyze(make_ctx(sample_export()), req).unwrap();
    let derived = receipt.derived.as_ref().unwrap();
    assert!(derived.tree.is_some(), "tree should be present");
}

#[test]
fn tree_absent_when_format_is_json() {
    // Given: args.format = "json" (no "tree")
    // When: analyze runs
    // Then: derived.tree is None
    let mut req = make_req(AnalysisPreset::Receipt);
    req.args.format = "json".to_string();

    let receipt = analyze(make_ctx(sample_export()), req).unwrap();
    let derived = receipt.derived.as_ref().unwrap();
    assert!(derived.tree.is_none(), "tree should not be built for json");
}

// ═══════════════════════════════════════════════════════════════════════════
// Scenario: Child row exclusion from derived totals
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn child_rows_excluded_from_file_count_and_totals() {
    // Given: export with parent and child rows
    // When: derived metrics are computed
    // Then: child rows are not counted in files/code totals
    let export = ExportData {
        rows: vec![
            row("src/lib.rs", "src", "Rust", 100),
            child_row("src/lib.rs", "src", "JavaScript", 20),
        ],
        module_roots: vec!["crates".to_string()],
        module_depth: 2,
        children: ChildIncludeMode::Separate,
    };

    let receipt = analyze(make_ctx(export), make_req(AnalysisPreset::Receipt)).unwrap();
    let derived = receipt.derived.unwrap();

    assert_eq!(derived.totals.files, 1, "child rows should not count as files");
    assert_eq!(derived.totals.code, 100, "child code should not inflate totals");
}

// ═══════════════════════════════════════════════════════════════════════════
// Scenario: context_window absent/present
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn context_window_absent_when_no_window_tokens() {
    // Given: req.window_tokens = None
    // When: analyze runs
    // Then: derived.context_window is None
    let req = make_req(AnalysisPreset::Receipt);
    let receipt = analyze(make_ctx(sample_export()), req).unwrap();
    let derived = receipt.derived.unwrap();
    assert!(derived.context_window.is_none());
}

#[test]
fn context_window_present_when_window_tokens_set() {
    // Given: req.window_tokens = Some(8000)
    // When: analyze runs
    // Then: derived.context_window is Some
    let mut req = make_req(AnalysisPreset::Receipt);
    req.window_tokens = Some(8000);

    let receipt = analyze(make_ctx(sample_export()), req).unwrap();
    let derived = receipt.derived.unwrap();
    assert!(derived.context_window.is_some());
}

// ═══════════════════════════════════════════════════════════════════════════
// Scenario: Preset Receipt produces only derived (no optional enrichers)
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn receipt_preset_produces_only_derived() {
    let receipt = analyze(make_ctx(sample_export()), make_req(AnalysisPreset::Receipt)).unwrap();

    assert!(receipt.derived.is_some(), "derived must be present");
    // Receipt preset should not trigger any optional enrichers
    assert!(receipt.git.is_none());
    assert!(receipt.imports.is_none());
    assert!(receipt.dup.is_none());
    assert!(receipt.assets.is_none());
    assert!(receipt.deps.is_none());
    assert!(receipt.entropy.is_none());
    assert!(receipt.license.is_none());
    assert!(receipt.complexity.is_none());
    assert!(receipt.api_surface.is_none());
    assert!(receipt.predictive_churn.is_none());
    assert!(receipt.fun.is_none());
}

// ═══════════════════════════════════════════════════════════════════════════
// Scenario: All presets produce valid receipts
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn all_presets_produce_valid_receipt() {
    let presets = [
        AnalysisPreset::Receipt,
        AnalysisPreset::Health,
        AnalysisPreset::Risk,
        AnalysisPreset::Supply,
        AnalysisPreset::Architecture,
        AnalysisPreset::Topics,
        AnalysisPreset::Security,
        AnalysisPreset::Identity,
        AnalysisPreset::Git,
        AnalysisPreset::Deep,
        AnalysisPreset::Fun,
    ];

    for preset in &presets {
        let mut req = make_req(*preset);
        // Disable git to avoid needing a real repo
        req.git = Some(false);

        let receipt = analyze(make_ctx(sample_export()), req)
            .unwrap_or_else(|e| panic!("preset {:?} failed: {}", preset, e));

        assert!(
            receipt.derived.is_some(),
            "preset {:?} should always produce derived",
            preset
        );
        assert_eq!(receipt.schema_version, ANALYSIS_SCHEMA_VERSION);
        assert_eq!(receipt.mode, "analysis");
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// Scenario: Fun preset with fun feature enabled
// ═══════════════════════════════════════════════════════════════════════════

#[cfg(feature = "fun")]
#[test]
fn fun_preset_produces_fun_report_and_derived() {
    let receipt = analyze(make_ctx(sample_export()), make_req(AnalysisPreset::Fun)).unwrap();
    assert!(receipt.derived.is_some());
    assert!(receipt.fun.is_some(), "fun preset should produce fun report");
}

#[cfg(feature = "fun")]
#[test]
fn non_fun_presets_do_not_produce_fun_report() {
    // Receipt, Health, Risk, etc. should not have fun
    for preset in [
        AnalysisPreset::Receipt,
        AnalysisPreset::Health,
        AnalysisPreset::Risk,
    ] {
        let mut req = make_req(preset);
        req.git = Some(false);
        let receipt = analyze(make_ctx(sample_export()), req).unwrap();
        assert!(
            receipt.fun.is_none(),
            "preset {:?} should not produce fun report",
            preset
        );
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// Scenario: Topics preset with topics feature enabled
// ═══════════════════════════════════════════════════════════════════════════

#[cfg(feature = "topics")]
#[test]
fn topics_preset_produces_topics_report() {
    let export = ExportData {
        rows: vec![
            row("crates/auth/src/login.rs", "crates/auth", "Rust", 50),
            row("crates/db/src/pool.rs", "crates/db", "Rust", 40),
        ],
        module_roots: vec!["crates".to_string()],
        module_depth: 2,
        children: ChildIncludeMode::Separate,
    };

    let receipt = analyze(make_ctx(export), make_req(AnalysisPreset::Topics)).unwrap();
    assert!(receipt.topics.is_some());
}

#[cfg(feature = "topics")]
#[test]
fn receipt_preset_does_not_produce_topics() {
    let receipt = analyze(make_ctx(sample_export()), make_req(AnalysisPreset::Receipt)).unwrap();
    assert!(receipt.topics.is_none());
}

// ═══════════════════════════════════════════════════════════════════════════
// Scenario: Archetype detection
// ═══════════════════════════════════════════════════════════════════════════

#[cfg(feature = "archetype")]
#[test]
fn identity_preset_produces_archetype() {
    // Must look like a Rust workspace: root Cargo.toml + crates sub-Cargo.toml
    let export = ExportData {
        rows: vec![
            row("Cargo.toml", "(root)", "TOML", 10),
            row("crates/core/Cargo.toml", "crates/core", "TOML", 5),
            row("src/main.rs", "src", "Rust", 100),
        ],
        module_roots: vec!["crates".to_string()],
        module_depth: 2,
        children: ChildIncludeMode::Separate,
    };

    let mut req = make_req(AnalysisPreset::Identity);
    req.git = Some(false);

    let receipt = analyze(make_ctx(export), req).unwrap();
    assert!(receipt.archetype.is_some());
}

#[cfg(feature = "archetype")]
#[test]
fn receipt_preset_does_not_produce_archetype() {
    let receipt = analyze(make_ctx(sample_export()), make_req(AnalysisPreset::Receipt)).unwrap();
    assert!(receipt.archetype.is_none());
}

// ═══════════════════════════════════════════════════════════════════════════
// Scenario: Single-file edge case
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn single_file_produces_valid_metrics() {
    let export = ExportData {
        rows: vec![row("main.rs", "(root)", "Rust", 42)],
        module_roots: vec![],
        module_depth: 1,
        children: ChildIncludeMode::Separate,
    };

    let receipt = analyze(make_ctx(export), make_req(AnalysisPreset::Receipt)).unwrap();
    let derived = receipt.derived.unwrap();

    assert_eq!(derived.totals.files, 1);
    assert_eq!(derived.totals.code, 42);
    assert_eq!(derived.polyglot.lang_count, 1);
    assert_eq!(derived.polyglot.dominant_lang, "Rust");
    assert!((derived.polyglot.dominant_pct - 1.0).abs() < 0.001);
    assert_eq!(derived.distribution.count, 1);
    assert_eq!(derived.distribution.min, derived.distribution.max);
    assert!((derived.distribution.gini - 0.0).abs() < 0.001);
}

// ═══════════════════════════════════════════════════════════════════════════
// Scenario: Empty export
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn empty_export_produces_valid_receipt() {
    let export = ExportData {
        rows: vec![],
        module_roots: vec![],
        module_depth: 1,
        children: ChildIncludeMode::Separate,
    };

    let receipt = analyze(make_ctx(export), make_req(AnalysisPreset::Receipt)).unwrap();
    let derived = receipt.derived.unwrap();

    assert_eq!(derived.totals.files, 0);
    assert_eq!(derived.totals.code, 0);
    assert_eq!(derived.polyglot.lang_count, 0);
    assert!(derived.cocomo.is_none());
    assert_eq!(derived.integrity.entries, 0);
}

// ═══════════════════════════════════════════════════════════════════════════
// Scenario: Large file count does not panic
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn large_file_count_does_not_panic() {
    let rows: Vec<FileRow> = (0..500)
        .map(|i| row(&format!("src/file_{}.rs", i), "src", "Rust", (i + 1) * 3))
        .collect();
    let export = ExportData {
        rows,
        module_roots: vec!["crates".to_string()],
        module_depth: 2,
        children: ChildIncludeMode::Separate,
    };

    let receipt = analyze(make_ctx(export), make_req(AnalysisPreset::Receipt)).unwrap();
    let derived = receipt.derived.unwrap();
    assert_eq!(derived.totals.files, 500);
    assert_eq!(derived.top.largest_lines.len(), 10); // top-10 capped
}

// ═══════════════════════════════════════════════════════════════════════════
// Scenario: args passthrough
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn args_meta_is_preserved_in_receipt() {
    let mut req = make_req(AnalysisPreset::Receipt);
    req.args.preset = "receipt".to_string();
    req.args.format = "md".to_string();
    req.args.window_tokens = Some(4096);

    let receipt = analyze(make_ctx(sample_export()), req).unwrap();
    assert_eq!(receipt.args.preset, "receipt");
    assert_eq!(receipt.args.format, "md");
    assert_eq!(receipt.args.window_tokens, Some(4096));
}

// ═══════════════════════════════════════════════════════════════════════════
// Scenario: source passthrough
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn source_inputs_and_module_roots_are_preserved() {
    let mut ctx = make_ctx(sample_export());
    ctx.source.inputs = vec!["/foo/bar".to_string()];
    ctx.source.module_roots = vec!["packages".to_string()];
    ctx.source.module_depth = 3;

    let receipt = analyze(ctx, make_req(AnalysisPreset::Receipt)).unwrap();
    assert_eq!(receipt.source.inputs, vec!["/foo/bar"]);
    assert_eq!(receipt.source.module_roots, vec!["packages"]);
    assert_eq!(receipt.source.module_depth, 3);
}

// ═══════════════════════════════════════════════════════════════════════════
// Scenario: Context window fits logic
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn context_window_does_not_fit_when_tokens_exceed_window() {
    let export = ExportData {
        rows: vec![FileRow {
            path: "big.rs".to_string(),
            module: "src".to_string(),
            lang: "Rust".to_string(),
            kind: FileKind::Parent,
            code: 1000,
            comments: 0,
            blanks: 0,
            lines: 1000,
            bytes: 10000,
            tokens: 5000,
        }],
        module_roots: vec![],
        module_depth: 1,
        children: ChildIncludeMode::Separate,
    };

    let mut req = make_req(AnalysisPreset::Receipt);
    req.window_tokens = Some(1000); // 5000 tokens > 1000 window

    let receipt = analyze(make_ctx(export), req).unwrap();
    let cw = receipt.derived.unwrap().context_window.unwrap();

    assert!(!cw.fits, "5000 tokens should not fit in 1000 window");
    assert!(cw.pct > 1.0, "pct should exceed 1.0");
}

// ═══════════════════════════════════════════════════════════════════════════
// Scenario: Integrity hash stability
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn integrity_hash_is_stable_across_runs() {
    let export = sample_export();
    let receipt1 = analyze(make_ctx(export.clone()), make_req(AnalysisPreset::Receipt)).unwrap();
    let receipt2 = analyze(make_ctx(export), make_req(AnalysisPreset::Receipt)).unwrap();

    let h1 = &receipt1.derived.unwrap().integrity.hash;
    let h2 = &receipt2.derived.unwrap().integrity.hash;
    assert_eq!(h1, h2, "same input should yield same integrity hash");
}

// ═══════════════════════════════════════════════════════════════════════════
// Scenario: Polyglot with many languages
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn polyglot_many_languages() {
    let langs = ["Rust", "Python", "Go", "Java", "TypeScript", "C", "Haskell"];
    let rows: Vec<FileRow> = langs
        .iter()
        .enumerate()
        .map(|(i, lang)| row(&format!("src/file.{}", i), "src", lang, 100))
        .collect();
    let export = ExportData {
        rows,
        module_roots: vec![],
        module_depth: 1,
        children: ChildIncludeMode::Separate,
    };

    let receipt = analyze(make_ctx(export), make_req(AnalysisPreset::Receipt)).unwrap();
    let poly = &receipt.derived.unwrap().polyglot;

    assert_eq!(poly.lang_count, 7);
    // All equal → entropy should be log2(7) ≈ 2.807
    assert!(
        (poly.entropy - 7_f64.log2()).abs() < 0.01,
        "entropy should be log2(7)≈2.807, got {}",
        poly.entropy
    );
}

// ═══════════════════════════════════════════════════════════════════════════
// Scenario: ImportGranularity variants
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn file_import_granularity_does_not_affect_derived_metrics() {
    let mut req_module = make_req(AnalysisPreset::Receipt);
    req_module.import_granularity = ImportGranularity::Module;

    let mut req_file = make_req(AnalysisPreset::Receipt);
    req_file.import_granularity = ImportGranularity::File;

    let export = sample_export();
    let r1 = analyze(make_ctx(export.clone()), req_module).unwrap();
    let r2 = analyze(make_ctx(export), req_file).unwrap();

    // Derived metrics should be identical regardless of import granularity
    let d1 = serde_json::to_string(&r1.derived).unwrap();
    let d2 = serde_json::to_string(&r2.derived).unwrap();
    assert_eq!(d1, d2);
}

// ═══════════════════════════════════════════════════════════════════════════
// Scenario: Mixed ChildIncludeMode
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn parents_only_child_mode_works() {
    let export = ExportData {
        rows: vec![
            row("src/main.rs", "src", "Rust", 100),
            child_row("src/main.rs", "src", "JavaScript", 30),
        ],
        module_roots: vec![],
        module_depth: 1,
        children: ChildIncludeMode::ParentsOnly,
    };

    let receipt = analyze(make_ctx(export), make_req(AnalysisPreset::Receipt)).unwrap();
    assert!(receipt.derived.is_some());
}

// ═══════════════════════════════════════════════════════════════════════════
// Property-based tests
// ═══════════════════════════════════════════════════════════════════════════

fn arb_file_row() -> impl Strategy<Value = FileRow> {
    (1..500_usize, 0..100_usize, 0..50_usize).prop_map(|(code, comments, blanks)| FileRow {
        path: "src/arb.rs".to_string(),
        module: "src".to_string(),
        lang: "Rust".to_string(),
        kind: FileKind::Parent,
        code,
        comments,
        blanks,
        lines: code + comments + blanks,
        bytes: (code + comments + blanks) * 10,
        tokens: code * 2,
    })
}

proptest! {
    #[test]
    fn prop_analyze_never_panics(
        code in 0..10_000_usize,
        comments in 0..1_000_usize,
        blanks in 0..500_usize,
    ) {
        let lines = code + comments + blanks;
        let export = ExportData {
            rows: vec![FileRow {
                path: "src/f.rs".to_string(),
                module: "src".to_string(),
                lang: "Rust".to_string(),
                kind: FileKind::Parent,
                code,
                comments,
                blanks,
                lines,
                bytes: lines * 10,
                tokens: code * 2,
            }],
            module_roots: vec![],
            module_depth: 1,
            children: ChildIncludeMode::Separate,
        };
        let _ = analyze(make_ctx(export), make_req(AnalysisPreset::Receipt));
    }

    #[test]
    fn prop_derived_totals_match_parent_rows(rows in proptest::collection::vec(arb_file_row(), 1..20)) {
        let expected_files = rows.len();
        let expected_code: usize = rows.iter().map(|r| r.code).sum();
        let expected_comments: usize = rows.iter().map(|r| r.comments).sum();
        let expected_blanks: usize = rows.iter().map(|r| r.blanks).sum();

        let export = ExportData {
            rows,
            module_roots: vec![],
            module_depth: 1,
            children: ChildIncludeMode::Separate,
        };
        let receipt = analyze(make_ctx(export), make_req(AnalysisPreset::Receipt)).unwrap();
        let d = receipt.derived.unwrap();

        prop_assert_eq!(d.totals.files, expected_files);
        prop_assert_eq!(d.totals.code, expected_code);
        prop_assert_eq!(d.totals.comments, expected_comments);
        prop_assert_eq!(d.totals.blanks, expected_blanks);
    }

    #[test]
    fn prop_cocomo_kloc_equals_code_div_1000(code in 1..100_000_usize) {
        let export = ExportData {
            rows: vec![FileRow {
                path: "a.rs".to_string(),
                module: "src".to_string(),
                lang: "Rust".to_string(),
                kind: FileKind::Parent,
                code,
                comments: 0,
                blanks: 0,
                lines: code,
                bytes: code * 10,
                tokens: code * 2,
            }],
            module_roots: vec![],
            module_depth: 1,
            children: ChildIncludeMode::Separate,
        };
        let receipt = analyze(make_ctx(export), make_req(AnalysisPreset::Receipt)).unwrap();
        let cocomo = receipt.derived.unwrap().cocomo.unwrap();
        let expected_kloc = code as f64 / 1000.0;
        prop_assert!((cocomo.kloc - expected_kloc).abs() < 0.001);
    }

    #[test]
    fn prop_gini_is_zero_to_one(rows in proptest::collection::vec(arb_file_row(), 2..30)) {
        let export = ExportData {
            rows,
            module_roots: vec![],
            module_depth: 1,
            children: ChildIncludeMode::Separate,
        };
        let receipt = analyze(make_ctx(export), make_req(AnalysisPreset::Receipt)).unwrap();
        let gini = receipt.derived.unwrap().distribution.gini;
        prop_assert!(gini >= 0.0 && gini <= 1.0, "gini={} out of [0,1]", gini);
    }

    #[test]
    fn prop_reading_time_proportional_to_code(code in 1..100_000_usize) {
        let export = ExportData {
            rows: vec![FileRow {
                path: "a.rs".to_string(),
                module: "src".to_string(),
                lang: "Rust".to_string(),
                kind: FileKind::Parent,
                code,
                comments: 0,
                blanks: 0,
                lines: code,
                bytes: code * 10,
                tokens: code * 2,
            }],
            module_roots: vec![],
            module_depth: 1,
            children: ChildIncludeMode::Separate,
        };
        let receipt = analyze(make_ctx(export), make_req(AnalysisPreset::Receipt)).unwrap();
        let rt = receipt.derived.unwrap().reading_time;
        let expected = code as f64 / 20.0;
        prop_assert!((rt.minutes - expected).abs() < 0.01);
    }

    #[test]
    fn prop_integrity_entry_count_equals_file_count(rows in proptest::collection::vec(arb_file_row(), 0..50)) {
        let expected = rows.len();
        let export = ExportData {
            rows,
            module_roots: vec![],
            module_depth: 1,
            children: ChildIncludeMode::Separate,
        };
        let receipt = analyze(make_ctx(export), make_req(AnalysisPreset::Receipt)).unwrap();
        let entries = receipt.derived.unwrap().integrity.entries;
        prop_assert_eq!(entries, expected);
    }

    #[test]
    fn prop_doc_density_in_zero_one(code in 1..5_000_usize, comments in 0..2_000_usize) {
        let export = ExportData {
            rows: vec![FileRow {
                path: "a.rs".to_string(),
                module: "src".to_string(),
                lang: "Rust".to_string(),
                kind: FileKind::Parent,
                code,
                comments,
                blanks: 0,
                lines: code + comments,
                bytes: (code + comments) * 10,
                tokens: code * 2,
            }],
            module_roots: vec![],
            module_depth: 1,
            children: ChildIncludeMode::Separate,
        };
        let receipt = analyze(make_ctx(export), make_req(AnalysisPreset::Receipt)).unwrap();
        let ratio = receipt.derived.unwrap().doc_density.total.ratio;
        prop_assert!(ratio >= 0.0 && ratio <= 1.0, "doc_density={} out of [0,1]", ratio);
    }
}
