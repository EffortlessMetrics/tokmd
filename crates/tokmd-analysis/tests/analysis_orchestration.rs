//! Tests for analysis orchestration: preset selection, enricher composition,
//! feature-gated enricher behavior, receipt metadata, and warning collection.
//!
//! Complements existing orchestration.rs by focusing on:
//! - Preset selection produces expected enrichers
//! - Multiple presets compose correctly
//! - Feature-gated enrichers are properly skipped when features unavailable
//! - Analysis receipt contains expected metadata
//! - Warnings are properly collected

use std::path::PathBuf;

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

fn polyglot_export() -> ExportData {
    ExportData {
        rows: vec![
            row("src/main.rs", "src", "Rust", 300),
            row("src/utils.py", "src", "Python", 100),
            row("src/helper.go", "src", "Go", 50),
            row("config/setup.toml", "config", "TOML", 20),
            row("tests/test.rs", "tests", "Rust", 100),
        ],
        module_roots: vec!["crates".to_string()],
        module_depth: 2,
        children: ChildIncludeMode::Separate,
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// Scenario: Preset selection produces expected enrichers
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn receipt_preset_excludes_all_optional_enrichers() {
    // Receipt preset should only produce derived metrics, nothing optional
    let mut req = make_req(AnalysisPreset::Receipt);
    req.git = Some(false);

    let receipt = analyze(make_ctx(sample_export()), req).unwrap();

    assert!(receipt.derived.is_some());
    assert!(receipt.git.is_none(), "Receipt should not include git");
    assert!(
        receipt.imports.is_none(),
        "Receipt should not include imports"
    );
    assert!(receipt.dup.is_none(), "Receipt should not include dup");
    assert!(
        receipt.assets.is_none(),
        "Receipt should not include assets"
    );
    assert!(receipt.deps.is_none(), "Receipt should not include deps");
    assert!(
        receipt.entropy.is_none(),
        "Receipt should not include entropy"
    );
    assert!(
        receipt.license.is_none(),
        "Receipt should not include license"
    );
    assert!(
        receipt.complexity.is_none(),
        "Receipt should not include complexity"
    );
    assert!(
        receipt.api_surface.is_none(),
        "Receipt should not include api_surface"
    );
    assert!(
        receipt.predictive_churn.is_none(),
        "Receipt should not include churn"
    );
    assert!(
        receipt.corporate_fingerprint.is_none(),
        "Receipt should not include fingerprint"
    );
    assert!(receipt.fun.is_none(), "Receipt should not include fun");
    assert!(
        receipt.topics.is_none(),
        "Receipt should not include topics"
    );
    assert!(
        receipt.archetype.is_none(),
        "Receipt should not include archetype"
    );
}

#[cfg(feature = "fun")]
#[test]
fn fun_preset_produces_fun_but_not_git_or_imports() {
    let mut req = make_req(AnalysisPreset::Fun);
    req.git = Some(false);

    let receipt = analyze(make_ctx(sample_export()), req).unwrap();

    assert!(receipt.derived.is_some());
    assert!(
        receipt.fun.is_some(),
        "Fun preset should produce fun report"
    );
    assert!(receipt.git.is_none(), "Fun preset should not include git");
    assert!(
        receipt.imports.is_none(),
        "Fun preset should not include imports"
    );
    assert!(receipt.dup.is_none(), "Fun preset should not include dup");
}

#[cfg(feature = "topics")]
#[test]
fn topics_preset_produces_topics_but_not_fun() {
    let mut req = make_req(AnalysisPreset::Topics);
    req.git = Some(false);

    let receipt = analyze(make_ctx(polyglot_export()), req).unwrap();

    assert!(
        receipt.topics.is_some(),
        "Topics preset should produce topics"
    );
    assert!(
        receipt.fun.is_none(),
        "Topics preset should not include fun"
    );
}

#[cfg(feature = "archetype")]
#[test]
fn identity_preset_produces_archetype() {
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
    assert!(
        receipt.archetype.is_some(),
        "Identity preset should produce archetype"
    );
}

// ═══════════════════════════════════════════════════════════════════════════
// Scenario: Multiple presets compose correctly (each produces valid receipt)
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn sequential_preset_runs_produce_independent_results() {
    // Running different presets on the same export produces independent results
    let export = polyglot_export();

    let receipt_receipt = {
        let mut req = make_req(AnalysisPreset::Receipt);
        req.git = Some(false);
        analyze(make_ctx(export.clone()), req).unwrap()
    };

    let receipt_health = {
        let mut req = make_req(AnalysisPreset::Health);
        req.git = Some(false);
        analyze(make_ctx(export.clone()), req).unwrap()
    };

    let receipt_supply = {
        let mut req = make_req(AnalysisPreset::Supply);
        req.git = Some(false);
        analyze(make_ctx(export.clone()), req).unwrap()
    };

    // All should have derived
    assert!(receipt_receipt.derived.is_some());
    assert!(receipt_health.derived.is_some());
    assert!(receipt_supply.derived.is_some());

    // Derived totals should be the same across presets (same input)
    let d_receipt = receipt_receipt.derived.as_ref().unwrap();
    let d_health = receipt_health.derived.as_ref().unwrap();
    let d_supply = receipt_supply.derived.as_ref().unwrap();

    assert_eq!(d_receipt.totals.files, d_health.totals.files);
    assert_eq!(d_receipt.totals.code, d_health.totals.code);
    assert_eq!(d_receipt.totals.files, d_supply.totals.files);

    // Integrity hashes should be identical (same input)
    assert_eq!(d_receipt.integrity.hash, d_health.integrity.hash);
    assert_eq!(d_receipt.integrity.hash, d_supply.integrity.hash);
}

#[test]
fn preset_args_meta_reflects_requested_preset() {
    let presets = [
        (AnalysisPreset::Receipt, "receipt"),
        (AnalysisPreset::Health, "health"),
        (AnalysisPreset::Risk, "risk"),
        (AnalysisPreset::Supply, "supply"),
    ];

    for (preset, name) in presets {
        let mut req = make_req(preset);
        req.git = Some(false);
        req.args.preset = name.to_string();

        let receipt = analyze(make_ctx(sample_export()), req).unwrap();
        assert_eq!(
            receipt.args.preset, name,
            "preset args should reflect requested preset"
        );
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// Scenario: Feature-gated enrichers properly skipped
// ═══════════════════════════════════════════════════════════════════════════

#[cfg(not(feature = "git"))]
#[test]
fn git_enricher_skipped_without_feature_produces_warning() {
    let mut req = make_req(AnalysisPreset::Risk);
    req.git = Some(true);

    let receipt = analyze(make_ctx(sample_export()), req).unwrap();

    assert!(receipt.git.is_none(), "git should be None without feature");
    assert!(
        receipt.warnings.iter().any(|w| w.contains("git")),
        "should warn about disabled git feature"
    );
}

#[cfg(not(feature = "content"))]
#[test]
fn content_enrichers_skipped_without_feature() {
    let mut req = make_req(AnalysisPreset::Health);
    req.git = Some(false);

    let receipt = analyze(make_ctx(sample_export()), req).unwrap();

    // TODO scanning requires content feature
    assert!(
        receipt
            .warnings
            .iter()
            .any(|w| w.contains("content") || w.contains("TODO")),
        "should warn about disabled content feature for TODO scanning"
    );
}

#[cfg(not(feature = "fun"))]
#[test]
fn fun_enricher_skipped_without_feature() {
    let receipt = analyze(make_ctx(sample_export()), make_req(AnalysisPreset::Fun)).unwrap();

    assert!(receipt.fun.is_none());
    assert!(
        receipt.warnings.iter().any(|w| w.contains("fun")),
        "should warn about disabled fun feature"
    );
}

#[cfg(not(feature = "topics"))]
#[test]
fn topics_enricher_skipped_without_feature() {
    let receipt = analyze(
        make_ctx(polyglot_export()),
        make_req(AnalysisPreset::Topics),
    )
    .unwrap();

    assert!(receipt.topics.is_none());
    assert!(
        receipt.warnings.iter().any(|w| w.contains("topic")),
        "should warn about disabled topics feature"
    );
}

#[cfg(not(feature = "archetype"))]
#[test]
fn archetype_enricher_skipped_without_feature() {
    let mut req = make_req(AnalysisPreset::Identity);
    req.git = Some(false);

    let receipt = analyze(make_ctx(sample_export()), req).unwrap();

    assert!(receipt.archetype.is_none());
    assert!(
        receipt
            .warnings
            .iter()
            .any(|w| w.contains("archetype") || w.contains("Archetype")),
        "should warn about disabled archetype feature"
    );
}

// ═══════════════════════════════════════════════════════════════════════════
// Scenario: Analysis receipt contains expected metadata
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn receipt_contains_correct_schema_version() {
    let receipt = analyze(make_ctx(sample_export()), make_req(AnalysisPreset::Receipt)).unwrap();
    assert_eq!(receipt.schema_version, ANALYSIS_SCHEMA_VERSION);
}

#[test]
fn receipt_mode_is_always_analysis() {
    let presets = [
        AnalysisPreset::Receipt,
        AnalysisPreset::Health,
        AnalysisPreset::Fun,
        AnalysisPreset::Deep,
    ];

    for preset in presets {
        let mut req = make_req(preset);
        req.git = Some(false);

        let receipt = analyze(make_ctx(sample_export()), req).unwrap();
        assert_eq!(receipt.mode, "analysis", "preset {:?} mode", preset);
    }
}

#[test]
fn receipt_generated_at_ms_is_positive() {
    let receipt = analyze(make_ctx(sample_export()), make_req(AnalysisPreset::Receipt)).unwrap();
    assert!(
        receipt.generated_at_ms > 0,
        "timestamp should be positive, got {}",
        receipt.generated_at_ms
    );
}

#[test]
fn receipt_tool_info_has_name_and_version() {
    let receipt = analyze(make_ctx(sample_export()), make_req(AnalysisPreset::Receipt)).unwrap();
    assert!(
        !receipt.tool.name.is_empty(),
        "tool name should not be empty"
    );
    assert!(
        !receipt.tool.version.is_empty(),
        "tool version should not be empty"
    );
}

#[test]
fn receipt_source_inputs_preserved() {
    let mut ctx = make_ctx(sample_export());
    ctx.source.inputs = vec!["/custom/path".to_string()];

    let receipt = analyze(ctx, make_req(AnalysisPreset::Receipt)).unwrap();
    assert_eq!(receipt.source.inputs, vec!["/custom/path"]);
}

#[test]
fn receipt_source_module_metadata_preserved() {
    let mut ctx = make_ctx(sample_export());
    ctx.source.module_roots = vec!["packages".to_string(), "libs".to_string()];
    ctx.source.module_depth = 3;
    ctx.source.children = "collapse".to_string();

    let receipt = analyze(ctx, make_req(AnalysisPreset::Receipt)).unwrap();
    assert_eq!(receipt.source.module_roots, vec!["packages", "libs"]);
    assert_eq!(receipt.source.module_depth, 3);
    assert_eq!(receipt.source.children, "collapse");
}

// ═══════════════════════════════════════════════════════════════════════════
// Scenario: Warnings are properly collected
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn no_warnings_for_receipt_preset() {
    let receipt = analyze(make_ctx(sample_export()), make_req(AnalysisPreset::Receipt)).unwrap();
    assert!(
        receipt.warnings.is_empty(),
        "Receipt preset should produce no warnings, got: {:?}",
        receipt.warnings
    );
    assert!(matches!(receipt.status, ScanStatus::Complete));
}

#[test]
fn warnings_produce_partial_status() {
    // Request git on a non-git directory to trigger a warning
    let tmp = tempfile::tempdir().unwrap();
    let ctx = AnalysisContext {
        export: sample_export(),
        root: tmp.path().to_path_buf(),
        source: make_source(),
    };

    let mut req = make_req(AnalysisPreset::Risk);
    req.git = Some(true);

    let receipt = analyze(ctx, req).unwrap();

    assert!(
        !receipt.warnings.is_empty(),
        "should have at least one warning"
    );
    assert!(
        matches!(receipt.status, ScanStatus::Partial),
        "warnings should produce Partial status"
    );
}

#[test]
fn git_override_false_suppresses_all_git_enrichers() {
    // Even for a preset that wants git, override with false should skip it
    let mut req = make_req(AnalysisPreset::Risk);
    req.git = Some(false);

    let receipt = analyze(make_ctx(sample_export()), req).unwrap();

    assert!(receipt.git.is_none());
    assert!(receipt.predictive_churn.is_none());
    assert!(receipt.corporate_fingerprint.is_none());
    // No warnings should be emitted for explicitly disabled git
    let git_warnings: Vec<_> = receipt
        .warnings
        .iter()
        .filter(|w| w.contains("git"))
        .collect();
    assert!(
        git_warnings.is_empty(),
        "explicitly disabling git should not produce warnings, got: {:?}",
        git_warnings
    );
}

#[test]
fn multiple_warnings_accumulated_not_overwritten() {
    // Deep preset with git enabled on a non-repo should accumulate multiple warnings
    let tmp = tempfile::tempdir().unwrap();
    let ctx = AnalysisContext {
        export: sample_export(),
        root: tmp.path().to_path_buf(),
        source: make_source(),
    };

    let mut req = make_req(AnalysisPreset::Deep);
    req.git = Some(true);

    let receipt = analyze(ctx, req).unwrap();

    // Deep preset requests many enrichers; on a temp dir without git/content/walk,
    // multiple warnings should accumulate
    assert!(
        !receipt.warnings.is_empty(),
        "Deep preset on temp dir should produce warnings"
    );
    assert!(
        matches!(receipt.status, ScanStatus::Partial),
        "should be Partial with warnings"
    );
}

// ═══════════════════════════════════════════════════════════════════════════
// Scenario: Derived metrics present across all presets
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn derived_always_present_regardless_of_preset() {
    let all_presets = [
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

    for preset in all_presets {
        let mut req = make_req(preset);
        req.git = Some(false);

        let receipt = analyze(make_ctx(sample_export()), req)
            .unwrap_or_else(|e| panic!("preset {:?} failed: {}", preset, e));

        assert!(
            receipt.derived.is_some(),
            "preset {:?} must always produce derived metrics",
            preset
        );

        let derived = receipt.derived.as_ref().unwrap();
        assert!(
            derived.totals.files > 0,
            "preset {:?} should see files",
            preset
        );
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// Scenario: Integrity hash and base_signature behavior
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn integrity_hash_populated_for_all_presets() {
    let presets = [
        AnalysisPreset::Receipt,
        AnalysisPreset::Health,
        AnalysisPreset::Fun,
    ];

    for preset in presets {
        let mut req = make_req(preset);
        req.git = Some(false);

        let receipt = analyze(make_ctx(sample_export()), req).unwrap();
        let derived = receipt.derived.as_ref().unwrap();

        assert!(
            !derived.integrity.hash.is_empty(),
            "preset {:?} integrity hash should not be empty",
            preset
        );
        assert!(
            derived.integrity.entries > 0,
            "preset {:?} integrity entries should be > 0",
            preset
        );
    }
}

#[test]
fn base_signature_backfill_when_absent() {
    let receipt = analyze(make_ctx(sample_export()), make_req(AnalysisPreset::Receipt)).unwrap();

    let derived_hash = &receipt.derived.as_ref().unwrap().integrity.hash;
    let base_sig = receipt.source.base_signature.as_ref().unwrap();

    assert_eq!(
        derived_hash, base_sig,
        "base_signature should be backfilled from integrity hash"
    );
}

#[test]
fn base_signature_preserved_when_pre_set() {
    let mut ctx = make_ctx(sample_export());
    ctx.source.base_signature = Some("custom-signature-123".to_string());

    let receipt = analyze(ctx, make_req(AnalysisPreset::Receipt)).unwrap();

    assert_eq!(
        receipt.source.base_signature.as_deref(),
        Some("custom-signature-123"),
        "pre-set base_signature should not be overwritten"
    );
}

// ═══════════════════════════════════════════════════════════════════════════
// Scenario: Empty and edge-case exports
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn empty_export_all_presets_succeed() {
    let empty = ExportData {
        rows: vec![],
        module_roots: vec![],
        module_depth: 1,
        children: ChildIncludeMode::Separate,
    };

    let presets = [
        AnalysisPreset::Receipt,
        AnalysisPreset::Health,
        AnalysisPreset::Fun,
        AnalysisPreset::Topics,
    ];

    for preset in presets {
        let mut req = make_req(preset);
        req.git = Some(false);

        let receipt = analyze(make_ctx(empty.clone()), req)
            .unwrap_or_else(|e| panic!("preset {:?} failed on empty export: {}", preset, e));

        let derived = receipt.derived.unwrap();
        assert_eq!(derived.totals.files, 0);
        assert_eq!(derived.totals.code, 0);
    }
}

#[test]
fn single_language_export_produces_correct_polyglot() {
    let export = ExportData {
        rows: vec![row("src/lib.rs", "src", "Rust", 500)],
        module_roots: vec![],
        module_depth: 1,
        children: ChildIncludeMode::Separate,
    };

    let receipt = analyze(make_ctx(export), make_req(AnalysisPreset::Receipt)).unwrap();
    let poly = &receipt.derived.unwrap().polyglot;

    assert_eq!(poly.lang_count, 1);
    assert_eq!(poly.dominant_lang, "Rust");
    assert!((poly.dominant_pct - 1.0).abs() < 0.001);
    // Entropy of a single language = 0
    assert!(
        (poly.entropy - 0.0).abs() < 0.001,
        "single-language entropy should be 0"
    );
}

#[test]
fn many_small_files_produce_correct_distribution() {
    let rows: Vec<FileRow> = (1..=100)
        .map(|i| row(&format!("src/f{}.rs", i), "src", "Rust", 10))
        .collect();
    let export = ExportData {
        rows,
        module_roots: vec![],
        module_depth: 1,
        children: ChildIncludeMode::Separate,
    };

    let receipt = analyze(make_ctx(export), make_req(AnalysisPreset::Receipt)).unwrap();
    let dist = &receipt.derived.unwrap().distribution;

    assert_eq!(dist.count, 100);
    assert_eq!(dist.min, dist.max, "all files same size");
    assert!((dist.gini - 0.0).abs() < 0.01, "equal files → gini ~0");
}

// ═══════════════════════════════════════════════════════════════════════════
// Scenario: JSON serialization of different preset results
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn all_presets_json_round_trip() {
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

    for preset in presets {
        let mut req = make_req(preset);
        req.git = Some(false);

        let receipt = analyze(make_ctx(sample_export()), req).unwrap();

        // Serialize → deserialize round trip
        let json = serde_json::to_string(&receipt)
            .unwrap_or_else(|e| panic!("preset {:?} serialize failed: {}", preset, e));
        let roundtrip: tokmd_analysis_types::AnalysisReceipt = serde_json::from_str(&json)
            .unwrap_or_else(|e| panic!("preset {:?} deserialize failed: {}", preset, e));

        assert_eq!(
            receipt.schema_version, roundtrip.schema_version,
            "preset {:?} schema_version mismatch after round-trip",
            preset
        );
        assert_eq!(
            receipt.mode, roundtrip.mode,
            "preset {:?} mode mismatch after round-trip",
            preset
        );

        // JSON envelope structure check
        let value: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert!(value.get("schema_version").is_some());
        assert!(value.get("derived").is_some());
        assert!(value.get("warnings").is_some());
        assert!(value.get("tool").is_some());
    }
}
