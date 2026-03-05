//! W61 depth tests for `tokmd-analysis-types`.
//!
//! Coverage: serde roundtrips for every report struct, enum exhaustiveness,
//! schema-version verification, NearDup/Halstead/Baseline contracts,
//! ComplexityHistogram logic, CommitIntentCounts increment coverage,
//! and proptest-based property verification.

use std::collections::BTreeMap;

use proptest::prelude::*;
use tokmd_analysis_types::*;
use tokmd_types::{CommitIntentKind, ScanStatus, ToolInfo};

// ══════════════════════════════════════════════════════════════════════
// Helpers
// ══════════════════════════════════════════════════════════════════════

fn tool() -> ToolInfo {
    ToolInfo {
        name: "tokmd".into(),
        version: "0.0.0-test".into(),
    }
}

fn source() -> AnalysisSource {
    AnalysisSource {
        inputs: vec![".".into()],
        export_path: None,
        base_receipt_path: None,
        export_schema_version: None,
        export_generated_at_ms: None,
        base_signature: None,
        module_roots: vec![],
        module_depth: 2,
        children: "parents-only".into(),
    }
}

fn args() -> AnalysisArgsMeta {
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
    }
}

fn minimal_receipt() -> AnalysisReceipt {
    AnalysisReceipt {
        schema_version: ANALYSIS_SCHEMA_VERSION,
        generated_at_ms: 1_700_000_000_000,
        tool: tool(),
        mode: "receipt".into(),
        status: ScanStatus::Complete,
        warnings: vec![],
        source: source(),
        args: args(),
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

// ══════════════════════════════════════════════════════════════════════
// 1. Schema version constants
// ══════════════════════════════════════════════════════════════════════

#[test]
fn schema_version_is_8() {
    assert_eq!(ANALYSIS_SCHEMA_VERSION, 8);
}

#[test]
fn baseline_version_is_1() {
    assert_eq!(BASELINE_VERSION, 1);
}

#[test]
fn envelope_schema_constant_format() {
    assert!(ENVELOPE_SCHEMA.starts_with("sensor.report."));
}

// ══════════════════════════════════════════════════════════════════════
// 2. Enum serde exhaustiveness — every variant survives a roundtrip
// ══════════════════════════════════════════════════════════════════════

#[test]
fn entropy_class_all_variants_roundtrip() {
    let variants = [
        EntropyClass::Low,
        EntropyClass::Normal,
        EntropyClass::Suspicious,
        EntropyClass::High,
    ];
    for v in variants {
        let json = serde_json::to_string(&v).unwrap();
        let back: EntropyClass = serde_json::from_str(&json).unwrap();
        assert_eq!(back, v);
    }
}

#[test]
fn trend_class_all_variants_roundtrip() {
    let variants = [TrendClass::Rising, TrendClass::Flat, TrendClass::Falling];
    for v in variants {
        let json = serde_json::to_string(&v).unwrap();
        let back: TrendClass = serde_json::from_str(&json).unwrap();
        assert_eq!(back, v);
    }
}

#[test]
fn license_source_kind_all_variants_roundtrip() {
    let variants = [LicenseSourceKind::Metadata, LicenseSourceKind::Text];
    for v in variants {
        let json = serde_json::to_string(&v).unwrap();
        let back: LicenseSourceKind = serde_json::from_str(&json).unwrap();
        assert_eq!(back, v);
    }
}

#[test]
fn complexity_risk_all_variants_roundtrip() {
    let variants = [
        ComplexityRisk::Low,
        ComplexityRisk::Moderate,
        ComplexityRisk::High,
        ComplexityRisk::Critical,
    ];
    for v in variants {
        let json = serde_json::to_string(&v).unwrap();
        let back: ComplexityRisk = serde_json::from_str(&json).unwrap();
        assert_eq!(back, v);
    }
}

#[test]
fn technical_debt_level_all_variants_roundtrip() {
    let variants = [
        TechnicalDebtLevel::Low,
        TechnicalDebtLevel::Moderate,
        TechnicalDebtLevel::High,
        TechnicalDebtLevel::Critical,
    ];
    for v in variants {
        let json = serde_json::to_string(&v).unwrap();
        let back: TechnicalDebtLevel = serde_json::from_str(&json).unwrap();
        assert_eq!(back, v);
    }
}

#[test]
fn near_dup_scope_all_variants_roundtrip() {
    let variants = [
        NearDupScope::Module,
        NearDupScope::Lang,
        NearDupScope::Global,
    ];
    for v in variants {
        let json = serde_json::to_string(&v).unwrap();
        let back: NearDupScope = serde_json::from_str(&json).unwrap();
        assert_eq!(back, v);
    }
}

#[test]
fn near_dup_scope_default_is_module() {
    assert_eq!(NearDupScope::default(), NearDupScope::Module);
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
fn finding_severity_all_variants_roundtrip() {
    let variants = [
        FindingSeverity::Error,
        FindingSeverity::Warn,
        FindingSeverity::Info,
    ];
    for v in variants {
        let json = serde_json::to_string(&v).unwrap();
        let back: FindingSeverity = serde_json::from_str(&json).unwrap();
        assert_eq!(back, v);
    }
}

#[test]
fn verdict_all_variants_roundtrip() {
    let variants = [
        Verdict::Pass,
        Verdict::Fail,
        Verdict::Warn,
        Verdict::Skip,
        Verdict::Pending,
    ];
    for v in variants {
        let json = serde_json::to_string(&v).unwrap();
        let back: Verdict = serde_json::from_str(&json).unwrap();
        assert_eq!(back, v);
    }
}

#[test]
fn verdict_default_is_pass() {
    assert_eq!(Verdict::default(), Verdict::Pass);
}

// ══════════════════════════════════════════════════════════════════════
// 3. Struct serde roundtrips — complex report types
// ══════════════════════════════════════════════════════════════════════

#[test]
fn halstead_metrics_serde_roundtrip() {
    let h = HalsteadMetrics {
        distinct_operators: 20,
        distinct_operands: 40,
        total_operators: 200,
        total_operands: 300,
        vocabulary: 60,
        length: 500,
        volume: 2959.0,
        difficulty: 75.0,
        effort: 221925.0,
        time_seconds: 12329.17,
        estimated_bugs: 0.99,
    };
    let json = serde_json::to_string(&h).unwrap();
    let back: HalsteadMetrics = serde_json::from_str(&json).unwrap();
    assert_eq!(back.distinct_operators, 20);
    assert_eq!(back.vocabulary, 60);
    assert_eq!(back.length, 500);
}

#[test]
fn maintainability_index_serde_roundtrip() {
    let mi = MaintainabilityIndex {
        score: 95.5,
        avg_cyclomatic: 3.2,
        avg_loc: 42.0,
        avg_halstead_volume: Some(800.0),
        grade: "A".into(),
    };
    let json = serde_json::to_string(&mi).unwrap();
    let back: MaintainabilityIndex = serde_json::from_str(&json).unwrap();
    assert_eq!(back.grade, "A");
    assert_eq!(back.avg_halstead_volume, Some(800.0));
}

#[test]
fn maintainability_index_without_halstead() {
    let mi = MaintainabilityIndex {
        score: 70.0,
        avg_cyclomatic: 8.0,
        avg_loc: 100.0,
        avg_halstead_volume: None,
        grade: "B".into(),
    };
    let json = serde_json::to_string(&mi).unwrap();
    assert!(!json.contains("avg_halstead_volume"));
}

#[test]
fn technical_debt_ratio_serde_roundtrip() {
    let td = TechnicalDebtRatio {
        ratio: 12.5,
        complexity_points: 250,
        code_kloc: 20.0,
        level: TechnicalDebtLevel::Moderate,
    };
    let json = serde_json::to_string(&td).unwrap();
    let back: TechnicalDebtRatio = serde_json::from_str(&json).unwrap();
    assert_eq!(back.level, TechnicalDebtLevel::Moderate);
    assert_eq!(back.complexity_points, 250);
}

#[test]
fn near_dup_algorithm_serde_roundtrip() {
    let algo = NearDupAlgorithm {
        k_gram_size: 5,
        window_size: 4,
        max_postings: 1000,
    };
    let json = serde_json::to_string(&algo).unwrap();
    let back: NearDupAlgorithm = serde_json::from_str(&json).unwrap();
    assert_eq!(back, algo);
}

#[test]
fn near_dup_stats_serde_roundtrip() {
    let stats = NearDupStats {
        fingerprinting_ms: 120,
        pairing_ms: 45,
        bytes_processed: 1_000_000,
    };
    let json = serde_json::to_string(&stats).unwrap();
    let back: NearDupStats = serde_json::from_str(&json).unwrap();
    assert_eq!(back, stats);
}

#[test]
fn near_dup_cluster_serde_roundtrip() {
    let cluster = NearDupCluster {
        files: vec!["a.rs".into(), "b.rs".into()],
        max_similarity: 0.95,
        representative: "a.rs".into(),
        pair_count: 1,
    };
    let json = serde_json::to_string(&cluster).unwrap();
    let back: NearDupCluster = serde_json::from_str(&json).unwrap();
    assert_eq!(back.files.len(), 2);
    assert_eq!(back.representative, "a.rs");
}

#[test]
fn api_surface_report_serde_roundtrip() {
    let mut by_language = BTreeMap::new();
    by_language.insert(
        "Rust".into(),
        LangApiSurface {
            total_items: 100,
            public_items: 40,
            internal_items: 60,
            public_ratio: 0.4,
        },
    );
    let report = ApiSurfaceReport {
        total_items: 100,
        public_items: 40,
        internal_items: 60,
        public_ratio: 0.4,
        documented_ratio: 0.8,
        by_language,
        by_module: vec![ModuleApiRow {
            module: "src".into(),
            total_items: 100,
            public_items: 40,
            public_ratio: 0.4,
        }],
        top_exporters: vec![ApiExportItem {
            path: "src/lib.rs".into(),
            lang: "Rust".into(),
            public_items: 20,
            total_items: 50,
        }],
    };
    let json = serde_json::to_string(&report).unwrap();
    let back: ApiSurfaceReport = serde_json::from_str(&json).unwrap();
    assert_eq!(back.total_items, 100);
    assert_eq!(back.by_language.len(), 1);
    assert_eq!(back.top_exporters.len(), 1);
}

#[test]
fn code_age_distribution_serde_roundtrip() {
    let dist = CodeAgeDistributionReport {
        buckets: vec![CodeAgeBucket {
            label: "0-30 days".into(),
            min_days: 0,
            max_days: Some(30),
            files: 10,
            pct: 0.5,
        }],
        recent_refreshes: 5,
        prior_refreshes: 3,
        refresh_trend: TrendClass::Rising,
    };
    let json = serde_json::to_string(&dist).unwrap();
    let back: CodeAgeDistributionReport = serde_json::from_str(&json).unwrap();
    assert_eq!(back.buckets.len(), 1);
    assert_eq!(back.refresh_trend, TrendClass::Rising);
}

#[test]
fn commit_intent_report_serde_roundtrip() {
    let report = CommitIntentReport {
        overall: CommitIntentCounts::default(),
        by_module: vec![],
        unknown_pct: 0.1,
        corrective_ratio: Some(0.15),
    };
    let json = serde_json::to_string(&report).unwrap();
    let back: CommitIntentReport = serde_json::from_str(&json).unwrap();
    assert_eq!(back.corrective_ratio, Some(0.15));
}

// ══════════════════════════════════════════════════════════════════════
// 4. CommitIntentCounts increment coverage
// ══════════════════════════════════════════════════════════════════════

#[test]
fn commit_intent_counts_increment_all_kinds() {
    let mut counts = CommitIntentCounts::default();
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
        counts.increment(kind);
    }
    assert_eq!(counts.total, 12);
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
}

#[test]
fn commit_intent_counts_increments_total() {
    let mut counts = CommitIntentCounts::default();
    counts.increment(CommitIntentKind::Fix);
    counts.increment(CommitIntentKind::Fix);
    counts.increment(CommitIntentKind::Feat);
    assert_eq!(counts.fix, 2);
    assert_eq!(counts.feat, 1);
    assert_eq!(counts.total, 3);
}

#[test]
fn commit_intent_counts_default_is_zeroed() {
    let c = CommitIntentCounts::default();
    assert_eq!(c.total, 0);
    assert_eq!(c.feat, 0);
    assert_eq!(c.other, 0);
}

// ══════════════════════════════════════════════════════════════════════
// 5. ComplexityHistogram logic
// ══════════════════════════════════════════════════════════════════════

#[test]
fn histogram_ascii_lines_equal_counts_len() {
    let h = ComplexityHistogram {
        buckets: vec![0, 5, 10, 15],
        counts: vec![20, 10, 5, 1],
        total: 36,
    };
    let ascii = h.to_ascii(40);
    assert_eq!(ascii.lines().count(), 4);
}

#[test]
fn histogram_ascii_single_bucket() {
    let h = ComplexityHistogram {
        buckets: vec![0],
        counts: vec![42],
        total: 42,
    };
    let ascii = h.to_ascii(10);
    assert_eq!(ascii.lines().count(), 1);
    assert!(ascii.contains("42"));
}

#[test]
fn histogram_ascii_all_zero_no_panic() {
    let h = ComplexityHistogram {
        buckets: vec![0, 10, 20],
        counts: vec![0, 0, 0],
        total: 0,
    };
    let ascii = h.to_ascii(20);
    assert_eq!(ascii.lines().count(), 3);
}

// ══════════════════════════════════════════════════════════════════════
// 6. ComplexityBaseline construction and from_analysis
// ══════════════════════════════════════════════════════════════════════

#[test]
fn baseline_new_matches_default() {
    let a = ComplexityBaseline::new();
    let b = ComplexityBaseline::default();
    assert_eq!(a.baseline_version, b.baseline_version);
    assert_eq!(a.files.len(), b.files.len());
}

#[test]
fn baseline_from_analysis_no_complexity() {
    let receipt = minimal_receipt();
    let baseline = ComplexityBaseline::from_analysis(&receipt);
    assert_eq!(baseline.baseline_version, BASELINE_VERSION);
    assert!(baseline.files.is_empty());
    assert!(baseline.complexity.is_none());
}

#[test]
fn baseline_from_analysis_with_complexity() {
    let mut receipt = minimal_receipt();
    receipt.derived = Some(DerivedReport {
        totals: DerivedTotals {
            files: 5,
            code: 200,
            comments: 20,
            blanks: 10,
            lines: 230,
            bytes: 5000,
            tokens: 1000,
        },
        doc_density: RatioReport {
            total: RatioRow {
                key: "t".into(),
                numerator: 20,
                denominator: 230,
                ratio: 0.087,
            },
            by_lang: vec![],
            by_module: vec![],
        },
        whitespace: RatioReport {
            total: RatioRow {
                key: "t".into(),
                numerator: 10,
                denominator: 230,
                ratio: 0.043,
            },
            by_lang: vec![],
            by_module: vec![],
        },
        verbosity: RateReport {
            total: RateRow {
                key: "t".into(),
                numerator: 5000,
                denominator: 230,
                rate: 21.7,
            },
            by_lang: vec![],
            by_module: vec![],
        },
        max_file: MaxFileReport {
            overall: FileStatRow {
                path: "f.rs".into(),
                module: "src".into(),
                lang: "Rust".into(),
                code: 100,
                comments: 10,
                blanks: 5,
                lines: 115,
                bytes: 2500,
                tokens: 500,
                doc_pct: None,
                bytes_per_line: None,
                depth: 1,
            },
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
            test_lines: 50,
            prod_lines: 150,
            test_files: 1,
            prod_files: 4,
            ratio: 0.33,
        },
        boilerplate: BoilerplateReport {
            infra_lines: 10,
            logic_lines: 190,
            ratio: 0.05,
            infra_langs: vec![],
        },
        polyglot: PolyglotReport {
            lang_count: 1,
            entropy: 0.0,
            dominant_lang: "Rust".into(),
            dominant_lines: 200,
            dominant_pct: 1.0,
        },
        distribution: DistributionReport {
            count: 5,
            min: 10,
            max: 100,
            mean: 40.0,
            median: 35.0,
            p90: 90.0,
            p99: 99.0,
            gini: 0.25,
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
            minutes: 1.5,
            lines_per_minute: 130,
            basis_lines: 200,
        },
        context_window: None,
        cocomo: None,
        todo: None,
        integrity: IntegrityReport {
            algo: "blake3".into(),
            hash: "abc".into(),
            entries: 5,
        },
    });
    receipt.complexity = Some(ComplexityReport {
        total_functions: 10,
        avg_function_length: 20.0,
        max_function_length: 50,
        avg_cyclomatic: 4.5,
        max_cyclomatic: 12,
        avg_cognitive: Some(3.0),
        max_cognitive: Some(8),
        avg_nesting_depth: Some(2.0),
        max_nesting_depth: Some(5),
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
            cognitive_complexity: Some(8),
            max_nesting: Some(5),
            risk_level: ComplexityRisk::High,
            functions: None,
        }],
    });

    let baseline = ComplexityBaseline::from_analysis(&receipt);
    assert_eq!(baseline.files.len(), 1);
    assert_eq!(baseline.files[0].path, "src/main.rs");
    assert_eq!(baseline.files[0].cyclomatic, 12);
    assert_eq!(baseline.metrics.avg_cyclomatic, 4.5);
    assert_eq!(baseline.metrics.total_code_lines, 200);
    assert_eq!(baseline.metrics.total_files, 5);

    let cs = baseline.complexity.as_ref().unwrap();
    assert_eq!(cs.total_functions, 10);
    assert_eq!(cs.high_risk_files, 1);
}

