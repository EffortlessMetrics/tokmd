//! Property-based tests for tokmd-analysis-types.
//!
//! These tests verify JSON serialization round-trips for all analysis types,
//! ensuring data integrity and schema stability.

use proptest::prelude::*;
use tokmd_analysis_types::*;

// ============================================================================
// Enum round-trip tests
// ============================================================================

proptest! {
    /// EntropyClass round-trips through JSON.
    #[test]
    fn entropy_class_roundtrip(variant in arb_entropy_class()) {
        let json = serde_json::to_string(&variant).expect("serialize");
        let parsed: EntropyClass = serde_json::from_str(&json).expect("deserialize");
        prop_assert_eq!(variant, parsed);
    }

    /// EntropyClass serializes to snake_case.
    #[test]
    fn entropy_class_snake_case(variant in arb_entropy_class()) {
        let json = serde_json::to_string(&variant).expect("serialize");
        let s = json.trim_matches('"');

        prop_assert!(
            !s.chars().any(|c| c.is_uppercase()),
            "EntropyClass should be snake_case: {}",
            s
        );
    }

    /// TrendClass round-trips through JSON.
    #[test]
    fn trend_class_roundtrip(variant in arb_trend_class()) {
        let json = serde_json::to_string(&variant).expect("serialize");
        let parsed: TrendClass = serde_json::from_str(&json).expect("deserialize");
        prop_assert_eq!(variant, parsed);
    }

    /// TrendClass serializes to snake_case.
    #[test]
    fn trend_class_snake_case(variant in arb_trend_class()) {
        let json = serde_json::to_string(&variant).expect("serialize");
        let s = json.trim_matches('"');

        prop_assert!(
            !s.chars().any(|c| c.is_uppercase()),
            "TrendClass should be snake_case: {}",
            s
        );
    }

    /// LicenseSourceKind round-trips through JSON.
    #[test]
    fn license_source_kind_roundtrip(variant in arb_license_source_kind()) {
        let json = serde_json::to_string(&variant).expect("serialize");
        let parsed: LicenseSourceKind = serde_json::from_str(&json).expect("deserialize");
        prop_assert_eq!(variant, parsed);
    }
}

// ============================================================================
// Simple struct round-trip tests
// ============================================================================

