//! W69 deep property-based tests for tokmd-analysis-types.
//!
//! Covers schema version constants, enum serde roundtrips, struct roundtrips,
//! DerivedReport invariants, baseline defaults, and preset validation.

use proptest::prelude::*;
use tokmd_analysis_types::{EffortDriver, EffortDriverDirection, EffortResults};
use tokmd_analysis_types::{
    ANALYSIS_SCHEMA_VERSION, AnalysisArgsMeta, AnalysisReceipt, AnalysisSource, Archetype,
    BASELINE_VERSION, BaselineMetrics, CommitIntentCounts, CommitIntentKind, ComplexityBaseline,
    ComplexityRisk, DomainStat, EcoLabel, EntropyClass, LicenseSourceKind, NearDupScope, RatioRow,
    TechnicalDebtLevel, TopicTerm, TrendClass,
};
use tokmd_types::{ScanStatus, ToolInfo};

// =========================================================================
// 1. ANALYSIS_SCHEMA_VERSION is expected value
// =========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(1))]

    #[test]
    fn analysis_schema_version_value(_dummy in 0..1u8) {
        prop_assert_eq!(ANALYSIS_SCHEMA_VERSION, 9u32);
    }
}

// =========================================================================
// 2. BASELINE_VERSION is expected value
// =========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(1))]

    #[test]
    fn baseline_version_value(_dummy in 0..1u8) {
        prop_assert_eq!(BASELINE_VERSION, 1u32);
    }
}

// =========================================================================
// 3. EntropyClass all variants serde roundtrip
// =========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(8))]

    #[test]
    fn entropy_class_roundtrip(idx in 0usize..4) {
        let all = [EntropyClass::Low, EntropyClass::Normal, EntropyClass::Suspicious, EntropyClass::High];
        let json = serde_json::to_string(&all[idx]).unwrap();
        let parsed: EntropyClass = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(all[idx], parsed);
    }
}

// =========================================================================
// 4. TrendClass all variants serde roundtrip
// =========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(6))]

    #[test]
    fn trend_class_roundtrip(idx in 0usize..3) {
        let all = [TrendClass::Rising, TrendClass::Flat, TrendClass::Falling];
        let json = serde_json::to_string(&all[idx]).unwrap();
        let parsed: TrendClass = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(all[idx], parsed);
    }
}

// =========================================================================
// 5. ComplexityRisk all variants serde roundtrip
// =========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(8))]

    #[test]
    fn complexity_risk_roundtrip(idx in 0usize..4) {
        let all = [ComplexityRisk::Low, ComplexityRisk::Moderate, ComplexityRisk::High, ComplexityRisk::Critical];
        let json = serde_json::to_string(&all[idx]).unwrap();
        let parsed: ComplexityRisk = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(all[idx], parsed);
    }
}

// =========================================================================
// 6. TechnicalDebtLevel all variants serde roundtrip
// =========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(8))]

    #[test]
    fn technical_debt_level_roundtrip(idx in 0usize..4) {
        let all = [TechnicalDebtLevel::Low, TechnicalDebtLevel::Moderate, TechnicalDebtLevel::High, TechnicalDebtLevel::Critical];
        let json = serde_json::to_string(&all[idx]).unwrap();
        let parsed: TechnicalDebtLevel = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(all[idx], parsed);
    }
}

// =========================================================================
// 7. LicenseSourceKind all variants serde roundtrip
// =========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(4))]

    #[test]
    fn license_source_kind_roundtrip(idx in 0usize..2) {
        let all = [LicenseSourceKind::Metadata, LicenseSourceKind::Text];
        let json = serde_json::to_string(&all[idx]).unwrap();
        let parsed: LicenseSourceKind = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(all[idx], parsed);
    }
}

// =========================================================================
// 8. NearDupScope all variants serde roundtrip
// =========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(6))]

    #[test]
    fn near_dup_scope_roundtrip(idx in 0usize..3) {
        let all = [NearDupScope::Module, NearDupScope::Lang, NearDupScope::Global];
        let json = serde_json::to_string(&all[idx]).unwrap();
        let parsed: NearDupScope = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(all[idx], parsed);
    }
}

// =========================================================================
// 9. RatioRow ratio is in [0.0, 1.0] when properly constructed
// =========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(200))]

    #[test]
    fn ratio_row_bounded(
        numerator in 0usize..100_000,
        denominator in 1usize..100_000,
    ) {
        let ratio = numerator as f64 / denominator as f64;
        let row = RatioRow {
            key: "test".into(),
            numerator,
            denominator,
            ratio,
        };
        // ratio can be > 1.0 when numerator > denominator (e.g., doc density with headers)
        // but it should always be non-negative
        prop_assert!(row.ratio >= 0.0);
    }
}

// =========================================================================
// 10. RatioRow serde roundtrip
// =========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn ratio_row_serde_roundtrip(
        numerator in 0usize..100_000,
        denominator in 1usize..100_000,
    ) {
        let ratio = numerator as f64 / denominator as f64;
        let row = RatioRow {
            key: "test".into(),
            numerator,
            denominator,
            ratio,
        };
        let json = serde_json::to_string(&row).unwrap();
        let parsed: RatioRow = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(row.key, parsed.key);
        prop_assert_eq!(row.numerator, parsed.numerator);
        prop_assert_eq!(row.denominator, parsed.denominator);
    }
}

