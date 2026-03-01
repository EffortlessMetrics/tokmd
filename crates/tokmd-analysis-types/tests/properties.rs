//! Property-based tests for tokmd-analysis-types.
//!
//! These tests verify JSON serialization round-trips for all analysis types,
//! ensuring data integrity and schema stability.

use std::collections::BTreeMap;

use proptest::prelude::*;
use tokmd_analysis_types::*;
use tokmd_types::{ScanStatus, ToolInfo};

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
// Missing enum round-trip tests
// ============================================================================

proptest! {
    /// NearDupScope round-trips through JSON.
    #[test]
    fn near_dup_scope_roundtrip(variant in arb_near_dup_scope()) {
        let json = serde_json::to_string(&variant).expect("serialize");
        let parsed: NearDupScope = serde_json::from_str(&json).expect("deserialize");
        prop_assert_eq!(variant, parsed);
    }

    /// NearDupScope serializes to kebab-case.
    #[test]
    fn near_dup_scope_kebab_case(variant in arb_near_dup_scope()) {
        let json = serde_json::to_string(&variant).expect("serialize");
        let s = json.trim_matches('"');
        prop_assert!(
            !s.chars().any(|c| c.is_uppercase()),
            "NearDupScope should be kebab-case: {}",
            s
        );
    }

    /// TechnicalDebtLevel round-trips through JSON.
    #[test]
    fn technical_debt_level_roundtrip(variant in arb_technical_debt_level()) {
        let json = serde_json::to_string(&variant).expect("serialize");
        let parsed: TechnicalDebtLevel = serde_json::from_str(&json).expect("deserialize");
        prop_assert_eq!(variant, parsed);
    }

    /// TechnicalDebtLevel serializes to snake_case.
    #[test]
    fn technical_debt_level_snake_case(variant in arb_technical_debt_level()) {
        let json = serde_json::to_string(&variant).expect("serialize");
        let s = json.trim_matches('"');
        prop_assert!(
            !s.chars().any(|c| c.is_uppercase()),
            "TechnicalDebtLevel should be snake_case: {}",
            s
        );
    }

    /// ComplexityRisk round-trips through JSON.
    #[test]
    fn complexity_risk_roundtrip(variant in arb_complexity_risk()) {
        let json = serde_json::to_string(&variant).expect("serialize");
        let parsed: ComplexityRisk = serde_json::from_str(&json).expect("deserialize");
        prop_assert_eq!(variant, parsed);
    }

    /// ComplexityRisk serializes to snake_case.
    #[test]
    fn complexity_risk_snake_case(variant in arb_complexity_risk()) {
        let json = serde_json::to_string(&variant).expect("serialize");
        let s = json.trim_matches('"');
        prop_assert!(
            !s.chars().any(|c| c.is_uppercase()),
            "ComplexityRisk should be snake_case: {}",
            s
        );
    }
}

// ============================================================================
// AnalysisReceipt round-trip tests
// ============================================================================