proptest! {
    /// Archetype round-trips through JSON.
    #[test]
    fn archetype_roundtrip(kind in "[a-z_]{3,15}", evidence in prop::collection::vec("[a-z ]{5,20}", 0..=5)) {
        let archetype = Archetype { kind, evidence };

        let json = serde_json::to_string(&archetype).expect("serialize");
        let parsed: Archetype = serde_json::from_str(&json).expect("deserialize");

        prop_assert_eq!(archetype.kind, parsed.kind);
        prop_assert_eq!(archetype.evidence, parsed.evidence);
    }

    /// TopicTerm round-trips through JSON.
    #[test]
    fn topic_term_roundtrip(
        term in "[a-z]{3,15}",
        score in 0.0f64..1.0,
        tf in 0u32..1000,
        df in 0u32..100
    ) {
        let topic = TopicTerm { term, score, tf, df };

        let json = serde_json::to_string(&topic).expect("serialize");
        let parsed: TopicTerm = serde_json::from_str(&json).expect("deserialize");

        prop_assert_eq!(topic.term, parsed.term);
        prop_assert!((topic.score - parsed.score).abs() < 1e-10);
        prop_assert_eq!(topic.tf, parsed.tf);
        prop_assert_eq!(topic.df, parsed.df);
    }

    /// EntropyFinding round-trips through JSON.
    #[test]
    fn entropy_finding_roundtrip(
        path in "[a-z/]{5,30}",
        module in "[a-z_]{3,15}",
        entropy in 0.0f32..8.0,
        sample_bytes in 0u32..10000,
        class in arb_entropy_class()
    ) {
        let finding = EntropyFinding {
            path,
            module,
            entropy_bits_per_byte: entropy,
            sample_bytes,
            class,
        };

        let json = serde_json::to_string(&finding).expect("serialize");
        let parsed: EntropyFinding = serde_json::from_str(&json).expect("deserialize");

        prop_assert_eq!(finding.path, parsed.path);
        prop_assert_eq!(finding.module, parsed.module);
        prop_assert!((finding.entropy_bits_per_byte - parsed.entropy_bits_per_byte).abs() < 1e-6);
        prop_assert_eq!(finding.sample_bytes, parsed.sample_bytes);
        prop_assert_eq!(finding.class, parsed.class);
    }

    /// ChurnTrend round-trips through JSON.
    #[test]
    fn churn_trend_roundtrip(
        slope in -10.0f64..10.0,
        r2 in 0.0f64..1.0,
        recent_change in -1000i64..1000,
        classification in arb_trend_class()
    ) {
        let trend = ChurnTrend {
            slope,
            r2,
            recent_change,
            classification,
        };

        let json = serde_json::to_string(&trend).expect("serialize");
        let parsed: ChurnTrend = serde_json::from_str(&json).expect("deserialize");

        prop_assert!((trend.slope - parsed.slope).abs() < 1e-10);
        prop_assert!((trend.r2 - parsed.r2).abs() < 1e-10);
        prop_assert_eq!(trend.recent_change, parsed.recent_change);
        prop_assert_eq!(trend.classification, parsed.classification);
    }

    /// DomainStat round-trips through JSON.
    #[test]
    fn domain_stat_roundtrip(
        domain in "[a-z]{3,10}\\.[a-z]{2,4}",
        commits in 0u32..10000,
        pct in 0.0f32..100.0
    ) {
        let stat = DomainStat { domain, commits, pct };

        let json = serde_json::to_string(&stat).expect("serialize");
        let parsed: DomainStat = serde_json::from_str(&json).expect("deserialize");

        prop_assert_eq!(stat.domain, parsed.domain);
        prop_assert_eq!(stat.commits, parsed.commits);
        prop_assert!((stat.pct - parsed.pct).abs() < 1e-6);
    }

    /// LicenseFinding round-trips through JSON.
    #[test]
    fn license_finding_roundtrip(
        spdx in "[A-Z]{2,5}-[0-9]\\.[0-9]",
        confidence in 0.0f32..1.0,
        source_path in "[a-z/]{5,20}",
        source_kind in arb_license_source_kind()
    ) {
        let finding = LicenseFinding {
            spdx,
            confidence,
            source_path,
            source_kind,
        };

        let json = serde_json::to_string(&finding).expect("serialize");
        let parsed: LicenseFinding = serde_json::from_str(&json).expect("deserialize");

        prop_assert_eq!(finding.spdx, parsed.spdx);
        prop_assert!((finding.confidence - parsed.confidence).abs() < 1e-6);
        prop_assert_eq!(finding.source_path, parsed.source_path);
        prop_assert_eq!(finding.source_kind, parsed.source_kind);
    }
}

// ============================================================================
// Report struct round-trip tests
// ============================================================================