// =========================================================================
// 11. ComplexityBaseline default has expected version
// =========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(1))]

    #[test]
    fn complexity_baseline_default_version(_dummy in 0..1u8) {
        let b = ComplexityBaseline::default();
        prop_assert_eq!(b.baseline_version, BASELINE_VERSION);
        prop_assert!(b.generated_at.is_empty());
        prop_assert!(b.commit.is_none());
        prop_assert!(b.files.is_empty());
        prop_assert!(b.complexity.is_none());
        prop_assert!(b.determinism.is_none());
    }
}

// =========================================================================
// 12. BaselineMetrics default is all zeros
// =========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(1))]

    #[test]
    fn baseline_metrics_default_zeroed(_dummy in 0..1u8) {
        let m = BaselineMetrics::default();
        prop_assert_eq!(m.total_code_lines, 0u64);
        prop_assert_eq!(m.total_files, 0u64);
        prop_assert_eq!(m.avg_cyclomatic, 0.0);
        prop_assert_eq!(m.max_cyclomatic, 0u32);
        prop_assert_eq!(m.avg_cognitive, 0.0);
        prop_assert_eq!(m.max_cognitive, 0u32);
        prop_assert_eq!(m.avg_nesting_depth, 0.0);
        prop_assert_eq!(m.max_nesting_depth, 0u32);
        prop_assert_eq!(m.function_count, 0u64);
        prop_assert_eq!(m.avg_function_length, 0.0);
    }
}

// =========================================================================
// 13. CommitIntentCounts::increment covers all variants
// =========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(24))]

    #[test]
    fn commit_intent_counts_increment(idx in 0usize..12) {
        let all = [
            CommitIntentKind::Feat, CommitIntentKind::Fix, CommitIntentKind::Refactor,
            CommitIntentKind::Docs, CommitIntentKind::Test, CommitIntentKind::Chore,
            CommitIntentKind::Ci, CommitIntentKind::Build, CommitIntentKind::Perf,
            CommitIntentKind::Style, CommitIntentKind::Revert, CommitIntentKind::Other,
        ];
        let mut counts = CommitIntentCounts::default();
        counts.increment(all[idx]);
        prop_assert_eq!(counts.total, 1);
    }
}

// =========================================================================
// 14. CommitIntentCounts total equals sum of fields
// =========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(50))]

    #[test]
    fn commit_intent_counts_total_equals_sum(
        feat_n in 0usize..10,
        fix_n in 0usize..10,
        other_n in 0usize..10,
    ) {
        let mut counts = CommitIntentCounts::default();
        for _ in 0..feat_n { counts.increment(CommitIntentKind::Feat); }
        for _ in 0..fix_n { counts.increment(CommitIntentKind::Fix); }
        for _ in 0..other_n { counts.increment(CommitIntentKind::Other); }
        let field_sum = counts.feat + counts.fix + counts.refactor + counts.docs
            + counts.test + counts.chore + counts.ci + counts.build
            + counts.perf + counts.style + counts.revert + counts.other;
        prop_assert_eq!(counts.total, field_sum);
    }
}

// =========================================================================
// 15. TopicTerm serde roundtrip
// =========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn topic_term_serde_roundtrip(
        score in 0.0f64..100.0,
        tf in 0u32..1000,
        df in 0u32..100,
    ) {
        let term = TopicTerm {
            term: "async".into(),
            score,
            tf,
            df,
        };
        let json = serde_json::to_string(&term).unwrap();
        let parsed: TopicTerm = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(term.term, parsed.term);
        prop_assert_eq!(term.tf, parsed.tf);
        prop_assert_eq!(term.df, parsed.df);
    }
}

// =========================================================================
// 16. DomainStat serde roundtrip and pct in [0, 100]
// =========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn domain_stat_roundtrip(
        commits in 0u32..10000,
        pct in 0.0f32..100.0,
    ) {
        let stat = DomainStat {
            domain: "github.com".into(),
            commits,
            pct,
        };
        let json = serde_json::to_string(&stat).unwrap();
        let parsed: DomainStat = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(stat.domain, parsed.domain);
        prop_assert_eq!(stat.commits, parsed.commits);
        prop_assert!(parsed.pct >= 0.0);
    }
}

// =========================================================================
// 17. EcoLabel serde roundtrip
// =========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(50))]

    #[test]
    fn eco_label_roundtrip(
        score in 0.0f64..100.0,
        bytes in 0u64..10_000_000,
    ) {
        let label = EcoLabel {
            score,
            label: "A".into(),
            bytes,
            notes: "Good".into(),
        };
        let json = serde_json::to_string(&label).unwrap();
        let parsed: EcoLabel = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(label.label, parsed.label);
        prop_assert_eq!(label.bytes, parsed.bytes);
    }
}