proptest! {
    /// AnalysisReceipt with all-None optional fields round-trips through JSON.
    #[test]
    fn analysis_receipt_minimal_roundtrip(
        mode in "[a-z]{3,10}",
        preset in "[a-z]{3,10}",
        format in "[a-z]{3,6}",
    ) {
        let receipt = AnalysisReceipt {
            schema_version: ANALYSIS_SCHEMA_VERSION,
            generated_at_ms: 0,
            tool: ToolInfo { name: "tokmd".into(), version: "0.0.0".into() },
            mode,
            status: ScanStatus::Complete,
            warnings: Vec::new(),
            source: AnalysisSource {
                inputs: vec![".".into()],
                export_path: None,
                base_receipt_path: None,
                export_schema_version: None,
                export_generated_at_ms: None,
                base_signature: None,
                module_roots: vec!["src".into()],
                module_depth: 1,
                children: "separate".into(),
            },
            args: AnalysisArgsMeta {
                preset,
                format,
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
            complexity: None,
            api_surface: None,
            fun: None,
        };

        let json = serde_json::to_string(&receipt).expect("serialize");
        let parsed: AnalysisReceipt = serde_json::from_str(&json).expect("deserialize");

        prop_assert_eq!(receipt.schema_version, parsed.schema_version);
        prop_assert_eq!(receipt.mode, parsed.mode);
        prop_assert_eq!(receipt.warnings.len(), parsed.warnings.len());
        prop_assert!(parsed.archetype.is_none());
        prop_assert!(parsed.topics.is_none());
        prop_assert!(parsed.derived.is_none());
        prop_assert!(parsed.complexity.is_none());
        prop_assert!(parsed.fun.is_none());
    }

    /// AnalysisReceipt with warnings vector round-trips through JSON.
    #[test]
    fn analysis_receipt_warnings_roundtrip(
        warnings in prop::collection::vec("[a-z ]{5,30}", 0..=10),
    ) {
        let receipt = AnalysisReceipt {
            schema_version: ANALYSIS_SCHEMA_VERSION,
            generated_at_ms: 42,
            tool: ToolInfo { name: "tokmd".into(), version: "1.0.0".into() },
            mode: "analysis".into(),
            status: ScanStatus::Partial,
            warnings: warnings.clone(),
            source: AnalysisSource {
                inputs: vec![".".into()],
                export_path: None,
                base_receipt_path: None,
                export_schema_version: None,
                export_generated_at_ms: None,
                base_signature: None,
                module_roots: Vec::new(),
                module_depth: 1,
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
            complexity: None,
            api_surface: None,
            fun: None,
        };

        let json = serde_json::to_string(&receipt).expect("serialize");
        let parsed: AnalysisReceipt = serde_json::from_str(&json).expect("deserialize");

        prop_assert_eq!(&warnings, &parsed.warnings);
    }
}

// ============================================================================
// AnalysisSource and AnalysisArgsMeta round-trip tests
// ============================================================================

proptest! {
    /// AnalysisSource round-trips through JSON.
    #[test]
    fn analysis_source_roundtrip(
        inputs in prop::collection::vec("[a-z./]{1,15}", 1..=3),
        module_roots in prop::collection::vec("[a-z]{2,10}", 0..=3),
        module_depth in 0usize..5,
        children in prop_oneof![Just("separate".to_string()), Just("collapse".to_string())],
    ) {
        let source = AnalysisSource {
            inputs,
            export_path: Some("export.jsonl".into()),
            base_receipt_path: None,
            export_schema_version: Some(2),
            export_generated_at_ms: Some(12345),
            base_signature: None,
            module_roots,
            module_depth,
            children,
        };

        let json = serde_json::to_string(&source).expect("serialize");
        let parsed: AnalysisSource = serde_json::from_str(&json).expect("deserialize");

        prop_assert_eq!(source.inputs, parsed.inputs);
        prop_assert_eq!(source.export_path, parsed.export_path);
        prop_assert_eq!(source.export_schema_version, parsed.export_schema_version);
        prop_assert_eq!(source.module_roots, parsed.module_roots);
        prop_assert_eq!(source.module_depth, parsed.module_depth);
        prop_assert_eq!(source.children, parsed.children);
    }

    /// AnalysisArgsMeta round-trips through JSON.
    #[test]
    fn analysis_args_meta_roundtrip(
        preset in "[a-z]{3,10}",
        format in "[a-z]{3,6}",
        window_tokens in prop::option::of(1usize..1000000),
        git in prop::option::of(any::<bool>()),
        max_files in prop::option::of(1usize..10000),
    ) {
        let args = AnalysisArgsMeta {
            preset,
            format,
            window_tokens,
            git,
            max_files,
            max_bytes: None,
            max_commits: Some(500),
            max_commit_files: None,
            max_file_bytes: None,
            import_granularity: "file".into(),
        };

        let json = serde_json::to_string(&args).expect("serialize");
        let parsed: AnalysisArgsMeta = serde_json::from_str(&json).expect("deserialize");

        prop_assert_eq!(args.preset, parsed.preset);
        prop_assert_eq!(args.format, parsed.format);
        prop_assert_eq!(args.window_tokens, parsed.window_tokens);
        prop_assert_eq!(args.git, parsed.git);
        prop_assert_eq!(args.max_files, parsed.max_files);
        prop_assert_eq!(args.max_commits, parsed.max_commits);
        prop_assert_eq!(args.import_granularity, parsed.import_granularity);
    }
}

// ============================================================================
// Default value tests
// ============================================================================

proptest! {
    /// Default ComplexityBaseline serializes to valid JSON.
    #[test]
    fn default_complexity_baseline_valid_json(_dummy in 0..1u8) {
        let baseline = ComplexityBaseline::default();
        let json = serde_json::to_string(&baseline).expect("serialize");
        let _: serde_json::Value = serde_json::from_str(&json).expect("valid JSON");
    }

    /// Default BaselineMetrics round-trips through JSON.
    #[test]
    fn default_baseline_metrics_roundtrip(_dummy in 0..1u8) {
        let metrics = BaselineMetrics::default();
        let json = serde_json::to_string(&metrics).expect("serialize");
        let parsed: BaselineMetrics = serde_json::from_str(&json).expect("deserialize");
        prop_assert_eq!(metrics.total_code_lines, parsed.total_code_lines);
        prop_assert_eq!(metrics.total_files, parsed.total_files);
        prop_assert_eq!(metrics.max_cyclomatic, parsed.max_cyclomatic);
    }

    /// Default CommitIntentCounts round-trips through JSON.
    #[test]
    fn default_commit_intent_counts_roundtrip(_dummy in 0..1u8) {
        let counts = CommitIntentCounts::default();
        let json = serde_json::to_string(&counts).expect("serialize");
        let parsed: CommitIntentCounts = serde_json::from_str(&json).expect("deserialize");
        prop_assert_eq!(counts.total, parsed.total);
        prop_assert_eq!(counts.feat, parsed.feat);
        prop_assert_eq!(counts.fix, parsed.fix);
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
        prop_assert!(ANALYSIS_SCHEMA_VERSION <= 100);
    }

    /// Baseline version is a valid positive number.
    #[test]
    fn baseline_version_is_valid(_dummy in 0..1u8) {
        prop_assert!(BASELINE_VERSION > 0);
        prop_assert!(BASELINE_VERSION <= 100);
    }
}

// ============================================================================
// Nested structure round-trip tests
// ============================================================================

proptest! {
    /// DerivedReport round-trips through JSON with nested sub-reports.
    #[test]
    fn derived_report_roundtrip(
        files in 1usize..100,
        code in 1usize..10000,
        comments in 0usize..1000,
        blanks in 0usize..1000,
        gini in 0.0f64..1.0,
    ) {
        let totals = DerivedTotals {
            files, code, comments, blanks,
            lines: code + comments + blanks,
            bytes: code * 40,
            tokens: code * 4,
        };
        let ratio_row = RatioRow {
            key: "total".into(),
            numerator: comments,
            denominator: code.max(1),
            ratio: comments as f64 / code.max(1) as f64,
        };
        let report = DerivedReport {
            totals,
            doc_density: RatioReport {
                total: ratio_row.clone(),
                by_lang: vec![ratio_row.clone()],
                by_module: Vec::new(),
            },
            whitespace: RatioReport {
                total: ratio_row.clone(),
                by_lang: Vec::new(),
                by_module: Vec::new(),
            },
            verbosity: RateReport {
                total: RateRow { key: "total".into(), numerator: code * 40, denominator: code.max(1), rate: 40.0 },
                by_lang: Vec::new(),
                by_module: Vec::new(),
            },
            max_file: MaxFileReport {
                overall: FileStatRow {
                    path: "src/main.rs".into(), module: "src".into(), lang: "Rust".into(),
                    code, comments, blanks, lines: code + comments + blanks,
                    bytes: code * 40, tokens: code * 4,
                    doc_pct: Some(0.1), bytes_per_line: Some(40.0), depth: 1,
                },
                by_lang: Vec::new(),
                by_module: Vec::new(),
            },
            lang_purity: LangPurityReport { rows: Vec::new() },
            nesting: NestingReport { max: 3, avg: 1.5, by_module: Vec::new() },
            test_density: TestDensityReport {
                test_lines: 100, prod_lines: code, test_files: 5, prod_files: files, ratio: 0.1,
            },
            boilerplate: BoilerplateReport {
                infra_lines: 50, logic_lines: code, ratio: 0.05, infra_langs: vec!["TOML".into()],
            },
            polyglot: PolyglotReport {
                lang_count: 2, entropy: 0.5, dominant_lang: "Rust".into(),
                dominant_lines: code, dominant_pct: 90.0,
            },
            distribution: DistributionReport {
                count: files, min: 1, max: code, mean: 50.0, median: 40.0,
                p90: 100.0, p99: 200.0, gini,
            },
            histogram: vec![HistogramBucket {
                label: "0-99".into(), min: 0, max: Some(99), files: files / 2, pct: 50.0,
            }],
            top: TopOffenders {
                largest_lines: Vec::new(),
                largest_tokens: Vec::new(),
                largest_bytes: Vec::new(),
                least_documented: Vec::new(),
                most_dense: Vec::new(),
            },
            tree: None,
            reading_time: ReadingTimeReport { minutes: 10.0, lines_per_minute: 200, basis_lines: code },
            context_window: Some(ContextWindowReport {
                window_tokens: 128000, total_tokens: code * 4, pct: 50.0, fits: true,
            }),
            cocomo: Some(CocomoReport {
                mode: "organic".into(), kloc: code as f64 / 1000.0,
                effort_pm: 5.0, duration_months: 3.0, staff: 1.5,
                a: 2.4, b: 1.05, c: 2.5, d: 0.38,
            }),
            todo: Some(TodoReport {
                total: 5, density_per_kloc: 2.5,
                tags: vec![TodoTagRow { tag: "TODO".into(), count: 3 }],
            }),
            integrity: IntegrityReport { algo: "blake3".into(), hash: "abc123".into(), entries: files },
        };

        let json = serde_json::to_string(&report).expect("serialize");
        let parsed: DerivedReport = serde_json::from_str(&json).expect("deserialize");

        prop_assert_eq!(report.totals.files, parsed.totals.files);
        prop_assert_eq!(report.totals.code, parsed.totals.code);
        prop_assert_eq!(report.doc_density.by_lang.len(), parsed.doc_density.by_lang.len());
        prop_assert_eq!(report.histogram.len(), parsed.histogram.len());
        prop_assert!(parsed.cocomo.is_some());
        prop_assert!(parsed.context_window.is_some());
        prop_assert!(parsed.todo.is_some());
        prop_assert_eq!(report.integrity.entries, parsed.integrity.entries);
    }

    /// ComplexityReport round-trips through JSON.
    #[test]
    fn complexity_report_roundtrip(
        total_functions in 1usize..100,
        avg_cyclomatic in 1.0f64..20.0,
        max_cyclomatic in 1usize..50,
        risk in arb_complexity_risk(),
    ) {
        let report = ComplexityReport {
            total_functions,
            avg_function_length: 15.0,
            max_function_length: 50,
            avg_cyclomatic,
            max_cyclomatic,
            avg_cognitive: Some(3.5),
            max_cognitive: Some(12),
            avg_nesting_depth: Some(1.2),
            max_nesting_depth: Some(4),
            high_risk_files: 2,
            histogram: Some(ComplexityHistogram {
                buckets: vec![0, 5, 10],
                counts: vec![10, 5, 2],
                total: 17,
            }),
            halstead: None,
            maintainability_index: None,
            technical_debt: None,
            files: vec![FileComplexity {
                path: "src/lib.rs".into(),
                module: "src".into(),
                function_count: 5,
                max_function_length: 30,
                cyclomatic_complexity: max_cyclomatic,
                cognitive_complexity: Some(8),
                max_nesting: Some(3),
                risk_level: risk,
                functions: Some(vec![FunctionComplexityDetail {
                    name: "process".into(),
                    line_start: 10, line_end: 25, length: 16,
                    cyclomatic: 4, cognitive: Some(3),
                    max_nesting: Some(2), param_count: Some(3),
                }]),
            }],
        };

        let json = serde_json::to_string(&report).expect("serialize");
        let parsed: ComplexityReport = serde_json::from_str(&json).expect("deserialize");

        prop_assert_eq!(report.total_functions, parsed.total_functions);
        prop_assert!((report.avg_cyclomatic - parsed.avg_cyclomatic).abs() < 1e-10);
        prop_assert_eq!(report.max_cyclomatic, parsed.max_cyclomatic);
        prop_assert_eq!(report.files.len(), parsed.files.len());
        prop_assert_eq!(report.files[0].risk_level, parsed.files[0].risk_level);
        prop_assert!(parsed.histogram.is_some());
        prop_assert!(parsed.files[0].functions.is_some());
    }

    /// DuplicationDensityReport round-trips through JSON.
    #[test]
    fn duplication_density_report_roundtrip(
        groups in 0usize..100,
        dup_files in 0usize..200,
        wasted_pct in 0.0f64..100.0,
    ) {
        let report = DuplicationDensityReport {
            duplicate_groups: groups,
            duplicate_files: dup_files,
            duplicated_bytes: 5000,
            wasted_bytes: 2500,
            wasted_pct_of_codebase: wasted_pct,
            by_module: vec![ModuleDuplicationDensityRow {
                module: "src".into(),
                duplicate_files: 2,
                wasted_files: 1,
                duplicated_bytes: 1000,
                wasted_bytes: 500,
                module_bytes: 10000,
                density: 0.05,
            }],
        };

        let json = serde_json::to_string(&report).expect("serialize");
        let parsed: DuplicationDensityReport = serde_json::from_str(&json).expect("deserialize");

        prop_assert_eq!(report.duplicate_groups, parsed.duplicate_groups);
        prop_assert_eq!(report.duplicate_files, parsed.duplicate_files);
        prop_assert!((report.wasted_pct_of_codebase - parsed.wasted_pct_of_codebase).abs() < 1e-10);
        prop_assert_eq!(report.by_module.len(), parsed.by_module.len());
    }

    /// HalsteadMetrics round-trips through JSON.
    #[test]
    fn halstead_metrics_roundtrip(
        distinct_ops in 1usize..200,
        distinct_opnds in 1usize..500,
        total_ops in 1usize..10000,
        total_opnds in 1usize..10000,
    ) {
        let metrics = HalsteadMetrics {
            distinct_operators: distinct_ops,
            distinct_operands: distinct_opnds,
            total_operators: total_ops,
            total_operands: total_opnds,
            vocabulary: distinct_ops + distinct_opnds,
            length: total_ops + total_opnds,
            volume: 1000.0,
            difficulty: 5.5,
            effort: 5500.0,
            time_seconds: 305.6,
            estimated_bugs: 0.33,
        };

        let json = serde_json::to_string(&metrics).expect("serialize");
        let parsed: HalsteadMetrics = serde_json::from_str(&json).expect("deserialize");

        prop_assert_eq!(metrics.distinct_operators, parsed.distinct_operators);
        prop_assert_eq!(metrics.distinct_operands, parsed.distinct_operands);
        prop_assert_eq!(metrics.vocabulary, parsed.vocabulary);
        prop_assert_eq!(metrics.length, parsed.length);
        prop_assert!((metrics.volume - parsed.volume).abs() < 1e-10);
    }

    /// MaintainabilityIndex round-trips through JSON.
    #[test]
    fn maintainability_index_roundtrip(
        score in 0.0f64..171.0,
        avg_cyc in 1.0f64..50.0,
        avg_loc in 1.0f64..1000.0,
    ) {
        let mi = MaintainabilityIndex {
            score,
            avg_cyclomatic: avg_cyc,
            avg_loc,
            avg_halstead_volume: Some(500.0),
            grade: if score >= 85.0 { "A".into() } else if score >= 65.0 { "B".into() } else { "C".into() },
        };

        let json = serde_json::to_string(&mi).expect("serialize");
        let parsed: MaintainabilityIndex = serde_json::from_str(&json).expect("deserialize");

        prop_assert!((mi.score - parsed.score).abs() < 1e-10);
        prop_assert_eq!(mi.grade, parsed.grade);
        prop_assert_eq!(mi.avg_halstead_volume, parsed.avg_halstead_volume);
    }

    /// TechnicalDebtRatio round-trips through JSON.
    #[test]
    fn technical_debt_ratio_roundtrip(
        ratio in 0.0f64..100.0,
        points in 0usize..10000,
        kloc in 0.1f64..1000.0,
        level in arb_technical_debt_level(),
    ) {
        let debt = TechnicalDebtRatio {
            ratio,
            complexity_points: points,
            code_kloc: kloc,
            level,
        };

        let json = serde_json::to_string(&debt).expect("serialize");
        let parsed: TechnicalDebtRatio = serde_json::from_str(&json).expect("deserialize");

        prop_assert!((debt.ratio - parsed.ratio).abs() < 1e-10);
        prop_assert_eq!(debt.complexity_points, parsed.complexity_points);
        prop_assert_eq!(debt.level, parsed.level);
    }
}

// ============================================================================
// Vector collection round-trip tests
// ============================================================================

proptest! {
    /// TopicClouds with per_module BTreeMap round-trips through JSON.
    #[test]
    fn topic_clouds_roundtrip(
        overall_count in 0usize..=5,
        module_count in 0usize..=3,
    ) {
        let make_term = |i: usize| TopicTerm {
            term: format!("term{}", i),
            score: 0.5,
            tf: 10,
            df: 3,
        };
        let overall: Vec<TopicTerm> = (0..overall_count).map(make_term).collect();
        let mut per_module = BTreeMap::new();
        for m in 0..module_count {
            per_module.insert(format!("mod{}", m), vec![make_term(m)]);
        }
        let clouds = TopicClouds { per_module, overall };

        let json = serde_json::to_string(&clouds).expect("serialize");
        let parsed: TopicClouds = serde_json::from_str(&json).expect("deserialize");

        prop_assert_eq!(clouds.overall.len(), parsed.overall.len());
        prop_assert_eq!(clouds.per_module.len(), parsed.per_module.len());
    }

    /// EntropyReport suspects vector round-trips through JSON.
    #[test]
    fn entropy_report_roundtrip(
        count in 0usize..=5,
        class in arb_entropy_class(),
    ) {
        let suspects: Vec<EntropyFinding> = (0..count).map(|i| EntropyFinding {
            path: format!("src/file{}.rs", i),
            module: "src".into(),
            entropy_bits_per_byte: 6.5,
            sample_bytes: 1024,
            class,
        }).collect();
        let report = EntropyReport { suspects };

        let json = serde_json::to_string(&report).expect("serialize");
        let parsed: EntropyReport = serde_json::from_str(&json).expect("deserialize");

        prop_assert_eq!(report.suspects.len(), parsed.suspects.len());
    }

    /// PredictiveChurnReport per_module BTreeMap round-trips.
    #[test]
    fn predictive_churn_report_roundtrip(
        count in 0usize..=5,
        trend in arb_trend_class(),
    ) {
        let mut per_module = BTreeMap::new();
        for i in 0..count {
            per_module.insert(format!("mod{}", i), ChurnTrend {
                slope: 0.5,
                r2: 0.9,
                recent_change: 10,
                classification: trend,
            });
        }
        let report = PredictiveChurnReport { per_module };

        let json = serde_json::to_string(&report).expect("serialize");
        let parsed: PredictiveChurnReport = serde_json::from_str(&json).expect("deserialize");

        prop_assert_eq!(report.per_module.len(), parsed.per_module.len());
    }

    /// LicenseReport findings vector round-trips through JSON.
    #[test]
    fn license_report_roundtrip(
        count in 0usize..=5,
        source_kind in arb_license_source_kind(),
    ) {
        let findings: Vec<LicenseFinding> = (0..count).map(|_| LicenseFinding {
            spdx: "MIT".into(),
            confidence: 0.95,
            source_path: "LICENSE".into(),
            source_kind,
        }).collect();
        let report = LicenseReport {
            findings,
            effective: Some("MIT".into()),
        };

        let json = serde_json::to_string(&report).expect("serialize");
        let parsed: LicenseReport = serde_json::from_str(&json).expect("deserialize");

        prop_assert_eq!(report.findings.len(), parsed.findings.len());
        prop_assert_eq!(report.effective, parsed.effective);
    }

    /// GitReport round-trips through JSON.
    #[test]
    fn git_report_roundtrip(
        commits in 0usize..1000,
        files_seen in 0usize..500,
        hotspot_count in 0usize..=3,
    ) {
        let hotspots: Vec<HotspotRow> = (0..hotspot_count).map(|i| HotspotRow {
            path: format!("src/file{}.rs", i),
            commits: 10,
            lines: 100,
            score: 1000,
        }).collect();
        let report = GitReport {
            commits_scanned: commits,
            files_seen,
            hotspots,
            bus_factor: vec![BusFactorRow { module: "src".into(), authors: 3 }],
            freshness: FreshnessReport {
                threshold_days: 90,
                stale_files: 5,
                total_files: 20,
                stale_pct: 25.0,
                by_module: vec![ModuleFreshnessRow {
                    module: "src".into(),
                    avg_days: 30.0,
                    p90_days: 60.0,
                    stale_pct: 10.0,
                }],
            },
            coupling: Vec::new(),
            age_distribution: None,
            intent: None,
        };

        let json = serde_json::to_string(&report).expect("serialize");
        let parsed: GitReport = serde_json::from_str(&json).expect("deserialize");

        prop_assert_eq!(report.commits_scanned, parsed.commits_scanned);
        prop_assert_eq!(report.files_seen, parsed.files_seen);
        prop_assert_eq!(report.hotspots.len(), parsed.hotspots.len());
        prop_assert_eq!(report.bus_factor.len(), parsed.bus_factor.len());
    }

    /// ApiSurfaceReport round-trips through JSON.
    #[test]
    fn api_surface_report_roundtrip(
        total in 1usize..1000,
        public_ratio in 0.0f64..1.0,
    ) {
        let public_items = (total as f64 * public_ratio) as usize;
        let mut by_language = BTreeMap::new();
        by_language.insert("Rust".into(), LangApiSurface {
            total_items: total,
            public_items,
            internal_items: total - public_items,
            public_ratio,
        });
        let report = ApiSurfaceReport {
            total_items: total,
            public_items,
            internal_items: total - public_items,
            public_ratio,
            documented_ratio: 0.5,
            by_language,
            by_module: vec![ModuleApiRow {
                module: "src".into(),
                total_items: total,
                public_items,
                public_ratio,
            }],
            top_exporters: vec![ApiExportItem {
                path: "src/lib.rs".into(),
                lang: "Rust".into(),
                public_items: 10,
                total_items: 20,
            }],
        };

        let json = serde_json::to_string(&report).expect("serialize");
        let parsed: ApiSurfaceReport = serde_json::from_str(&json).expect("deserialize");

        prop_assert_eq!(report.total_items, parsed.total_items);
        prop_assert_eq!(report.public_items, parsed.public_items);
        prop_assert_eq!(report.by_language.len(), parsed.by_language.len());
        prop_assert_eq!(report.by_module.len(), parsed.by_module.len());
        prop_assert_eq!(report.top_exporters.len(), parsed.top_exporters.len());
    }

    /// CommitIntentReport round-trips through JSON.
    #[test]
    fn commit_intent_report_roundtrip(
        feat in 0usize..100,
        fix in 0usize..100,
        unknown_pct in 0.0f64..100.0,
    ) {
        let report = CommitIntentReport {
            overall: CommitIntentCounts {
                feat,
                fix,
                refactor: 5,
                docs: 3,
                test: 7,
                chore: 2,
                ci: 1,
                build: 1,
                perf: 0,
                style: 0,
                revert: 0,
                other: 2,
                total: feat + fix + 21,
            },
            by_module: vec![ModuleIntentRow {
                module: "src".into(),
                counts: CommitIntentCounts::default(),
            }],
            unknown_pct,
            corrective_ratio: Some(0.1),
        };

        let json = serde_json::to_string(&report).expect("serialize");
        let parsed: CommitIntentReport = serde_json::from_str(&json).expect("deserialize");

        prop_assert_eq!(report.overall.feat, parsed.overall.feat);
        prop_assert_eq!(report.overall.fix, parsed.overall.fix);
        prop_assert_eq!(report.overall.total, parsed.overall.total);
        prop_assert_eq!(report.by_module.len(), parsed.by_module.len());
        prop_assert_eq!(report.corrective_ratio, parsed.corrective_ratio);
    }

    /// AssetReport round-trips through JSON.
    #[test]
    fn asset_report_roundtrip(
        total_files in 0usize..100,
        total_bytes in 0u64..1000000,
    ) {
        let report = AssetReport {
            total_files,
            total_bytes,
            categories: vec![AssetCategoryRow {
                category: "image".into(),
                files: 5,
                bytes: 50000,
                extensions: vec!["png".into(), "jpg".into()],
            }],
            top_files: vec![AssetFileRow {
                path: "assets/logo.png".into(),
                bytes: 10000,
                category: "image".into(),
                extension: "png".into(),
            }],
        };

        let json = serde_json::to_string(&report).expect("serialize");
        let parsed: AssetReport = serde_json::from_str(&json).expect("deserialize");

        prop_assert_eq!(report.total_files, parsed.total_files);
        prop_assert_eq!(report.total_bytes, parsed.total_bytes);
        prop_assert_eq!(report.categories.len(), parsed.categories.len());
        prop_assert_eq!(report.top_files.len(), parsed.top_files.len());
    }

    /// DependencyReport round-trips through JSON.
    #[test]
    fn dependency_report_roundtrip(
        total in 0usize..1000,
        lockfile_count in 0usize..=3,
    ) {
        let lockfiles: Vec<LockfileReport> = (0..lockfile_count).map(|i| LockfileReport {
            path: format!("Cargo.lock.{}", i),
            kind: "cargo".into(),
            dependencies: total / lockfile_count.max(1),
        }).collect();
        let report = DependencyReport { total, lockfiles };

        let json = serde_json::to_string(&report).expect("serialize");
        let parsed: DependencyReport = serde_json::from_str(&json).expect("deserialize");

        prop_assert_eq!(report.total, parsed.total);
        prop_assert_eq!(report.lockfiles.len(), parsed.lockfiles.len());
    }

    /// NearDuplicateReport round-trips through JSON.
    #[test]
    fn near_duplicate_report_roundtrip(
        pair_count in 0usize..=5,
        scope in arb_near_dup_scope(),
        truncated in any::<bool>(),
    ) {
        let pairs: Vec<NearDupPairRow> = (0..pair_count).map(|i| NearDupPairRow {
            left: format!("a{}.rs", i),
            right: format!("b{}.rs", i),
            similarity: 0.8,
            shared_fingerprints: 50,
            left_fingerprints: 100,
            right_fingerprints: 100,
        }).collect();
        let report = NearDuplicateReport {
            params: NearDupParams {
                scope,
                threshold: 0.7,
                max_files: 1000,
                max_pairs: Some(500),
                max_file_bytes: Some(100_000),
                selection_method: Some("top-by-size".into()),
                algorithm: Some(NearDupAlgorithm { k_gram_size: 5, window_size: 4, max_postings: 100 }),
                exclude_patterns: vec!["*.lock".into()],
            },
            pairs,
            files_analyzed: 50,
            files_skipped: 5,
            eligible_files: Some(55),
            clusters: Some(vec![NearDupCluster {
                files: vec!["a.rs".into(), "b.rs".into()],
                max_similarity: 0.9,
                representative: "a.rs".into(),
                pair_count: 1,
            }]),
            truncated,
            excluded_by_pattern: Some(3),
            stats: Some(NearDupStats { fingerprinting_ms: 100, pairing_ms: 50, bytes_processed: 50000 }),
        };

        let json = serde_json::to_string(&report).expect("serialize");
        let parsed: NearDuplicateReport = serde_json::from_str(&json).expect("deserialize");

        prop_assert_eq!(report.pairs.len(), parsed.pairs.len());
        prop_assert_eq!(report.files_analyzed, parsed.files_analyzed);
        prop_assert_eq!(report.truncated, parsed.truncated);
        prop_assert!(parsed.clusters.is_some());
        prop_assert!(parsed.stats.is_some());
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

fn arb_near_dup_scope() -> impl Strategy<Value = NearDupScope> {
    prop_oneof![
        Just(NearDupScope::Module),
        Just(NearDupScope::Lang),
        Just(NearDupScope::Global),
    ]
}

fn arb_technical_debt_level() -> impl Strategy<Value = TechnicalDebtLevel> {
    prop_oneof![
        Just(TechnicalDebtLevel::Low),
        Just(TechnicalDebtLevel::Moderate),
        Just(TechnicalDebtLevel::High),
        Just(TechnicalDebtLevel::Critical),
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