// ══════════════════════════════════════════════════════════════════════
// 7. Full AnalysisReceipt serde roundtrip
// ══════════════════════════════════════════════════════════════════════

#[test]
fn analysis_receipt_minimal_roundtrip() {
    let r = minimal_receipt();
    let json = serde_json::to_string(&r).unwrap();
    let back: AnalysisReceipt = serde_json::from_str(&json).unwrap();
    assert_eq!(back.schema_version, ANALYSIS_SCHEMA_VERSION);
    assert_eq!(back.mode, "receipt");
}

#[test]
fn analysis_receipt_json_contains_schema_version() {
    let r = minimal_receipt();
    let json = serde_json::to_string(&r).unwrap();
    let val: serde_json::Value = serde_json::from_str(&json).unwrap();
    assert_eq!(val["schema_version"], ANALYSIS_SCHEMA_VERSION);
}

#[test]
fn analysis_receipt_with_warnings_roundtrip() {
    let mut r = minimal_receipt();
    r.warnings = vec!["git not available".into(), "content scan skipped".into()];
    let json = serde_json::to_string(&r).unwrap();
    let back: AnalysisReceipt = serde_json::from_str(&json).unwrap();
    assert_eq!(back.warnings.len(), 2);
}

#[test]
fn analysis_receipt_scan_status_partial() {
    let mut r = minimal_receipt();
    r.status = ScanStatus::Partial;
    let json = serde_json::to_string(&r).unwrap();
    let _back: AnalysisReceipt = serde_json::from_str(&json).unwrap();
    let json_val: serde_json::Value = serde_json::from_str(&json).unwrap();
    assert_eq!(json_val["status"], "partial");
}

