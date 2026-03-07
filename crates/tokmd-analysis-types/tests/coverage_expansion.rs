//! Additional coverage tests for tokmd-analysis-types.
//!
//! Focuses on compound report roundtrips, deterministic serialization,
//! CommitIntentCounts logic, enum variants, and populated optional fields.

use std::collections::BTreeMap;
use tokmd_analysis_types::*;
use tokmd_types::{ScanStatus, ToolInfo};

// =============================================================================
// Helpers
// =============================================================================

fn sample_tool() -> ToolInfo {
    ToolInfo {
        name: "tokmd".into(),
        version: "0.0.0-test".into(),
    }
}

fn sample_source() -> AnalysisSource {
    AnalysisSource {
        inputs: vec![".".into()],
        export_path: None,
        base_receipt_path: None,
        export_schema_version: None,
        export_generated_at_ms: None,
        base_signature: None,
        module_roots: vec!["crates".into()],
        module_depth: 2,
        children: "separate".into(),
    }
}

fn sample_args() -> AnalysisArgsMeta {
    AnalysisArgsMeta {
        preset: "receipt".into(),
        format: "json".into(),
        window_tokens: None,
        git: None,
        max_files: None,
        max_bytes: None,
        max_commits: None,
        max_commit_files: None,
        max_file_bytes: None,
        import_granularity: "module".into(),
        effort_model: None,
        effort_layer: None,
        base_ref: None,
        head_ref: None,
        monte_carlo: None,
        mc_iterations: None,
        mc_seed: None,
    }
}

fn minimal_receipt() -> AnalysisReceipt {
    AnalysisReceipt {
        schema_version: ANALYSIS_SCHEMA_VERSION,
        generated_at_ms: 1700000000000,
        tool: sample_tool(),
        mode: "analyze".into(),
        status: ScanStatus::Complete,
        warnings: vec![],
        source: sample_source(),
        args: sample_args(),
        archetype: None,
        topics: None,
        entropy: None,
        predictive_churn: None,
        corporate_fingerprint: None,
        license: None,
        derived: None,
        assets: None,
        deps: None,
        git: None,
        imports: None,
        dup: None,
        complexity: None,
        api_surface: None,
        fun: None,
        effort: None,
    }
}

// =============================================================================
// AnalysisReceipt with populated archetype
// =============================================================================

#[test]
fn receipt_with_archetype_roundtrip() {
    let mut receipt = minimal_receipt();
    receipt.archetype = Some(Archetype {
        kind: "cli_tool".into(),
        evidence: vec!["clap dependency".into(), "main.rs entrypoint".into()],
    });

    let json = serde_json::to_string(&receipt).unwrap();
    let back: AnalysisReceipt = serde_json::from_str(&json).unwrap();

    let arch = back.archetype.unwrap();
    assert_eq!(arch.kind, "cli_tool");
    assert_eq!(arch.evidence.len(), 2);
}

// =============================================================================
// AnalysisReceipt double roundtrip stability
// =============================================================================

#[test]
fn receipt_double_roundtrip_is_stable() {
    let receipt = minimal_receipt();
    let json1 = serde_json::to_string(&receipt).unwrap();
    let mid: AnalysisReceipt = serde_json::from_str(&json1).unwrap();
    let json2 = serde_json::to_string(&mid).unwrap();
    assert_eq!(json1, json2, "Double roundtrip must produce identical JSON");
}

// =============================================================================
// FunReport roundtrip
// =============================================================================

#[test]
fn fun_report_with_eco_label_roundtrip() {
    let fun = FunReport {
        eco_label: Some(EcoLabel {
            score: 85.5,
            label: "A".into(),
            bytes: 1_234_567,
            notes: "Small, efficient codebase".into(),
        }),
    };
    let json = serde_json::to_string(&fun).unwrap();
    let back: FunReport = serde_json::from_str(&json).unwrap();
    let eco = back.eco_label.unwrap();
    assert!((eco.score - 85.5).abs() < f64::EPSILON);
    assert_eq!(eco.label, "A");
    assert_eq!(eco.bytes, 1_234_567);
}