proptest! {
    /// DerivedTotals round-trips through JSON.
    #[test]
    fn derived_totals_roundtrip(
        files in 0usize..10000,
        code in 0usize..1000000,
        comments in 0usize..100000,
        blanks in 0usize..100000,
        lines in 0usize..1000000,
        bytes in 0usize..100000000,
        tokens in 0usize..10000000
    ) {
        let totals = DerivedTotals {
            files,
            code,
            comments,
            blanks,
            lines,
            bytes,
            tokens,
        };

        let json = serde_json::to_string(&totals).expect("serialize");
        let parsed: DerivedTotals = serde_json::from_str(&json).expect("deserialize");

        prop_assert_eq!(totals.files, parsed.files);
        prop_assert_eq!(totals.code, parsed.code);
        prop_assert_eq!(totals.comments, parsed.comments);
        prop_assert_eq!(totals.blanks, parsed.blanks);
        prop_assert_eq!(totals.lines, parsed.lines);
        prop_assert_eq!(totals.bytes, parsed.bytes);
        prop_assert_eq!(totals.tokens, parsed.tokens);
    }

    /// RatioRow round-trips through JSON.
    #[test]
    fn ratio_row_roundtrip(
        key in "[a-z_]{3,15}",
        numerator in 0usize..10000,
        denominator in 1usize..10000,
        ratio in 0.0f64..1.0
    ) {
        let row = RatioRow {
            key,
            numerator,
            denominator,
            ratio,
        };

        let json = serde_json::to_string(&row).expect("serialize");
        let parsed: RatioRow = serde_json::from_str(&json).expect("deserialize");

        prop_assert_eq!(row.key, parsed.key);
        prop_assert_eq!(row.numerator, parsed.numerator);
        prop_assert_eq!(row.denominator, parsed.denominator);
        prop_assert!((row.ratio - parsed.ratio).abs() < 1e-10);
    }

    /// DistributionReport round-trips through JSON.
    #[test]
    fn distribution_report_roundtrip(
        count in 1usize..1000,
        min in 0usize..100,
        max in 100usize..10000,
        mean in 0.0f64..10000.0,
        median in 0.0f64..10000.0,
        p90 in 0.0f64..10000.0,
        p99 in 0.0f64..10000.0,
        gini in 0.0f64..1.0
    ) {
        let report = DistributionReport {
            count,
            min,
            max,
            mean,
            median,
            p90,
            p99,
            gini,
        };

        let json = serde_json::to_string(&report).expect("serialize");
        let parsed: DistributionReport = serde_json::from_str(&json).expect("deserialize");

        prop_assert_eq!(report.count, parsed.count);
        prop_assert_eq!(report.min, parsed.min);
        prop_assert_eq!(report.max, parsed.max);
        prop_assert!((report.mean - parsed.mean).abs() < 1e-10);
        prop_assert!((report.median - parsed.median).abs() < 1e-10);
        prop_assert!((report.p90 - parsed.p90).abs() < 1e-10);
        prop_assert!((report.p99 - parsed.p99).abs() < 1e-10);
        prop_assert!((report.gini - parsed.gini).abs() < 1e-10);
    }

    /// TestDensityReport round-trips through JSON.
    #[test]
    fn test_density_report_roundtrip(
        test_lines in 0usize..100000,
        prod_lines in 0usize..1000000,
        test_files in 0usize..1000,
        prod_files in 0usize..10000,
        ratio in 0.0f64..10.0
    ) {
        let report = TestDensityReport {
            test_lines,
            prod_lines,
            test_files,
            prod_files,
            ratio,
        };

        let json = serde_json::to_string(&report).expect("serialize");
        let parsed: TestDensityReport = serde_json::from_str(&json).expect("deserialize");

        prop_assert_eq!(report.test_lines, parsed.test_lines);
        prop_assert_eq!(report.prod_lines, parsed.prod_lines);
        prop_assert_eq!(report.test_files, parsed.test_files);
        prop_assert_eq!(report.prod_files, parsed.prod_files);
        prop_assert!((report.ratio - parsed.ratio).abs() < 1e-10);
    }

    /// BoilerplateReport round-trips through JSON.
    #[test]
    fn boilerplate_report_roundtrip(
        infra_lines in 0usize..100000,
        logic_lines in 0usize..1000000,
        ratio in 0.0f64..10.0,
        infra_langs in prop::collection::vec("[A-Z][a-z]{2,10}", 0..=5)
    ) {
        let report = BoilerplateReport {
            infra_lines,
            logic_lines,
            ratio,
            infra_langs,
        };

        let json = serde_json::to_string(&report).expect("serialize");
        let parsed: BoilerplateReport = serde_json::from_str(&json).expect("deserialize");

        prop_assert_eq!(report.infra_lines, parsed.infra_lines);
        prop_assert_eq!(report.logic_lines, parsed.logic_lines);
        prop_assert!((report.ratio - parsed.ratio).abs() < 1e-10);
        prop_assert_eq!(report.infra_langs, parsed.infra_langs);
    }

    /// PolyglotReport round-trips through JSON.
    #[test]
    fn polyglot_report_roundtrip(
        lang_count in 1usize..20,
        entropy in 0.0f64..5.0,
        dominant_lang in "[A-Z][a-z]{2,15}",
        dominant_lines in 0usize..1000000,
        dominant_pct in 0.0f64..100.0
    ) {
        let report = PolyglotReport {
            lang_count,
            entropy,
            dominant_lang,
            dominant_lines,
            dominant_pct,
        };

        let json = serde_json::to_string(&report).expect("serialize");
        let parsed: PolyglotReport = serde_json::from_str(&json).expect("deserialize");

        prop_assert_eq!(report.lang_count, parsed.lang_count);
        prop_assert!((report.entropy - parsed.entropy).abs() < 1e-10);
        prop_assert_eq!(report.dominant_lang, parsed.dominant_lang);
        prop_assert_eq!(report.dominant_lines, parsed.dominant_lines);
        prop_assert!((report.dominant_pct - parsed.dominant_pct).abs() < 1e-10);
    }

    /// ReadingTimeReport round-trips through JSON.
    #[test]
    fn reading_time_report_roundtrip(
        minutes in 0.0f64..10000.0,
        lines_per_minute in 1usize..500,
        basis_lines in 0usize..1000000
    ) {
        let report = ReadingTimeReport {
            minutes,
            lines_per_minute,
            basis_lines,
        };

        let json = serde_json::to_string(&report).expect("serialize");
        let parsed: ReadingTimeReport = serde_json::from_str(&json).expect("deserialize");

        prop_assert!((report.minutes - parsed.minutes).abs() < 1e-10);
        prop_assert_eq!(report.lines_per_minute, parsed.lines_per_minute);
        prop_assert_eq!(report.basis_lines, parsed.basis_lines);
    }

    /// TodoReport round-trips through JSON.
    #[test]
    fn todo_report_roundtrip(
        total in 0usize..1000,
        density_per_kloc in 0.0f64..100.0
    ) {
        let report = TodoReport {
            total,
            density_per_kloc,
            tags: vec![
                TodoTagRow { tag: "TODO".into(), count: total / 2 },
                TodoTagRow { tag: "FIXME".into(), count: total / 2 },
            ],
        };

        let json = serde_json::to_string(&report).expect("serialize");
        let parsed: TodoReport = serde_json::from_str(&json).expect("deserialize");

        prop_assert_eq!(report.total, parsed.total);
        prop_assert!((report.density_per_kloc - parsed.density_per_kloc).abs() < 1e-10);
        prop_assert_eq!(report.tags.len(), parsed.tags.len());
    }

    /// ContextWindowReport round-trips through JSON.
    #[test]
    fn context_window_report_roundtrip(
        window_tokens in 1usize..1000000,
        total_tokens in 0usize..10000000,
        pct in 0.0f64..1000.0,
        fits in any::<bool>()
    ) {
        let report = ContextWindowReport {
            window_tokens,
            total_tokens,
            pct,
            fits,
        };

        let json = serde_json::to_string(&report).expect("serialize");
        let parsed: ContextWindowReport = serde_json::from_str(&json).expect("deserialize");

        prop_assert_eq!(report.window_tokens, parsed.window_tokens);
        prop_assert_eq!(report.total_tokens, parsed.total_tokens);
        prop_assert!((report.pct - parsed.pct).abs() < 1e-10);
        prop_assert_eq!(report.fits, parsed.fits);
    }

    /// CocomoReport round-trips through JSON.
    #[test]
    fn cocomo_report_roundtrip(
        kloc in 0.1f64..1000.0,
        effort_pm in 0.0f64..10000.0,
        duration_months in 0.0f64..100.0,
        staff in 0.0f64..100.0
    ) {
        let report = CocomoReport {
            mode: "organic".into(),
            kloc,
            effort_pm,
            duration_months,
            staff,
            a: 2.4,
            b: 1.05,
            c: 2.5,
            d: 0.38,
        };

        let json = serde_json::to_string(&report).expect("serialize");
        let parsed: CocomoReport = serde_json::from_str(&json).expect("deserialize");

        prop_assert_eq!(report.mode, parsed.mode);
        prop_assert!((report.kloc - parsed.kloc).abs() < 1e-10);
        prop_assert!((report.effort_pm - parsed.effort_pm).abs() < 1e-10);
        prop_assert!((report.duration_months - parsed.duration_months).abs() < 1e-10);
        prop_assert!((report.staff - parsed.staff).abs() < 1e-10);
    }

    /// IntegrityReport round-trips through JSON.
    #[test]
    fn integrity_report_roundtrip(
        hash in "[a-f0-9]{64}",
        entries in 0usize..10000
    ) {
        let report = IntegrityReport {
            algo: "blake3".into(),
            hash,
            entries,
        };

        let json = serde_json::to_string(&report).expect("serialize");
        let parsed: IntegrityReport = serde_json::from_str(&json).expect("deserialize");

        prop_assert_eq!(report.algo, parsed.algo);
        prop_assert_eq!(report.hash, parsed.hash);
        prop_assert_eq!(report.entries, parsed.entries);
    }
}