// ══════════════════════════════════════════════════════════════════════
// 8. Envelope / SensorReport re-export roundtrip
// ══════════════════════════════════════════════════════════════════════

#[test]
fn sensor_report_alias_serde_roundtrip() {
    let report = SensorReport {
        schema: ENVELOPE_SCHEMA.into(),
        tool: ToolMeta {
            name: "tokmd".into(),
            version: "1.0.0".into(),
            mode: "sensor".into(),
        },
        generated_at: "2025-01-01T00:00:00Z".into(),
        verdict: Verdict::Pass,
        summary: "OK".into(),
        findings: vec![],
        artifacts: None,
        capabilities: None,
        data: None,
    };
    let json = serde_json::to_string(&report).unwrap();
    let back: Envelope = serde_json::from_str(&json).unwrap();
    assert_eq!(back.schema, ENVELOPE_SCHEMA);
    assert_eq!(back.verdict, Verdict::Pass);
}

#[test]
fn gate_results_serde_roundtrip() {
    let gates = GateResults {
        status: Verdict::Fail,
        items: vec![GateItem {
            id: "complexity".into(),
            status: Verdict::Fail,
            threshold: Some(10.0),
            actual: Some(15.0),
            reason: Some("exceeded threshold".into()),
            source: None,
            artifact_path: None,
        }],
    };
    let json = serde_json::to_string(&gates).unwrap();
    let back: GatesEnvelope = serde_json::from_str(&json).unwrap();
    assert_eq!(back.items.len(), 1);
    assert_eq!(back.status, Verdict::Fail);
}

