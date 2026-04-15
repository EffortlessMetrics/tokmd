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
// Near-duplicate types
// ============================================================================

proptest! {
    /// NearDupAlgorithm round-trips through JSON.
    #[test]
    fn near_dup_algorithm_roundtrip(
        k_gram_size in 1usize..100,
        window_size in 1usize..20,
        max_postings in 1usize..1000
    ) {
        let algo = NearDupAlgorithm { k_gram_size, window_size, max_postings };

        let json = serde_json::to_string(&algo).expect("serialize");
        let parsed: NearDupAlgorithm = serde_json::from_str(&json).expect("deserialize");

        prop_assert_eq!(algo, parsed);
    }

    /// NearDupCluster round-trips through JSON.
    #[test]
    fn near_dup_cluster_roundtrip(
        files in prop::collection::vec("[a-z/]{5,20}", 2..=5),
        max_similarity in 0.0f64..1.0,
        representative in "[a-z/]{5,20}",
        pair_count in 1usize..100
    ) {
        let cluster = NearDupCluster {
            files,
            max_similarity,
            representative,
            pair_count,
        };

        let json = serde_json::to_string(&cluster).expect("serialize");
        let parsed: NearDupCluster = serde_json::from_str(&json).expect("deserialize");

        prop_assert_eq!(cluster.files, parsed.files);
        prop_assert!((cluster.max_similarity - parsed.max_similarity).abs() < 1e-10);
        prop_assert_eq!(cluster.representative, parsed.representative);
        prop_assert_eq!(cluster.pair_count, parsed.pair_count);
    }

    /// NearDupStats round-trips through JSON.
    #[test]
    fn near_dup_stats_roundtrip(
        fingerprinting_ms in 0u64..100000,
        pairing_ms in 0u64..100000,
        bytes_processed in 0u64..100000000
    ) {
        let stats = NearDupStats { fingerprinting_ms, pairing_ms, bytes_processed };

        let json = serde_json::to_string(&stats).expect("serialize");
        let parsed: NearDupStats = serde_json::from_str(&json).expect("deserialize");

        prop_assert_eq!(stats, parsed);
    }

    /// NearDupPairRow round-trips through JSON.
    #[test]
    fn near_dup_pair_row_roundtrip(
        left in "[a-z/]{5,20}",
        right in "[a-z/]{5,20}",
        similarity in 0.0f64..1.0,
        shared in 0usize..1000,
        left_fps in 0usize..10000,
        right_fps in 0usize..10000
    ) {
        let row = NearDupPairRow {
            left,
            right,
            similarity,
            shared_fingerprints: shared,
            left_fingerprints: left_fps,
            right_fingerprints: right_fps,
        };

        let json = serde_json::to_string(&row).expect("serialize");
        let parsed: NearDupPairRow = serde_json::from_str(&json).expect("deserialize");

        prop_assert_eq!(row.left, parsed.left);
        prop_assert_eq!(row.right, parsed.right);
        prop_assert!((row.similarity - parsed.similarity).abs() < 1e-10);
        prop_assert_eq!(row.shared_fingerprints, parsed.shared_fingerprints);
    }
}

// ============================================================================

// ============================================================================
// Effort types
// ============================================================================

fn arb_effort_model() -> impl Strategy<Value = EffortModel> {
    prop_oneof![
        Just(EffortModel::Cocomo81Basic),
        Just(EffortModel::Cocomo2Early),
        Just(EffortModel::Ensemble),
    ]
}

fn arb_effort_confidence_level() -> impl Strategy<Value = EffortConfidenceLevel> {
    prop_oneof![
        Just(EffortConfidenceLevel::Low),
        Just(EffortConfidenceLevel::Medium),
        Just(EffortConfidenceLevel::High),
    ]
}

fn arb_effort_driver_direction() -> impl Strategy<Value = EffortDriverDirection> {
    prop_oneof![
        Just(EffortDriverDirection::Raises),
        Just(EffortDriverDirection::Lowers),
        Just(EffortDriverDirection::Neutral),
    ]
}