#[test]
fn fun_report_none_eco_roundtrip() {
    let fun = FunReport { eco_label: None };
    let json = serde_json::to_string(&fun).unwrap();
    let back: FunReport = serde_json::from_str(&json).unwrap();
    assert!(back.eco_label.is_none());
}

// =============================================================================
// CommitIntentCounts increment
// =============================================================================

#[test]
fn commit_intent_counts_increment_all_kinds() {
    let mut counts = CommitIntentCounts::default();
    assert_eq!(counts.total, 0);

    counts.increment(CommitIntentKind::Feat);
    counts.increment(CommitIntentKind::Fix);
    counts.increment(CommitIntentKind::Refactor);
    counts.increment(CommitIntentKind::Docs);
    counts.increment(CommitIntentKind::Test);
    counts.increment(CommitIntentKind::Chore);
    counts.increment(CommitIntentKind::Ci);
    counts.increment(CommitIntentKind::Build);
    counts.increment(CommitIntentKind::Perf);
    counts.increment(CommitIntentKind::Style);
    counts.increment(CommitIntentKind::Revert);
    counts.increment(CommitIntentKind::Other);

    assert_eq!(counts.feat, 1);
    assert_eq!(counts.fix, 1);
    assert_eq!(counts.refactor, 1);
    assert_eq!(counts.docs, 1);
    assert_eq!(counts.test, 1);
    assert_eq!(counts.chore, 1);
    assert_eq!(counts.ci, 1);
    assert_eq!(counts.build, 1);
    assert_eq!(counts.perf, 1);
    assert_eq!(counts.style, 1);
    assert_eq!(counts.revert, 1);
    assert_eq!(counts.other, 1);
    assert_eq!(counts.total, 12);
}

#[test]
fn commit_intent_counts_default_is_zeroed() {
    let counts = CommitIntentCounts::default();
    assert_eq!(counts.feat, 0);
    assert_eq!(counts.fix, 0);
    assert_eq!(counts.total, 0);
}

// =============================================================================
// NearDupScope enum roundtrip
// =============================================================================

#[test]
fn near_dup_scope_all_variants_roundtrip() {
    for (variant, expected_json) in [
        (NearDupScope::Module, "\"module\""),
        (NearDupScope::Lang, "\"lang\""),
        (NearDupScope::Global, "\"global\""),
    ] {
        let json = serde_json::to_string(&variant).unwrap();
        assert_eq!(json, expected_json, "NearDupScope serializes to kebab-case");
        let back: NearDupScope = serde_json::from_str(&json).unwrap();
        assert_eq!(back, variant);
    }
}

#[test]
fn near_dup_scope_default_is_module() {
    assert_eq!(NearDupScope::default(), NearDupScope::Module);
}

// =============================================================================
// ComplexityRisk and TechnicalDebtLevel enum roundtrips
// =============================================================================

#[test]
fn complexity_risk_all_variants_snake_case() {
    for (variant, expected) in [
        (ComplexityRisk::Low, "\"low\""),
        (ComplexityRisk::Moderate, "\"moderate\""),
        (ComplexityRisk::High, "\"high\""),
        (ComplexityRisk::Critical, "\"critical\""),
    ] {
        let json = serde_json::to_string(&variant).unwrap();
        assert_eq!(json, expected);
        let back: ComplexityRisk = serde_json::from_str(&json).unwrap();
        assert_eq!(back, variant);
    }
}