// ══════════════════════════════════════════════════════════════════════
// 9. DeterminismBaseline
// ══════════════════════════════════════════════════════════════════════

#[test]
fn determinism_baseline_serde_roundtrip() {
    let db = DeterminismBaseline {
        baseline_version: 1,
        generated_at: "2025-06-01T00:00:00Z".into(),
        build_hash: "abc".into(),
        source_hash: "def".into(),
        cargo_lock_hash: Some("ghi".into()),
    };
    let json = serde_json::to_string(&db).unwrap();
    let back: DeterminismBaseline = serde_json::from_str(&json).unwrap();
    assert_eq!(back.cargo_lock_hash.as_deref(), Some("ghi"));
}

#[test]
fn determinism_baseline_without_cargo_lock() {
    let db = DeterminismBaseline {
        baseline_version: 1,
        generated_at: "2025-06-01T00:00:00Z".into(),
        build_hash: "abc".into(),
        source_hash: "def".into(),
        cargo_lock_hash: None,
    };
    let json = serde_json::to_string(&db).unwrap();
    let back: DeterminismBaseline = serde_json::from_str(&json).unwrap();
    assert!(back.cargo_lock_hash.is_none());
}

// ══════════════════════════════════════════════════════════════════════
// 10. FunctionComplexityDetail
// ══════════════════════════════════════════════════════════════════════