// ============================================================================
// Git report types
// ============================================================================

proptest! {
    /// HotspotRow round-trips through JSON.
    #[test]
    fn hotspot_row_roundtrip(
        path in "[a-z/]{5,30}",
        commits in 0usize..1000,
        lines in 0usize..10000,
        score in 0usize..100000
    ) {
        let row = HotspotRow {
            path,
            commits,
            lines,
            score,
        };

        let json = serde_json::to_string(&row).expect("serialize");
        let parsed: HotspotRow = serde_json::from_str(&json).expect("deserialize");

        prop_assert_eq!(row.path, parsed.path);
        prop_assert_eq!(row.commits, parsed.commits);
        prop_assert_eq!(row.lines, parsed.lines);
        prop_assert_eq!(row.score, parsed.score);
    }

    /// BusFactorRow round-trips through JSON.
    #[test]
    fn bus_factor_row_roundtrip(
        module in "[a-z_]{3,15}",
        authors in 1usize..50
    ) {
        let row = BusFactorRow { module, authors };

        let json = serde_json::to_string(&row).expect("serialize");
        let parsed: BusFactorRow = serde_json::from_str(&json).expect("deserialize");

        prop_assert_eq!(row.module, parsed.module);
        prop_assert_eq!(row.authors, parsed.authors);
    }

    /// CouplingRow round-trips through JSON.
    #[test]
    fn coupling_row_roundtrip(
        left in "[a-z/]{5,20}",
        right in "[a-z/]{5,20}",
        count in 0usize..100
    ) {
        let row = CouplingRow { left, right, count, jaccard: Some(0.5), lift: Some(1.2), n_left: Some(10), n_right: Some(8) };

        let json = serde_json::to_string(&row).expect("serialize");
        let parsed: CouplingRow = serde_json::from_str(&json).expect("deserialize");

        prop_assert_eq!(row.left, parsed.left);
        prop_assert_eq!(row.right, parsed.right);
        prop_assert_eq!(row.count, parsed.count);
        prop_assert_eq!(row.jaccard, parsed.jaccard);
        prop_assert_eq!(row.lift, parsed.lift);
    }

    /// ImportEdge round-trips through JSON.
    #[test]
    fn import_edge_roundtrip(
        from in "[a-z/]{3,20}",
        to in "[a-z/]{3,20}",
        count in 1usize..100
    ) {
        let edge = ImportEdge { from, to, count };

        let json = serde_json::to_string(&edge).expect("serialize");
        let parsed: ImportEdge = serde_json::from_str(&json).expect("deserialize");

        prop_assert_eq!(edge.from, parsed.from);
        prop_assert_eq!(edge.to, parsed.to);
        prop_assert_eq!(edge.count, parsed.count);
    }
}