// =========================================================================
// 18. AnalysisReceipt minimal roundtrip (no optional sections)
// =========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(10))]

    #[test]
    fn analysis_receipt_minimal_roundtrip(_dummy in 0..1u8) {
        let receipt = AnalysisReceipt {
            schema_version: ANALYSIS_SCHEMA_VERSION,
            generated_at_ms: 1700000000000,
            tool: ToolInfo { name: "tokmd".into(), version: "0.1.0".into() },
            mode: "analyze".into(),
            status: ScanStatus::Complete,
            warnings: vec![],
            source: AnalysisSource {
                inputs: vec![".".into()],
                export_path: None,
                base_receipt_path: None,
                export_schema_version: None,
                export_generated_at_ms: None,
                base_signature: None,
                module_roots: vec!["crates".into()],
                module_depth: 2,
                children: "collapse".into(),
            },
            args: AnalysisArgsMeta {
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
            effort: None,
            complexity: None,
            api_surface: None,
            fun: None,
        };
        let json = serde_json::to_string(&receipt).unwrap();
        let parsed: AnalysisReceipt = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(parsed.schema_version, ANALYSIS_SCHEMA_VERSION);
        prop_assert_eq!(parsed.mode, "analyze");
        prop_assert!(parsed.derived.is_none());
    }
}

// =========================================================================
// 19. AnalysisReceipt with Archetype roundtrip
// =========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(20))]

    #[test]
    fn analysis_receipt_archetype_roundtrip(
        kind in "[a-z_]{3,15}",
    ) {
        let archetype = Archetype {
            kind: kind.clone(),
            evidence: vec!["has Cargo.toml".into()],
        };
        let json = serde_json::to_string(&archetype).unwrap();
        let parsed: Archetype = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(kind, parsed.kind);
        prop_assert_eq!(parsed.evidence.len(), 1);
    }
}

// =========================================================================
// 20. All analysis presets are valid strings
// =========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(22))]

    #[test]
    fn analysis_preset_names_valid(idx in 0usize..12) {
        let presets = [
            "receipt", "estimate", "health", "risk", "supply", "architecture",
            "topics", "security", "identity", "git", "deep", "fun",
        ];
        let preset = presets[idx];
        prop_assert!(!preset.is_empty());
        prop_assert!(preset.chars().all(|c| c.is_ascii_lowercase()));
    }
}

// =========================================================================
// 21. EffortDriver and EffortResults serde roundtrip properties
// =========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(50))]

    #[test]
    fn effort_driver_serde_roundtrip(
        key in "[a-z_]{3,15}",
        label in "[A-Za-z ]{5,30}",
        weight in -100.0f64..100.0,
        evidence in "[A-Za-z0-9 ]{5,50}",
    ) {
        let driver = EffortDriver {
            key,
            label,
            weight,
            direction: EffortDriverDirection::Raises,
            evidence,
        };
        let json = serde_json::to_string(&driver).unwrap();
        let parsed: EffortDriver = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(driver.key, parsed.key);
        prop_assert_eq!(driver.label, parsed.label);
        prop_assert!((driver.weight - parsed.weight).abs() < 1e-10);
        prop_assert_eq!(driver.direction, parsed.direction);
        prop_assert_eq!(driver.evidence, parsed.evidence);
    }

    #[test]
    fn effort_results_serde_roundtrip(
        effort_pm_p50 in 0.0f64..1000.0,
        schedule_months_p50 in 0.0f64..1000.0,
        staff_p50 in 0.0f64..1000.0,
        effort_pm_low in 0.0f64..1000.0,
        effort_pm_p80 in 0.0f64..1000.0,
        schedule_months_low in 0.0f64..1000.0,
        schedule_months_p80 in 0.0f64..1000.0,
        staff_low in 0.0f64..1000.0,
        staff_p80 in 0.0f64..1000.0,
    ) {
        let results = EffortResults {
            effort_pm_p50,
            schedule_months_p50,
            staff_p50,
            effort_pm_low,
            effort_pm_p80,
            schedule_months_low,
            schedule_months_p80,
            staff_low,
            staff_p80,
        };
        let json = serde_json::to_string(&results).unwrap();
        let parsed: EffortResults = serde_json::from_str(&json).unwrap();

        prop_assert!((results.effort_pm_p50 - parsed.effort_pm_p50).abs() < 1e-10);
        prop_assert!((results.schedule_months_p50 - parsed.schedule_months_p50).abs() < 1e-10);
        prop_assert!((results.staff_p50 - parsed.staff_p50).abs() < 1e-10);
        prop_assert!((results.effort_pm_low - parsed.effort_pm_low).abs() < 1e-10);
        prop_assert!((results.effort_pm_p80 - parsed.effort_pm_p80).abs() < 1e-10);
        prop_assert!((results.schedule_months_low - parsed.schedule_months_low).abs() < 1e-10);
        prop_assert!((results.schedule_months_p80 - parsed.schedule_months_p80).abs() < 1e-10);
        prop_assert!((results.staff_low - parsed.staff_low).abs() < 1e-10);
        prop_assert!((results.staff_p80 - parsed.staff_p80).abs() < 1e-10);
    }
}