#[test]
fn function_complexity_detail_full_roundtrip() {
    let detail = FunctionComplexityDetail {
        name: "process".into(),
        line_start: 10,
        line_end: 50,
        length: 40,
        cyclomatic: 8,
        cognitive: Some(6),
        max_nesting: Some(3),
        param_count: Some(4),
    };
    let json = serde_json::to_string(&detail).unwrap();
    let back: FunctionComplexityDetail = serde_json::from_str(&json).unwrap();
    assert_eq!(back.name, "process");
    assert_eq!(back.param_count, Some(4));
}

#[test]
fn function_complexity_detail_minimal() {
    let detail = FunctionComplexityDetail {
        name: "f".into(),
        line_start: 1,
        line_end: 1,
        length: 1,
        cyclomatic: 1,
        cognitive: None,
        max_nesting: None,
        param_count: None,
    };
    let json = serde_json::to_string(&detail).unwrap();
    assert!(!json.contains("cognitive"));
    assert!(!json.contains("max_nesting"));
    assert!(!json.contains("param_count"));
}

// ══════════════════════════════════════════════════════════════════════
// 11. CouplingRow with optional normalization fields
// ══════════════════════════════════════════════════════════════════════

#[test]
fn coupling_row_with_normalization_roundtrip() {
    let row = CouplingRow {
        left: "mod_a".into(),
        right: "mod_b".into(),
        count: 10,
        jaccard: Some(0.3),
        lift: Some(2.5),
        n_left: Some(20),
        n_right: Some(15),
    };
    let json = serde_json::to_string(&row).unwrap();
    let back: CouplingRow = serde_json::from_str(&json).unwrap();
    assert_eq!(back.jaccard, Some(0.3));
    assert_eq!(back.lift, Some(2.5));
}

