//! Depth tests for tokmd-analysis-types: receipt construction, serialization, schema, presets.

use std::collections::BTreeMap;

use tokmd_analysis_types::*;
use tokmd_types::{ScanStatus, ToolInfo};

// ──────────────────────────────────────────────────────────────────────
// Helpers
// ──────────────────────────────────────────────────────────────────────

fn minimal_receipt() -> AnalysisReceipt {
    AnalysisReceipt {
        schema_version: ANALYSIS_SCHEMA_VERSION,
        generated_at_ms: 1_700_000_000_000,
        tool: ToolInfo {
            name: "tokmd".to_string(),
            version: "1.0.0".to_string(),
        },
        mode: "receipt".to_string(),
        status: ScanStatus::Complete,
        warnings: vec![],
        source: AnalysisSource {
            inputs: vec![".".to_string()],
            export_path: None,
            base_receipt_path: None,
            export_schema_version: None,
            export_generated_at_ms: None,
            base_signature: None,
            module_roots: vec![],
            module_depth: 2,
            children: "parents-only".to_string(),
        },
        args: AnalysisArgsMeta {
            preset: "receipt".to_string(),
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
    }
}

fn receipt_with_derived() -> AnalysisReceipt {
    let mut r = minimal_receipt();
    r.derived = Some(DerivedReport {
        totals: DerivedTotals {
            files: 10,
            code: 500,
            comments: 100,
            blanks: 50,
            lines: 650,
            bytes: 20_000,
            tokens: 3_000,
        },
        doc_density: ratio_report("total", 100, 650),
        whitespace: ratio_report("total", 50, 650),
        verbosity: rate_report("total", 20_000, 650),
        max_file: MaxFileReport {
            overall: file_stat_row("big.rs"),
            by_lang: vec![],
            by_module: vec![],
        },
        lang_purity: LangPurityReport { rows: vec![] },
        nesting: NestingReport {
            max: 3,
            avg: 1.5,
            by_module: vec![],
        },
        test_density: TestDensityReport {
            test_lines: 100,
            prod_lines: 400,
            test_files: 2,
            prod_files: 8,
            ratio: 0.25,
        },
        boilerplate: BoilerplateReport {
            infra_lines: 50,
            logic_lines: 450,
            ratio: 0.1,
            infra_langs: vec!["TOML".to_string()],
        },
        polyglot: PolyglotReport {
            lang_count: 2,
            entropy: 0.8,
            dominant_lang: "Rust".to_string(),
            dominant_lines: 400,
            dominant_pct: 0.8,
        },
        distribution: DistributionReport {
            count: 10,
            min: 5,
            max: 200,
            mean: 50.0,
            median: 45.0,
            p90: 150.0,
            p99: 195.0,
            gini: 0.3,
        },
        histogram: vec![],
        top: TopOffenders {
            largest_lines: vec![],
            largest_tokens: vec![],
            largest_bytes: vec![],
            least_documented: vec![],
            most_dense: vec![],
        },
        tree: None,
        reading_time: ReadingTimeReport {
            minutes: 5.0,
            lines_per_minute: 130,
            basis_lines: 650,
        },
        context_window: None,
        cocomo: None,
        todo: None,
        integrity: IntegrityReport {
            algo: "blake3".to_string(),
            hash: "abc123".to_string(),
            entries: 10,
        },
    });
    r
}

fn ratio_report(key: &str, num: usize, den: usize) -> RatioReport {
    RatioReport {
        total: RatioRow {
            key: key.to_string(),
            numerator: num,
            denominator: den,
            ratio: if den > 0 {
                num as f64 / den as f64
            } else {
                0.0
            },
        },
        by_lang: vec![],
        by_module: vec![],
    }
}

fn rate_report(key: &str, num: usize, den: usize) -> RateReport {
    RateReport {
        total: RateRow {
            key: key.to_string(),
            numerator: num,
            denominator: den,
            rate: if den > 0 {
                num as f64 / den as f64
            } else {
                0.0
            },
        },
        by_lang: vec![],
        by_module: vec![],
    }
}

fn file_stat_row(path: &str) -> FileStatRow {
    FileStatRow {
        path: path.to_string(),
        module: "src".to_string(),
        lang: "Rust".to_string(),
        code: 200,
        comments: 50,
        blanks: 20,
        lines: 270,
        bytes: 8000,
        tokens: 1500,
        doc_pct: Some(0.18),
        bytes_per_line: Some(29.6),
        depth: 1,
    }
}

// ──────────────────────────────────────────────────────────────────────
// 1. AnalysisReceipt construction and serialization
// ──────────────────────────────────────────────────────────────────────

#[test]
fn minimal_receipt_serializes_to_json() {
    let r = minimal_receipt();
    let json = serde_json::to_string(&r).unwrap();
    assert!(json.contains("\"schema_version\":8"));
    assert!(json.contains("\"mode\":\"receipt\""));
}

#[test]
fn minimal_receipt_json_roundtrip() {
    let r = minimal_receipt();
    let json = serde_json::to_string_pretty(&r).unwrap();
    let back: AnalysisReceipt = serde_json::from_str(&json).unwrap();
    assert_eq!(back.schema_version, r.schema_version);
    assert_eq!(back.mode, r.mode);
    assert_eq!(back.generated_at_ms, r.generated_at_ms);
}

#[test]
fn receipt_with_derived_roundtrip() {
    let r = receipt_with_derived();
    let json = serde_json::to_string(&r).unwrap();
    let back: AnalysisReceipt = serde_json::from_str(&json).unwrap();
    let derived = back.derived.unwrap();
    assert_eq!(derived.totals.files, 10);
    assert_eq!(derived.totals.code, 500);
    assert_eq!(derived.integrity.algo, "blake3");
}

#[test]
fn receipt_null_optional_fields_deserialize() {
    let json = serde_json::to_string(&minimal_receipt()).unwrap();
    let v: serde_json::Value = serde_json::from_str(&json).unwrap();
    assert!(v["archetype"].is_null());
    assert!(v["topics"].is_null());
    assert!(v["entropy"].is_null());
    assert!(v["git"].is_null());
    assert!(v["complexity"].is_null());
}

#[test]
fn receipt_source_fields_preserved() {
    let r = minimal_receipt();
    let json = serde_json::to_string(&r).unwrap();
    let back: AnalysisReceipt = serde_json::from_str(&json).unwrap();
    assert_eq!(back.source.inputs, vec!["."]);
    assert_eq!(back.source.module_depth, 2);
    assert_eq!(back.source.children, "parents-only");
}

// ──────────────────────────────────────────────────────────────────────
// 2. Schema version validation
// ──────────────────────────────────────────────────────────────────────

#[test]
fn analysis_schema_version_is_8() {
    assert_eq!(ANALYSIS_SCHEMA_VERSION, 8);
}

#[test]
fn baseline_version_is_1() {
    assert_eq!(BASELINE_VERSION, 1);
}

#[test]
fn envelope_schema_is_sensor_report_v1() {
    assert_eq!(ENVELOPE_SCHEMA, "sensor.report.v1");
}

#[test]
fn receipt_schema_version_matches_constant() {
    let r = minimal_receipt();
    assert_eq!(r.schema_version, ANALYSIS_SCHEMA_VERSION);
}

// ──────────────────────────────────────────────────────────────────────
// 3. Field presence/absence based on preset
// ──────────────────────────────────────────────────────────────────────

#[test]
fn minimal_receipt_has_no_optional_sections() {
    let r = minimal_receipt();
    assert!(r.archetype.is_none());
    assert!(r.topics.is_none());
    assert!(r.entropy.is_none());
    assert!(r.predictive_churn.is_none());
    assert!(r.corporate_fingerprint.is_none());
    assert!(r.license.is_none());
    assert!(r.derived.is_none());
    assert!(r.assets.is_none());
    assert!(r.deps.is_none());
    assert!(r.git.is_none());
    assert!(r.imports.is_none());
    assert!(r.dup.is_none());
    assert!(r.complexity.is_none());
    assert!(r.api_surface.is_none());
    assert!(r.fun.is_none());
}

#[test]
fn receipt_preset_stored_in_args() {
    let r = minimal_receipt();
    assert_eq!(r.args.preset, "receipt");
}

#[test]
fn receipt_with_archetype() {
    let mut r = minimal_receipt();
    r.archetype = Some(Archetype {
        kind: "web-app".to_string(),
        evidence: vec!["package.json".to_string(), "index.html".to_string()],
    });
    let json = serde_json::to_string(&r).unwrap();
    let back: AnalysisReceipt = serde_json::from_str(&json).unwrap();
    let arch = back.archetype.unwrap();
    assert_eq!(arch.kind, "web-app");
    assert_eq!(arch.evidence.len(), 2);
}

#[test]
fn receipt_with_complexity_report() {
    let mut r = minimal_receipt();
    r.complexity = Some(ComplexityReport {
        total_functions: 20,
        avg_function_length: 15.0,
        max_function_length: 80,
        avg_cyclomatic: 3.5,
        max_cyclomatic: 12,
        avg_cognitive: Some(4.2),
        max_cognitive: Some(15),
        avg_nesting_depth: Some(2.1),
        max_nesting_depth: Some(5),
        high_risk_files: 1,
        histogram: None,
        halstead: None,
        maintainability_index: None,
        technical_debt: None,
        files: vec![FileComplexity {
            path: "src/lib.rs".to_string(),
            module: "src".to_string(),
            function_count: 5,
            max_function_length: 40,
            cyclomatic_complexity: 8,
            cognitive_complexity: Some(10),
            max_nesting: Some(3),
            risk_level: ComplexityRisk::Moderate,
            functions: None,
        }],
    });
    let json = serde_json::to_string(&r).unwrap();
    let back: AnalysisReceipt = serde_json::from_str(&json).unwrap();
    let c = back.complexity.unwrap();
    assert_eq!(c.total_functions, 20);
    assert_eq!(c.files.len(), 1);
    assert_eq!(c.files[0].risk_level, ComplexityRisk::Moderate);
}

#[test]
fn receipt_with_git_report() {
    let mut r = minimal_receipt();
    r.git = Some(GitReport {
        commits_scanned: 100,
        files_seen: 50,
        hotspots: vec![HotspotRow {
            path: "src/lib.rs".to_string(),
            commits: 30,
            lines: 200,
            score: 6000,
        }],
        bus_factor: vec![],
        freshness: FreshnessReport {
            threshold_days: 90,
            stale_files: 5,
            total_files: 50,
            stale_pct: 10.0,
            by_module: vec![],
        },
        coupling: vec![],
        age_distribution: None,
        intent: None,
    });
    let json = serde_json::to_string(&r).unwrap();
    let back: AnalysisReceipt = serde_json::from_str(&json).unwrap();
    let git = back.git.unwrap();
    assert_eq!(git.commits_scanned, 100);
    assert_eq!(git.hotspots.len(), 1);
    assert_eq!(git.hotspots[0].score, 6000);
}

// ──────────────────────────────────────────────────────────────────────
// 4. Backward compatibility of type structures
// ──────────────────────────────────────────────────────────────────────

#[test]
fn commit_intent_counts_default_is_zeroed() {
    let c = CommitIntentCounts::default();
    assert_eq!(c.feat, 0);
    assert_eq!(c.fix, 0);
    assert_eq!(c.total, 0);
}

#[test]
fn commit_intent_counts_increment() {
    let mut c = CommitIntentCounts::default();
    c.increment(CommitIntentKind::Feat);
    c.increment(CommitIntentKind::Fix);
    c.increment(CommitIntentKind::Fix);
    assert_eq!(c.feat, 1);
    assert_eq!(c.fix, 2);
    assert_eq!(c.total, 3);
}

#[test]
fn commit_intent_all_kinds_increment() {
    let mut c = CommitIntentCounts::default();
    let kinds = [
        CommitIntentKind::Feat,
        CommitIntentKind::Fix,
        CommitIntentKind::Refactor,
        CommitIntentKind::Docs,
        CommitIntentKind::Test,
        CommitIntentKind::Chore,
        CommitIntentKind::Ci,
        CommitIntentKind::Build,
        CommitIntentKind::Perf,
        CommitIntentKind::Style,
        CommitIntentKind::Revert,
        CommitIntentKind::Other,
    ];
    for kind in kinds {
        c.increment(kind);
    }
    assert_eq!(c.total, 12);
}

#[test]
fn complexity_baseline_from_receipt_with_complexity() {
    let mut r = minimal_receipt();
    r.derived = Some(DerivedReport {
        totals: DerivedTotals {
            files: 5,
            code: 300,
            comments: 50,
            blanks: 25,
            lines: 375,
            bytes: 10_000,
            tokens: 2_000,
        },
        doc_density: ratio_report("total", 50, 375),
        whitespace: ratio_report("total", 25, 375),
        verbosity: rate_report("total", 10_000, 375),
        max_file: MaxFileReport {
            overall: file_stat_row("big.rs"),
            by_lang: vec![],
            by_module: vec![],
        },
        lang_purity: LangPurityReport { rows: vec![] },
        nesting: NestingReport {
            max: 2,
            avg: 1.0,
            by_module: vec![],
        },
        test_density: TestDensityReport {
            test_lines: 50,
            prod_lines: 250,
            test_files: 1,
            prod_files: 4,
            ratio: 0.2,
        },
        boilerplate: BoilerplateReport {
            infra_lines: 20,
            logic_lines: 280,
            ratio: 0.07,
            infra_langs: vec![],
        },
        polyglot: PolyglotReport {
            lang_count: 1,
            entropy: 0.0,
            dominant_lang: "Rust".to_string(),
            dominant_lines: 300,
            dominant_pct: 1.0,
        },
        distribution: DistributionReport {
            count: 5,
            min: 20,
            max: 100,
            mean: 60.0,
            median: 55.0,
            p90: 90.0,
            p99: 99.0,
            gini: 0.2,
        },
        histogram: vec![],
        top: TopOffenders {
            largest_lines: vec![],
            largest_tokens: vec![],
            largest_bytes: vec![],
            least_documented: vec![],
            most_dense: vec![],
        },
        tree: None,
        reading_time: ReadingTimeReport {
            minutes: 3.0,
            lines_per_minute: 130,
            basis_lines: 375,
        },
        context_window: None,
        cocomo: None,
        todo: None,
        integrity: IntegrityReport {
            algo: "blake3".to_string(),
            hash: "def456".to_string(),
            entries: 5,
        },
    });
    r.complexity = Some(ComplexityReport {
        total_functions: 15,
        avg_function_length: 20.0,
        max_function_length: 60,
        avg_cyclomatic: 4.0,
        max_cyclomatic: 10,
        avg_cognitive: Some(3.0),
        max_cognitive: Some(8),
        avg_nesting_depth: Some(1.5),
        max_nesting_depth: Some(4),
        high_risk_files: 0,
        histogram: None,
        halstead: None,
        maintainability_index: None,
        technical_debt: None,
        files: vec![FileComplexity {
            path: "src/lib.rs".to_string(),
            module: "src".to_string(),
            function_count: 5,
            max_function_length: 30,
            cyclomatic_complexity: 6,
            cognitive_complexity: Some(4),
            max_nesting: Some(2),
            risk_level: ComplexityRisk::Low,
            functions: None,
        }],
    });

    let baseline = ComplexityBaseline::from_analysis(&r);
    assert_eq!(baseline.baseline_version, BASELINE_VERSION);
    assert_eq!(baseline.metrics.total_code_lines, 300);
    assert_eq!(baseline.metrics.total_files, 5);
    assert_eq!(baseline.metrics.avg_cyclomatic, 4.0);
    assert_eq!(baseline.metrics.max_cyclomatic, 10);
    assert_eq!(baseline.files.len(), 1);
    assert!(baseline.complexity.is_some());
    let cs = baseline.complexity.unwrap();
    assert_eq!(cs.total_functions, 15);
    assert_eq!(cs.high_risk_files, 0);
}

#[test]
fn complexity_baseline_from_receipt_without_complexity() {
    let r = minimal_receipt();
    let baseline = ComplexityBaseline::from_analysis(&r);
    assert_eq!(baseline.metrics.total_code_lines, 0);
    assert!(baseline.files.is_empty());
    assert!(baseline.complexity.is_none());
}

#[test]
fn near_dup_scope_default_is_module() {
    assert_eq!(NearDupScope::default(), NearDupScope::Module);
}

#[test]
fn verdict_default_is_pass() {
    assert_eq!(Verdict::default(), Verdict::Pass);
}

// ──────────────────────────────────────────────────────────────────────
// 5. JSON round-trip fidelity
// ──────────────────────────────────────────────────────────────────────

#[test]
fn near_dup_scope_serde_roundtrip() {
    for variant in [
        NearDupScope::Module,
        NearDupScope::Lang,
        NearDupScope::Global,
    ] {
        let json = serde_json::to_string(&variant).unwrap();
        let back: NearDupScope = serde_json::from_str(&json).unwrap();
        assert_eq!(back, variant);
    }
}

#[test]
fn near_dup_scope_uses_kebab_case() {
    assert_eq!(
        serde_json::to_string(&NearDupScope::Module).unwrap(),
        "\"module\""
    );
    assert_eq!(
        serde_json::to_string(&NearDupScope::Lang).unwrap(),
        "\"lang\""
    );
    assert_eq!(
        serde_json::to_string(&NearDupScope::Global).unwrap(),
        "\"global\""
    );
}

#[test]
fn technical_debt_level_serde_roundtrip() {
    for variant in [
        TechnicalDebtLevel::Low,
        TechnicalDebtLevel::Moderate,
        TechnicalDebtLevel::High,
        TechnicalDebtLevel::Critical,
    ] {
        let json = serde_json::to_string(&variant).unwrap();
        let back: TechnicalDebtLevel = serde_json::from_str(&json).unwrap();
        assert_eq!(back, variant);
    }
}

#[test]
fn complexity_histogram_ascii_nonempty() {
    let h = ComplexityHistogram {
        buckets: vec![0, 5, 10, 15],
        counts: vec![20, 10, 5, 1],
        total: 36,
    };
    let ascii = h.to_ascii(30);
    assert_eq!(ascii.lines().count(), 4);
    assert!(ascii.contains("20"));
}

#[test]
fn full_receipt_json_roundtrip_fidelity() {
    let r = receipt_with_derived();
    let json = serde_json::to_string_pretty(&r).unwrap();
    let back: AnalysisReceipt = serde_json::from_str(&json).unwrap();
    // Verify structural fidelity: re-serialize the deserialized value and compare
    // via serde_json::Value to avoid floating-point repr differences.
    let v1: serde_json::Value = serde_json::from_str(&json).unwrap();
    let v2: serde_json::Value =
        serde_json::from_str(&serde_json::to_string_pretty(&back).unwrap()).unwrap();
    assert_eq!(v1, v2, "double roundtrip should be identical");
}

#[test]
fn receipt_json_keys_are_snake_case() {
    let r = minimal_receipt();
    let json = serde_json::to_string(&r).unwrap();
    assert!(json.contains("schema_version"));
    assert!(json.contains("generated_at_ms"));
    assert!(json.contains("module_depth"));
    assert!(!json.contains("schemaVersion"));
}

#[test]
fn scan_status_serde_roundtrip() {
    let json = serde_json::to_string(&ScanStatus::Complete).unwrap();
    let back: ScanStatus = serde_json::from_str(&json).unwrap();
    assert!(matches!(back, ScanStatus::Complete));

    let json2 = serde_json::to_string(&ScanStatus::Partial).unwrap();
    let back2: ScanStatus = serde_json::from_str(&json2).unwrap();
    assert!(matches!(back2, ScanStatus::Partial));
}

#[test]
fn coupling_row_optional_fields_skip_when_none() {
    let row = CouplingRow {
        left: "a".to_string(),
        right: "b".to_string(),
        count: 5,
        jaccard: None,
        lift: None,
        n_left: None,
        n_right: None,
    };
    let json = serde_json::to_string(&row).unwrap();
    assert!(!json.contains("jaccard"));
    assert!(!json.contains("lift"));
}

#[test]
fn coupling_row_optional_fields_present_when_some() {
    let row = CouplingRow {
        left: "a".to_string(),
        right: "b".to_string(),
        count: 5,
        jaccard: Some(0.75),
        lift: Some(1.2),
        n_left: Some(10),
        n_right: Some(8),
    };
    let json = serde_json::to_string(&row).unwrap();
    assert!(json.contains("\"jaccard\":0.75"));
    assert!(json.contains("\"lift\":1.2"));
}

#[test]
fn near_dup_report_truncated_default_false() {
    let report = NearDuplicateReport {
        params: NearDupParams {
            scope: NearDupScope::Module,
            threshold: 0.8,
            max_files: 100,
            max_pairs: None,
            max_file_bytes: None,
            selection_method: None,
            algorithm: None,
            exclude_patterns: vec![],
        },
        pairs: vec![],
        files_analyzed: 0,
        files_skipped: 0,
        eligible_files: None,
        clusters: None,
        truncated: false,
        excluded_by_pattern: None,
        stats: None,
    };
    let json = serde_json::to_string(&report).unwrap();
    let back: NearDuplicateReport = serde_json::from_str(&json).unwrap();
    assert!(!back.truncated);
}

#[test]
fn envelope_type_aliases_work() {
    // Verify re-exports compile and refer to the same types
    let _tool: EnvelopeTool = ToolMeta::new("test", "1.0.0", "check");
    let report: Envelope = SensorReport::new(
        ToolMeta::new("test", "1.0.0", "check"),
        "2025-01-01T00:00:00Z".to_string(),
        Verdict::Pass,
        "ok".to_string(),
    );
    assert_eq!(report.schema, "sensor.report.v1");
}

#[test]
fn api_surface_report_roundtrip() {
    let report = ApiSurfaceReport {
        total_items: 100,
        public_items: 30,
        internal_items: 70,
        public_ratio: 0.3,
        documented_ratio: 0.5,
        by_language: BTreeMap::from([(
            "Rust".to_string(),
            LangApiSurface {
                total_items: 100,
                public_items: 30,
                internal_items: 70,
                public_ratio: 0.3,
            },
        )]),
        by_module: vec![],
        top_exporters: vec![],
    };
    let json = serde_json::to_string(&report).unwrap();
    let back: ApiSurfaceReport = serde_json::from_str(&json).unwrap();
    assert_eq!(back.total_items, 100);
    assert_eq!(back.by_language.len(), 1);
}