// ============================================================================
// Asset and dependency types
// ============================================================================

proptest! {
    /// AssetFileRow round-trips through JSON.
    #[test]
    fn asset_file_row_roundtrip(
        path in "[a-z/]{5,30}",
        bytes in 0u64..100000000,
        category in "[a-z]{3,10}",
        extension in "[a-z]{1,5}"
    ) {
        let row = AssetFileRow {
            path,
            bytes,
            category,
            extension,
        };

        let json = serde_json::to_string(&row).expect("serialize");
        let parsed: AssetFileRow = serde_json::from_str(&json).expect("deserialize");

        prop_assert_eq!(row.path, parsed.path);
        prop_assert_eq!(row.bytes, parsed.bytes);
        prop_assert_eq!(row.category, parsed.category);
        prop_assert_eq!(row.extension, parsed.extension);
    }

    /// LockfileReport round-trips through JSON.
    #[test]
    fn lockfile_report_roundtrip(
        path in "[a-z.-]{5,25}",
        kind in "[a-z]{3,10}",
        dependencies in 0usize..1000
    ) {
        let report = LockfileReport {
            path,
            kind,
            dependencies,
        };

        let json = serde_json::to_string(&report).expect("serialize");
        let parsed: LockfileReport = serde_json::from_str(&json).expect("deserialize");

        prop_assert_eq!(report.path, parsed.path);
        prop_assert_eq!(report.kind, parsed.kind);
        prop_assert_eq!(report.dependencies, parsed.dependencies);
    }

    /// DuplicateGroup round-trips through JSON.
    #[test]
    fn duplicate_group_roundtrip(
        hash in "[a-f0-9]{64}",
        bytes in 0u64..100000,
        files in prop::collection::vec("[a-z/]{5,20}", 2..=5)
    ) {
        let group = DuplicateGroup { hash, bytes, files };

        let json = serde_json::to_string(&group).expect("serialize");
        let parsed: DuplicateGroup = serde_json::from_str(&json).expect("deserialize");

        prop_assert_eq!(group.hash, parsed.hash);
        prop_assert_eq!(group.bytes, parsed.bytes);
        prop_assert_eq!(group.files, parsed.files);
    }
}