#[test]
fn technical_debt_level_all_variants_snake_case() {
    for (variant, expected) in [
        (TechnicalDebtLevel::Low, "\"low\""),
        (TechnicalDebtLevel::Moderate, "\"moderate\""),
        (TechnicalDebtLevel::High, "\"high\""),
        (TechnicalDebtLevel::Critical, "\"critical\""),
    ] {
        let json = serde_json::to_string(&variant).unwrap();
        assert_eq!(json, expected);
        let back: TechnicalDebtLevel = serde_json::from_str(&json).unwrap();
        assert_eq!(back, variant);
    }
}

// =============================================================================
// TopicClouds deterministic serialization (BTreeMap key ordering)
// =============================================================================

#[test]
fn topic_clouds_btreemap_keys_are_sorted() {
    let mut per_module = BTreeMap::new();
    per_module.insert(
        "zebra".into(),
        vec![TopicTerm {
            term: "stripes".into(),
            score: 0.9,
            tf: 5,
            df: 1,
        }],
    );
    per_module.insert(
        "alpha".into(),
        vec![TopicTerm {
            term: "first".into(),
            score: 0.8,
            tf: 3,
            df: 2,
        }],
    );
    let clouds = TopicClouds {
        per_module,
        overall: vec![],
    };

    let json = serde_json::to_string(&clouds).unwrap();
    let alpha_pos = json.find("\"alpha\"").unwrap();
    let zebra_pos = json.find("\"zebra\"").unwrap();
    assert!(
        alpha_pos < zebra_pos,
        "BTreeMap keys must appear sorted in JSON"
    );

    // Double roundtrip stability
    let back: TopicClouds = serde_json::from_str(&json).unwrap();
    let json2 = serde_json::to_string(&back).unwrap();
    assert_eq!(json, json2);
}

// =============================================================================
// ApiSurfaceReport roundtrip
// =============================================================================

#[test]
fn api_surface_report_roundtrip() {
    let mut by_language = BTreeMap::new();
    by_language.insert(
        "Rust".into(),
        LangApiSurface {
            total_items: 100,
            public_items: 30,
            internal_items: 70,
            public_ratio: 0.3,
        },
    );

    let report = ApiSurfaceReport {
        total_items: 100,
        public_items: 30,
        internal_items: 70,
        public_ratio: 0.3,
        documented_ratio: 0.8,
        by_language,
        by_module: vec![ModuleApiRow {
            module: "core".into(),
            total_items: 50,
            public_items: 15,
            public_ratio: 0.3,
        }],
        top_exporters: vec![ApiExportItem {
            path: "src/lib.rs".into(),
            lang: "Rust".into(),
            public_items: 10,
            total_items: 20,
        }],
    };

    let json = serde_json::to_string(&report).unwrap();
    let back: ApiSurfaceReport = serde_json::from_str(&json).unwrap();
    assert_eq!(back.total_items, 100);
    assert_eq!(back.public_items, 30);
    assert!((back.documented_ratio - 0.8).abs() < f64::EPSILON);
    assert_eq!(back.by_language.len(), 1);
    assert_eq!(back.by_module.len(), 1);
    assert_eq!(back.top_exporters.len(), 1);
    assert_eq!(back.top_exporters[0].path, "src/lib.rs");
}

// =============================================================================
// ComplexityReport minimal roundtrip
// =============================================================================

#[test]
fn complexity_report_minimal_roundtrip() {
    let report = ComplexityReport {
        total_functions: 10,
        avg_function_length: 15.5,
        max_function_length: 50,
        avg_cyclomatic: 3.2,
        max_cyclomatic: 12,
        avg_cognitive: None,
        max_cognitive: None,
        avg_nesting_depth: None,
        max_nesting_depth: None,
        high_risk_files: 1,
        histogram: None,
        halstead: None,
        maintainability_index: None,
        technical_debt: None,
        files: vec![FileComplexity {
            path: "src/main.rs".into(),
            module: "src".into(),
            function_count: 5,
            max_function_length: 50,
            cyclomatic_complexity: 12,
            cognitive_complexity: None,
            max_nesting: None,
            risk_level: ComplexityRisk::High,
            functions: None,
        }],
    };

    let json = serde_json::to_string(&report).unwrap();
    let back: ComplexityReport = serde_json::from_str(&json).unwrap();
    assert_eq!(back.total_functions, 10);
    assert!((back.avg_cyclomatic - 3.2).abs() < f64::EPSILON);
    assert_eq!(back.files.len(), 1);
    assert_eq!(back.files[0].risk_level, ComplexityRisk::High);
    assert!(back.avg_cognitive.is_none());
    assert!(back.histogram.is_none());
}