fn arb_effort_delta_classification() -> impl Strategy<Value = EffortDeltaClassification> {
    prop_oneof![
        Just(EffortDeltaClassification::Low),
        Just(EffortDeltaClassification::Medium),
        Just(EffortDeltaClassification::High),
        Just(EffortDeltaClassification::Critical),
    ]
}

proptest! {


    /// EffortEstimateReport round-trips through JSON.
    #[test]
    fn effort_estimate_report_roundtrip(
        model in arb_effort_model(),
        total_lines in 0usize..100000,
        effort_pm_p50 in 0.0f64..1000.0,
        level in arb_effort_confidence_level()
    ) {
        let size_basis = tokmd_analysis_types::EffortSizeBasis {
            total_lines,
            authored_lines: total_lines,
            generated_lines: 0,
            vendored_lines: 0,
            kloc_total: total_lines as f64 / 1000.0,
            kloc_authored: total_lines as f64 / 1000.0,
            generated_pct: 0.0,
            vendored_pct: 0.0,
            classification_confidence: level,
            warnings: vec![],
            by_tag: vec![],
        };

        let results = tokmd_analysis_types::EffortResults {
            effort_pm_p50,
            schedule_months_p50: 1.0,
            staff_p50: 1.0,
            effort_pm_low: 1.0,
            effort_pm_p80: 1.0,
            schedule_months_low: 1.0,
            schedule_months_p80: 1.0,
            staff_low: 1.0,
            staff_p80: 1.0,
        };

        let confidence = EffortConfidence {
            level,
            reasons: vec![],
            data_coverage_pct: None,
        };

        let assumptions = tokmd_analysis_types::EffortAssumptions {
            notes: vec![],
            overrides: std::collections::BTreeMap::new(),
        };

        let report = EffortEstimateReport {
            model,
            size_basis,
            results,
            confidence,
            drivers: vec![],
            assumptions,
            delta: None,
        };

        let json = serde_json::to_string(&report).expect("serialize");
        let parsed: EffortEstimateReport = serde_json::from_str(&json).expect("deserialize");

        prop_assert_eq!(report.model.to_string(), parsed.model.to_string());
    }
    /// EffortResults round-trips through JSON.
    #[test]
    fn effort_results_roundtrip(
        effort_pm_p50 in 0.0f64..1000.0,
        schedule_months_p50 in 0.0f64..100.0,
        staff_p50 in 0.0f64..100.0,
        effort_pm_low in 0.0f64..1000.0,
        effort_pm_p80 in 0.0f64..1000.0,
        schedule_months_low in 0.0f64..100.0,
        schedule_months_p80 in 0.0f64..100.0,
        staff_low in 0.0f64..100.0,
        staff_p80 in 0.0f64..100.0
    ) {
        let results = tokmd_analysis_types::EffortResults {
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

        let json = serde_json::to_string(&results).expect("serialize");
        let parsed: tokmd_analysis_types::EffortResults = serde_json::from_str(&json).expect("deserialize");

        prop_assert!((results.effort_pm_p50 - parsed.effort_pm_p50).abs() < 1e-10);
        prop_assert!((results.schedule_months_p50 - parsed.schedule_months_p50).abs() < 1e-10);
        prop_assert!((results.staff_p50 - parsed.staff_p50).abs() < 1e-10);
    }
    /// EffortDriver round-trips through JSON.
    #[test]
    fn effort_driver_roundtrip(
        key in "[a-z_]{3,15}",
        label in "[a-z A-Z]{5,30}",
        weight in 0.1f64..10.0,
        direction in arb_effort_driver_direction(),
        evidence in "[a-z A-Z0-9]{10,50}"
    ) {
        let driver = EffortDriver {
            key,
            label,
            weight,
            direction,
            evidence,
        };

        let json = serde_json::to_string(&driver).expect("serialize");
        let parsed: EffortDriver = serde_json::from_str(&json).expect("deserialize");

        prop_assert_eq!(driver.key, parsed.key);
        prop_assert_eq!(driver.label, parsed.label);
        prop_assert!((driver.weight - parsed.weight).abs() < 1e-10);
        prop_assert_eq!(driver.direction, parsed.direction);
        prop_assert_eq!(driver.evidence, parsed.evidence);
    }

    /// EffortTagSizeRow round-trips through JSON.
    #[test]
    fn effort_tag_size_row_roundtrip(
        tag in "[a-z0-9_-]{2,15}",
        lines in 0usize..100000,
        authored_lines in 0usize..100000,
        pct_of_total in 0.0f64..1.0
    ) {
        let row = EffortTagSizeRow {
            tag,
            lines,
            authored_lines,
            pct_of_total,
        };

        let json = serde_json::to_string(&row).expect("serialize");
        let parsed: EffortTagSizeRow = serde_json::from_str(&json).expect("deserialize");

        prop_assert_eq!(row.tag, parsed.tag);
        prop_assert_eq!(row.lines, parsed.lines);
        prop_assert_eq!(row.authored_lines, parsed.authored_lines);
        prop_assert!((row.pct_of_total - parsed.pct_of_total).abs() < 1e-10);
    }

    /// EffortConfidence round-trips through JSON.
    #[test]
    fn effort_confidence_roundtrip(
        level in arb_effort_confidence_level(),
        reasons in prop::collection::vec("[a-z ]{5,30}", 0..5),
        data_coverage_pct in prop::option::of(0.0f64..1.0)
    ) {
        let conf = EffortConfidence {
            level,
            reasons,
            data_coverage_pct,
        };

        let json = serde_json::to_string(&conf).expect("serialize");
        let parsed: EffortConfidence = serde_json::from_str(&json).expect("deserialize");

        prop_assert_eq!(conf.level, parsed.level);
        prop_assert_eq!(conf.reasons, parsed.reasons);
        if let (Some(a), Some(b)) = (conf.data_coverage_pct, parsed.data_coverage_pct) {
            prop_assert!((a - b).abs() < 1e-10);
        } else {
            prop_assert_eq!(conf.data_coverage_pct, parsed.data_coverage_pct);
        }
    }

    /// EffortDeltaReport round-trips through JSON.
    #[test]
    fn effort_delta_report_roundtrip(
        base in "[a-f0-9]{40}",
        head in "[a-f0-9]{40}",
        files_changed in 0usize..1000,
        modules_changed in 0usize..100,
        langs_changed in 0usize..20,
        hotspot_files_touched in 0usize..100,
        coupled_neighbors_touched in 0usize..100,
        blast_radius in 0.0f64..1.0,
        classification in arb_effort_delta_classification(),
        effort_pm_low in 0.0f64..1000.0,
        effort_pm_est in 0.0f64..1000.0,
        effort_pm_high in 0.0f64..1000.0
    ) {
        let delta = EffortDeltaReport {
            base,
            head,
            files_changed,
            modules_changed,
            langs_changed,
            hotspot_files_touched,
            coupled_neighbors_touched,
            blast_radius,
            classification,
            effort_pm_low,
            effort_pm_est,
            effort_pm_high,
        };

        let json = serde_json::to_string(&delta).expect("serialize");
        let parsed: EffortDeltaReport = serde_json::from_str(&json).expect("deserialize");

        prop_assert_eq!(delta.base, parsed.base);
        prop_assert_eq!(delta.head, parsed.head);
        prop_assert_eq!(delta.files_changed, parsed.files_changed);
        prop_assert_eq!(delta.modules_changed, parsed.modules_changed);
        prop_assert_eq!(delta.langs_changed, parsed.langs_changed);
        prop_assert_eq!(delta.hotspot_files_touched, parsed.hotspot_files_touched);
        prop_assert_eq!(delta.coupled_neighbors_touched, parsed.coupled_neighbors_touched);
        prop_assert!((delta.blast_radius - parsed.blast_radius).abs() < 1e-10);
        prop_assert_eq!(delta.classification, parsed.classification);
        prop_assert!((delta.effort_pm_low - parsed.effort_pm_low).abs() < 1e-10);
        prop_assert!((delta.effort_pm_est - parsed.effort_pm_est).abs() < 1e-10);
        prop_assert!((delta.effort_pm_high - parsed.effort_pm_high).abs() < 1e-10);
    }
}

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