// ============================================================================
// Fun types
// ============================================================================

proptest! {
    /// EcoLabel round-trips through JSON.
    #[test]
    fn eco_label_roundtrip(
        score in 0.0f64..100.0,
        label in "[A-F]",
        bytes in 0u64..100000000,
        notes in "[a-z ]{10,50}"
    ) {
        let eco = EcoLabel {
            score,
            label,
            bytes,
            notes,
        };

        let json = serde_json::to_string(&eco).expect("serialize");
        let parsed: EcoLabel = serde_json::from_str(&json).expect("deserialize");

        prop_assert!((eco.score - parsed.score).abs() < 1e-10);
        prop_assert_eq!(eco.label, parsed.label);
        prop_assert_eq!(eco.bytes, parsed.bytes);
        prop_assert_eq!(eco.notes, parsed.notes);
    }
}

// ============================================================================
// Schema version constant test
// ============================================================================

proptest! {
    /// Schema version is a valid positive number.
    #[test]
    fn schema_version_is_valid(_dummy in 0..1u8) {
        prop_assert!(ANALYSIS_SCHEMA_VERSION > 0);
        prop_assert!(ANALYSIS_SCHEMA_VERSION <= 100); // Reasonable upper bound
    }
}

// ============================================================================
// Strategies
// ============================================================================

fn arb_entropy_class() -> impl Strategy<Value = EntropyClass> {
    prop_oneof![
        Just(EntropyClass::Low),
        Just(EntropyClass::Normal),
        Just(EntropyClass::Suspicious),
        Just(EntropyClass::High),
    ]
}

fn arb_trend_class() -> impl Strategy<Value = TrendClass> {
    prop_oneof![
        Just(TrendClass::Rising),
        Just(TrendClass::Flat),
        Just(TrendClass::Falling),
    ]
}

fn arb_license_source_kind() -> impl Strategy<Value = LicenseSourceKind> {
    prop_oneof![
        Just(LicenseSourceKind::Metadata),
        Just(LicenseSourceKind::Text),
    ]
}