// =============================================================================
// GitReport roundtrip
// =============================================================================

#[test]
fn git_report_roundtrip() {
    let report = GitReport {
        commits_scanned: 500,
        files_seen: 120,
        hotspots: vec![HotspotRow {
            path: "src/lib.rs".into(),
            commits: 45,
            lines: 300,
            score: 13500,
        }],
        bus_factor: vec![BusFactorRow {
            module: "core".into(),
            authors: 3,
        }],
        freshness: FreshnessReport {
            threshold_days: 90,
            stale_files: 5,
            total_files: 50,
            stale_pct: 10.0,
            by_module: vec![],
        },
        coupling: vec![CouplingRow {
            left: "src/a.rs".into(),
            right: "src/b.rs".into(),
            count: 12,
            jaccard: Some(0.4),
            lift: Some(1.5),
            n_left: Some(20),
            n_right: Some(15),
        }],
        age_distribution: None,
        intent: None,
    };

    let json = serde_json::to_string(&report).unwrap();
    let back: GitReport = serde_json::from_str(&json).unwrap();
    assert_eq!(back.commits_scanned, 500);
    assert_eq!(back.hotspots.len(), 1);
    assert_eq!(back.hotspots[0].score, 13500);
    assert_eq!(back.bus_factor[0].authors, 3);
    assert!((back.freshness.stale_pct - 10.0).abs() < f64::EPSILON);
    assert!(back.coupling[0].jaccard.is_some());
    assert!(back.age_distribution.is_none());
    assert!(back.intent.is_none());
}

// =============================================================================
// DuplicateReport roundtrip
// =============================================================================

#[test]
fn duplicate_report_roundtrip() {
    let report = DuplicateReport {
        groups: vec![DuplicateGroup {
            hash: "abc123".into(),
            bytes: 1024,
            files: vec!["a.rs".into(), "b.rs".into()],
        }],
        wasted_bytes: 1024,
        strategy: "blake3".into(),
        density: Some(DuplicationDensityReport {
            duplicate_groups: 1,
            duplicate_files: 2,
            duplicated_bytes: 2048,
            wasted_bytes: 1024,
            wasted_pct_of_codebase: 5.0,
            by_module: vec![],
        }),
        near: None,
    };

    let json = serde_json::to_string(&report).unwrap();
    let back: DuplicateReport = serde_json::from_str(&json).unwrap();
    assert_eq!(back.groups.len(), 1);
    assert_eq!(back.wasted_bytes, 1024);
    assert_eq!(back.strategy, "blake3");
    let density = back.density.unwrap();
    assert_eq!(density.duplicate_groups, 1);
    assert!((density.wasted_pct_of_codebase - 5.0).abs() < f64::EPSILON);
}

// =============================================================================
// AnalysisSource with all optional fields populated
// =============================================================================

#[test]
fn analysis_source_all_fields_roundtrip() {
    let source = AnalysisSource {
        inputs: vec!["src".into(), "lib".into()],
        export_path: Some("export.jsonl".into()),
        base_receipt_path: Some("base.json".into()),
        export_schema_version: Some(2),
        export_generated_at_ms: Some(1700000000000),
        base_signature: Some("abc123def456".into()),
        module_roots: vec!["crates".into()],
        module_depth: 2,
        children: "collapse".into(),
    };

    let json = serde_json::to_string(&source).unwrap();
    let back: AnalysisSource = serde_json::from_str(&json).unwrap();
    assert_eq!(back.export_path.as_deref(), Some("export.jsonl"));
    assert_eq!(back.base_receipt_path.as_deref(), Some("base.json"));
    assert_eq!(back.export_schema_version, Some(2));
    assert_eq!(back.base_signature.as_deref(), Some("abc123def456"));
}