#[test]
fn coupling_row_without_normalization_skips_fields() {
    let row = CouplingRow {
        left: "a".into(),
        right: "b".into(),
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

// ══════════════════════════════════════════════════════════════════════
// 12. Proptest — property-based verification
// ══════════════════════════════════════════════════════════════════════

fn arb_entropy_class() -> impl Strategy<Value = EntropyClass> {
    prop_oneof![
        Just(EntropyClass::Low),
        Just(EntropyClass::Normal),
        Just(EntropyClass::Suspicious),
        Just(EntropyClass::High),
    ]
}

fn arb_complexity_risk() -> impl Strategy<Value = ComplexityRisk> {
    prop_oneof![
        Just(ComplexityRisk::Low),
        Just(ComplexityRisk::Moderate),
        Just(ComplexityRisk::High),
        Just(ComplexityRisk::Critical),
    ]
}

fn arb_near_dup_scope() -> impl Strategy<Value = NearDupScope> {
    prop_oneof![
        Just(NearDupScope::Module),
        Just(NearDupScope::Lang),
        Just(NearDupScope::Global),
    ]
}

proptest! {
    #[test]
    fn prop_entropy_class_roundtrip(v in arb_entropy_class()) {
        let json = serde_json::to_string(&v).unwrap();
        let back: EntropyClass = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(back, v);
    }

    #[test]
    fn prop_complexity_risk_roundtrip(v in arb_complexity_risk()) {
        let json = serde_json::to_string(&v).unwrap();
        let back: ComplexityRisk = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(back, v);
    }

    #[test]
    fn prop_near_dup_scope_roundtrip(v in arb_near_dup_scope()) {
        let json = serde_json::to_string(&v).unwrap();
        let back: NearDupScope = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(back, v);
    }

    #[test]
    fn prop_topic_term_score_preserved(score in 0.0f64..=1.0, tf in 0u32..1000, df in 0u32..100) {
        let term = TopicTerm { term: "t".into(), score, tf, df };
        let json = serde_json::to_string(&term).unwrap();
        let back: TopicTerm = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(back.tf, tf);
        prop_assert_eq!(back.df, df);
    }

    #[test]
    fn prop_histogram_bucket_preserved(
        min in 0usize..1000,
        files in 0usize..10000,
        pct in 0.0f64..=100.0
    ) {
        let bucket = HistogramBucket {
            label: format!("{}-{}", min, min + 5),
            min,
            max: Some(min + 5),
            files,
            pct,
        };
        let json = serde_json::to_string(&bucket).unwrap();
        let back: HistogramBucket = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(back.min, min);
        prop_assert_eq!(back.files, files);
    }
}