// =============================================================================
// AnalysisArgsMeta with all optional fields
// =============================================================================

#[test]
fn analysis_args_meta_all_fields_roundtrip() {
    let args = AnalysisArgsMeta {
        preset: "deep".into(),
        format: "json".into(),
        window_tokens: Some(128000),
        git: Some(true),
        max_files: Some(1000),
        max_bytes: Some(50_000_000),
        max_commits: Some(5000),
        max_commit_files: Some(100),
        max_file_bytes: Some(500_000),
        import_granularity: "file".into(),
        effort_model: None,
        effort_layer: None,
        base_ref: None,
        head_ref: None,
        monte_carlo: None,
        mc_iterations: None,
        mc_seed: None,
    };

    let json = serde_json::to_string(&args).unwrap();
    let back: AnalysisArgsMeta = serde_json::from_str(&json).unwrap();
    assert_eq!(back.preset, "deep");
    assert_eq!(back.window_tokens, Some(128000));
    assert_eq!(back.git, Some(true));
    assert_eq!(back.max_files, Some(1000));
    assert_eq!(back.max_bytes, Some(50_000_000));
    assert_eq!(back.max_file_bytes, Some(500_000));
}

// =============================================================================
// ComplexityHistogram edge cases
// =============================================================================

#[test]
fn complexity_histogram_empty_counts() {
    let h = ComplexityHistogram {
        buckets: vec![0, 5, 10],
        counts: vec![0, 0, 0],
        total: 0,
    };
    let ascii = h.to_ascii(20);
    assert!(ascii.contains("0-4"));
    assert!(ascii.contains("5-9"));
    assert!(ascii.contains("10+"));
}

#[test]
fn complexity_histogram_single_bucket() {
    let h = ComplexityHistogram {
        buckets: vec![0],
        counts: vec![42],
        total: 42,
    };
    let ascii = h.to_ascii(10);
    assert!(ascii.contains("42"));
}

// =============================================================================
// ComplexityBaseline serde roundtrip
// =============================================================================

#[test]
fn complexity_baseline_serde_roundtrip() {
    let baseline = ComplexityBaseline {
        baseline_version: BASELINE_VERSION,
        generated_at: "2024-01-15T10:30:00.000Z".into(),
        commit: Some("abc123".into()),
        metrics: BaselineMetrics {
            total_code_lines: 5000,
            total_files: 50,
            avg_cyclomatic: 3.5,
            max_cyclomatic: 15,
            avg_cognitive: 2.0,
            max_cognitive: 8,
            avg_nesting_depth: 1.5,
            max_nesting_depth: 4,
            function_count: 200,
            avg_function_length: 20.0,
        },
        files: vec![FileBaselineEntry {
            path: "src/lib.rs".into(),
            code_lines: 100,
            cyclomatic: 5,
            cognitive: 3,
            max_nesting: 2,
            function_count: 10,
            content_hash: Some("deadbeef".into()),
        }],
        complexity: None,
        determinism: None,
    };

    let json = serde_json::to_string(&baseline).unwrap();
    let back: ComplexityBaseline = serde_json::from_str(&json).unwrap();
    assert_eq!(back.baseline_version, BASELINE_VERSION);
    assert_eq!(back.commit.as_deref(), Some("abc123"));
    assert_eq!(back.metrics.total_code_lines, 5000);
    assert_eq!(back.files.len(), 1);
    assert_eq!(back.files[0].content_hash.as_deref(), Some("deadbeef"));
}
