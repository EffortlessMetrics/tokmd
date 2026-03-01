//! Determinism and correctness tests for analysis format rendering.
//!
//! Verifies:
//! 1. Identical inputs produce identical outputs across all text formats.
//! 2. BTreeMap ordering is preserved in JSON output.
//! 3. No platform-specific paths (backslashes) appear in rendered output.
//! 4. Timestamps are normalized or excluded from comparison.
//! 5. Edge case inputs (empty sections, zero values, missing optionals) render without panic.

use std::collections::BTreeMap;

use tokmd_analysis_format::{RenderedOutput, render};
use tokmd_analysis_types::*;
use tokmd_types::{AnalysisFormat, ScanStatus, ToolInfo};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn base_receipt() -> AnalysisReceipt {
    AnalysisReceipt {
        schema_version: ANALYSIS_SCHEMA_VERSION,
        generated_at_ms: 0,
        tool: ToolInfo {
            name: "tokmd".into(),
            version: "0.0.0-test".into(),
        },
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
            module_roots: vec![],
            module_depth: 1,
            children: "collapse".into(),
        },
        args: AnalysisArgsMeta {
            preset: "receipt".into(),
            format: "md".into(),
            window_tokens: None,
            git: None,
            max_files: None,
            max_bytes: None,
            max_file_bytes: None,
            max_commits: None,
            max_commit_files: None,
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
    }
}

fn sample_derived() -> DerivedReport {
    DerivedReport {
        totals: DerivedTotals {
            files: 5,
            code: 500,
            comments: 100,
            blanks: 50,
            lines: 650,
            bytes: 5000,
            tokens: 1200,
        },
        doc_density: RatioReport {
            total: RatioRow {
                key: "total".into(),
                numerator: 100,
                denominator: 600,
                ratio: 0.1667,
            },
            by_lang: vec![],
            by_module: vec![],
        },
        whitespace: RatioReport {
            total: RatioRow {
                key: "total".into(),
                numerator: 50,
                denominator: 650,
                ratio: 0.0769,
            },
            by_lang: vec![],
            by_module: vec![],
        },
        verbosity: RateReport {
            total: RateRow {
                key: "total".into(),
                numerator: 5000,
                denominator: 650,
                rate: 7.69,
            },
            by_lang: vec![],
            by_module: vec![],
        },
        max_file: MaxFileReport {
            overall: FileStatRow {
                path: "src/lib.rs".into(),
                module: "src".into(),
                lang: "Rust".into(),
                code: 200,
                comments: 40,
                blanks: 20,
                lines: 260,
                bytes: 2000,
                tokens: 500,
                doc_pct: Some(0.167),
                bytes_per_line: Some(7.69),
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
            test_lines: 100,
            prod_lines: 400,
            test_files: 2,
            prod_files: 3,
            ratio: 0.25,
        },
        boilerplate: BoilerplateReport {
            infra_lines: 30,
            logic_lines: 470,
            ratio: 0.06,
            infra_langs: vec!["TOML".into()],
        },
        polyglot: PolyglotReport {
            lang_count: 2,
            entropy: 0.5,
            dominant_lang: "Rust".into(),
            dominant_lines: 400,
            dominant_pct: 0.8,
        },
        distribution: DistributionReport {
            count: 5,
            min: 50,
            max: 260,
            mean: 130.0,
            median: 120.0,
            p90: 250.0,
            p99: 260.0,
            gini: 0.25,
        },
        histogram: vec![HistogramBucket {
            label: "all".into(),
            min: 0,
            max: Some(300),
            files: 5,
            pct: 1.0,
        }],
        top: TopOffenders {
            largest_lines: vec![FileStatRow {
                path: "src/lib.rs".into(),
                module: "src".into(),
                lang: "Rust".into(),
                code: 200,
                comments: 40,
                blanks: 20,
                lines: 260,
                bytes: 2000,
                tokens: 500,
                doc_pct: Some(0.167),
                bytes_per_line: Some(7.69),
                depth: 1,
            }],
            largest_tokens: vec![],
            largest_bytes: vec![],
            least_documented: vec![],
            most_dense: vec![],
        },
        tree: None,
        reading_time: ReadingTimeReport {
            minutes: 32.5,
            lines_per_minute: 20,
            basis_lines: 650,
        },
        context_window: None,
        cocomo: None,
        todo: None,
        integrity: IntegrityReport {
            algo: "blake3".into(),
            hash: "a".repeat(64),
            entries: 5,
        },
    }
}

fn render_text(receipt: &AnalysisReceipt, format: AnalysisFormat) -> String {
    match render(receipt, format).expect("render failed") {
        RenderedOutput::Text(s) => s,
        RenderedOutput::Binary(_) => panic!("expected text output"),
    }
}

fn full_receipt() -> AnalysisReceipt {
    let mut receipt = base_receipt();
    receipt.derived = Some(sample_derived());
    receipt.archetype = Some(Archetype {
        kind: "cli-tool".into(),
        evidence: vec!["Cargo.toml".into(), "src/main.rs".into()],
    });
    receipt.topics = Some(TopicClouds {
        overall: vec![TopicTerm {
            term: "parsing".into(),
            score: 0.9,
            tf: 12,
            df: 3,
        }],
        per_module: BTreeMap::new(),
    });
    receipt.imports = Some(ImportReport {
        granularity: "module".into(),
        edges: vec![ImportEdge {
            from: "src/main".into(),
            to: "src/lib".into(),
            count: 5,
        }],
    });
    receipt
}

// ===========================================================================
// 1. Identical inputs → identical rendered output
// ===========================================================================

#[test]
fn identical_inputs_produce_identical_md_output() {
    let r1 = full_receipt();
    let r2 = full_receipt();
    let out1 = render_text(&r1, AnalysisFormat::Md);
    let out2 = render_text(&r2, AnalysisFormat::Md);
    assert_eq!(
        out1, out2,
        "Two identical receipts must render identically as Md"
    );
}

#[test]
fn identical_inputs_produce_identical_json_output() {
    let r1 = full_receipt();
    let r2 = full_receipt();
    let out1 = render_text(&r1, AnalysisFormat::Json);
    let out2 = render_text(&r2, AnalysisFormat::Json);
    assert_eq!(
        out1, out2,
        "Two identical receipts must render identically as JSON"
    );
}

#[test]
fn identical_inputs_produce_identical_xml_output() {
    let r1 = full_receipt();
    let r2 = full_receipt();
    let out1 = render_text(&r1, AnalysisFormat::Xml);
    let out2 = render_text(&r2, AnalysisFormat::Xml);
    assert_eq!(
        out1, out2,
        "Two identical receipts must render identically as XML"
    );
}

#[test]
fn identical_inputs_produce_identical_svg_output() {
    let r1 = full_receipt();
    let r2 = full_receipt();
    let out1 = render_text(&r1, AnalysisFormat::Svg);
    let out2 = render_text(&r2, AnalysisFormat::Svg);
    assert_eq!(
        out1, out2,
        "Two identical receipts must render identically as SVG"
    );
}

#[test]
fn identical_inputs_produce_identical_mermaid_output() {
    let r1 = full_receipt();
    let r2 = full_receipt();
    let out1 = render_text(&r1, AnalysisFormat::Mermaid);
    let out2 = render_text(&r2, AnalysisFormat::Mermaid);
    assert_eq!(
        out1, out2,
        "Two identical receipts must render identically as Mermaid"
    );
}

#[test]
fn identical_inputs_produce_identical_jsonld_output() {
    let r1 = full_receipt();
    let r2 = full_receipt();
    let out1 = render_text(&r1, AnalysisFormat::Jsonld);
    let out2 = render_text(&r2, AnalysisFormat::Jsonld);
    assert_eq!(
        out1, out2,
        "Two identical receipts must render identically as JSON-LD"
    );
}

// ===========================================================================
// 2. BTreeMap ordering preserved in JSON output
// ===========================================================================

#[test]
fn btreemap_ordering_preserved_in_json() {
    let mut receipt = base_receipt();
    let mut per_module = BTreeMap::new();
    // Insert in reverse-alphabetical order; BTreeMap sorts them
    per_module.insert(
        "zzz_module".into(),
        vec![TopicTerm {
            term: "last".into(),
            score: 0.1,
            tf: 1,
            df: 1,
        }],
    );
    per_module.insert(
        "aaa_module".into(),
        vec![TopicTerm {
            term: "first".into(),
            score: 0.9,
            tf: 9,
            df: 3,
        }],
    );
    per_module.insert(
        "mmm_module".into(),
        vec![TopicTerm {
            term: "middle".into(),
            score: 0.5,
            tf: 5,
            df: 2,
        }],
    );
    receipt.topics = Some(TopicClouds {
        overall: vec![],
        per_module,
    });

    let json = render_text(&receipt, AnalysisFormat::Json);
    let aaa_pos = json
        .find("aaa_module")
        .expect("aaa_module should be in JSON");
    let mmm_pos = json
        .find("mmm_module")
        .expect("mmm_module should be in JSON");
    let zzz_pos = json
        .find("zzz_module")
        .expect("zzz_module should be in JSON");
    assert!(
        aaa_pos < mmm_pos && mmm_pos < zzz_pos,
        "BTreeMap keys must appear in sorted order in JSON: aaa={aaa_pos}, mmm={mmm_pos}, zzz={zzz_pos}"
    );
}

#[test]
fn btreemap_churn_ordering_preserved_in_json() {
    let mut receipt = base_receipt();
    let mut per_module = BTreeMap::new();
    per_module.insert(
        "beta".into(),
        ChurnTrend {
            slope: 0.5,
            r2: 0.8,
            recent_change: 10,
            classification: TrendClass::Rising,
        },
    );
    per_module.insert(
        "alpha".into(),
        ChurnTrend {
            slope: 0.3,
            r2: 0.6,
            recent_change: 5,
            classification: TrendClass::Flat,
        },
    );
    receipt.predictive_churn = Some(PredictiveChurnReport { per_module });

    let json = render_text(&receipt, AnalysisFormat::Json);
    let alpha_pos = json.find("alpha").expect("alpha should be in JSON");
    let beta_pos = json.find("beta").expect("beta should be in JSON");
    assert!(
        alpha_pos < beta_pos,
        "BTreeMap keys must appear in sorted order: alpha={alpha_pos}, beta={beta_pos}"
    );
}

// ===========================================================================
// 3. No platform-specific paths in rendered output
// ===========================================================================

#[test]
fn no_backslash_paths_in_md_output() {
    let receipt = full_receipt();
    let md = render_text(&receipt, AnalysisFormat::Md);
    // Backslash should not appear as a path separator in rendered output.
    // We check that no lines contain typical Windows path patterns.
    for line in md.lines() {
        assert!(
            !line.contains("src\\"),
            "Rendered MD should not contain backslash paths: {line}"
        );
        assert!(
            !line.contains("tests\\"),
            "Rendered MD should not contain backslash paths: {line}"
        );
    }
}

#[test]
fn no_backslash_paths_in_json_output() {
    let receipt = full_receipt();
    let json = render_text(&receipt, AnalysisFormat::Json);
    // JSON paths should always use forward slashes
    assert!(
        !json.contains("src\\\\"),
        "JSON output should not contain escaped backslash paths"
    );
    assert!(
        !json.contains("src\\main"),
        "JSON output should not contain backslash paths"
    );
}

#[test]
fn forward_slash_paths_in_file_stat_rows() {
    let mut receipt = base_receipt();
    let mut derived = sample_derived();
    derived.top.largest_lines = vec![FileStatRow {
        path: "src/deep/nested/file.rs".into(),
        module: "src/deep".into(),
        lang: "Rust".into(),
        code: 300,
        comments: 60,
        blanks: 30,
        lines: 390,
        bytes: 3000,
        tokens: 700,
        doc_pct: Some(0.17),
        bytes_per_line: Some(7.69),
        depth: 3,
    }];
    receipt.derived = Some(derived);

    let md = render_text(&receipt, AnalysisFormat::Md);
    assert!(
        md.contains("src/deep/nested/file.rs"),
        "Paths in rendered output should use forward slashes"
    );
}

// ===========================================================================
// 4. Timestamps normalized or excluded from comparison
// ===========================================================================

#[test]
fn different_timestamps_produce_different_json() {
    let mut r1 = full_receipt();
    let mut r2 = full_receipt();
    r1.generated_at_ms = 1_000_000;
    r2.generated_at_ms = 2_000_000;

    let json1 = render_text(&r1, AnalysisFormat::Json);
    let json2 = render_text(&r2, AnalysisFormat::Json);
    // Timestamps are in JSON, so different timestamps = different JSON.
    assert_ne!(
        json1, json2,
        "Different timestamps should produce different JSON"
    );
}

#[test]
fn timestamp_not_in_md_output() {
    let mut receipt = full_receipt();
    receipt.generated_at_ms = 1_700_000_000_000;
    let md = render_text(&receipt, AnalysisFormat::Md);
    // The MD renderer does not include generated_at_ms
    assert!(
        !md.contains("1700000000000"),
        "MD output should not contain raw timestamp"
    );
}

#[test]
fn different_timestamps_same_md_output() {
    let mut r1 = full_receipt();
    let mut r2 = full_receipt();
    r1.generated_at_ms = 1_000_000;
    r2.generated_at_ms = 9_999_999;
    let md1 = render_text(&r1, AnalysisFormat::Md);
    let md2 = render_text(&r2, AnalysisFormat::Md);
    assert_eq!(
        md1, md2,
        "MD output should be identical regardless of timestamp"
    );
}

// ===========================================================================
// 5. Edge case inputs render without panicking
// ===========================================================================

#[test]
fn empty_receipt_renders_all_formats() {
    let receipt = base_receipt();
    let formats = [
        AnalysisFormat::Md,
        AnalysisFormat::Json,
        AnalysisFormat::Jsonld,
        AnalysisFormat::Xml,
        AnalysisFormat::Svg,
        AnalysisFormat::Mermaid,
        AnalysisFormat::Tree,
        AnalysisFormat::Html,
    ];
    for fmt in formats {
        let result = render(&receipt, fmt);
        assert!(result.is_ok(), "Empty receipt should render as {fmt:?}");
    }
}

#[test]
fn zero_value_derived_renders_without_panic() {
    let mut receipt = base_receipt();
    receipt.derived = Some(DerivedReport {
        totals: DerivedTotals {
            files: 0,
            code: 0,
            comments: 0,
            blanks: 0,
            lines: 0,
            bytes: 0,
            tokens: 0,
        },
        doc_density: RatioReport {
            total: RatioRow {
                key: "total".into(),
                numerator: 0,
                denominator: 0,
                ratio: 0.0,
            },
            by_lang: vec![],
            by_module: vec![],
        },
        whitespace: RatioReport {
            total: RatioRow {
                key: "total".into(),
                numerator: 0,
                denominator: 0,
                ratio: 0.0,
            },
            by_lang: vec![],
            by_module: vec![],
        },
        verbosity: RateReport {
            total: RateRow {
                key: "total".into(),
                numerator: 0,
                denominator: 0,
                rate: 0.0,
            },
            by_lang: vec![],
            by_module: vec![],
        },
        max_file: MaxFileReport {
            overall: FileStatRow {
                path: String::new(),
                module: String::new(),
                lang: String::new(),
                code: 0,
                comments: 0,
                blanks: 0,
                lines: 0,
                bytes: 0,
                tokens: 0,
                doc_pct: None,
                bytes_per_line: None,
                depth: 0,
            },
            by_lang: vec![],
            by_module: vec![],
        },
        lang_purity: LangPurityReport { rows: vec![] },
        nesting: NestingReport {
            max: 0,
            avg: 0.0,
            by_module: vec![],
        },
        test_density: TestDensityReport {
            test_lines: 0,
            prod_lines: 0,
            test_files: 0,
            prod_files: 0,
            ratio: 0.0,
        },
        boilerplate: BoilerplateReport {
            infra_lines: 0,
            logic_lines: 0,
            ratio: 0.0,
            infra_langs: vec![],
        },
        polyglot: PolyglotReport {
            lang_count: 0,
            entropy: 0.0,
            dominant_lang: String::new(),
            dominant_lines: 0,
            dominant_pct: 0.0,
        },
        distribution: DistributionReport {
            count: 0,
            min: 0,
            max: 0,
            mean: 0.0,
            median: 0.0,
            p90: 0.0,
            p99: 0.0,
            gini: 0.0,
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
            minutes: 0.0,
            lines_per_minute: 0,
            basis_lines: 0,
        },
        context_window: None,
        cocomo: None,
        todo: None,
        integrity: IntegrityReport {
            algo: "blake3".into(),
            hash: "0".repeat(64),
            entries: 0,
        },
    });

    let formats = [
        AnalysisFormat::Md,
        AnalysisFormat::Json,
        AnalysisFormat::Xml,
        AnalysisFormat::Svg,
        AnalysisFormat::Mermaid,
    ];
    for fmt in formats {
        let result = render(&receipt, fmt);
        assert!(
            result.is_ok(),
            "Zero-value derived should render as {fmt:?}"
        );
    }
}

#[test]
fn empty_optional_sections_render_without_panic() {
    let mut receipt = base_receipt();
    // Set all optional sections to Some with empty data
    receipt.entropy = Some(EntropyReport { suspects: vec![] });
    receipt.license = Some(LicenseReport {
        findings: vec![],
        effective: None,
    });
    receipt.corporate_fingerprint = Some(CorporateFingerprint { domains: vec![] });
    receipt.predictive_churn = Some(PredictiveChurnReport {
        per_module: BTreeMap::new(),
    });
    receipt.assets = Some(AssetReport {
        total_files: 0,
        total_bytes: 0,
        categories: vec![],
        top_files: vec![],
    });
    receipt.deps = Some(DependencyReport {
        total: 0,
        lockfiles: vec![],
    });
    receipt.git = Some(GitReport {
        commits_scanned: 0,
        files_seen: 0,
        hotspots: vec![],
        bus_factor: vec![],
        freshness: FreshnessReport {
            threshold_days: 90,
            stale_files: 0,
            total_files: 0,
            stale_pct: 0.0,
            by_module: vec![],
        },
        coupling: vec![],
        age_distribution: None,
        intent: None,
    });
    receipt.imports = Some(ImportReport {
        granularity: "module".into(),
        edges: vec![],
    });
    receipt.dup = Some(DuplicateReport {
        groups: vec![],
        wasted_bytes: 0,
        strategy: "blake3".into(),
        density: None,
        near: None,
    });
    receipt.complexity = Some(ComplexityReport {
        total_functions: 0,
        avg_function_length: 0.0,
        max_function_length: 0,
        avg_cyclomatic: 0.0,
        max_cyclomatic: 0,
        avg_cognitive: None,
        max_cognitive: None,
        avg_nesting_depth: None,
        max_nesting_depth: None,
        high_risk_files: 0,
        histogram: None,
        halstead: None,
        maintainability_index: None,
        technical_debt: None,
        files: vec![],
    });
    receipt.api_surface = Some(ApiSurfaceReport {
        total_items: 0,
        public_items: 0,
        internal_items: 0,
        public_ratio: 0.0,
        documented_ratio: 0.0,
        by_language: BTreeMap::new(),
        by_module: vec![],
        top_exporters: vec![],
    });

    let formats = [
        AnalysisFormat::Md,
        AnalysisFormat::Json,
        AnalysisFormat::Xml,
        AnalysisFormat::Svg,
    ];
    for fmt in formats {
        let result = render(&receipt, fmt);
        assert!(
            result.is_ok(),
            "Empty optional sections should render as {fmt:?}"
        );
    }
}

#[test]
fn missing_optional_sections_render_without_panic() {
    // All optional sections are None
    let receipt = base_receipt();
    let md = render_text(&receipt, AnalysisFormat::Md);
    assert!(
        md.starts_with("# tokmd analysis\n"),
        "MD should start with header even with no sections"
    );
    // Should not contain section headers for missing sections
    assert!(!md.contains("## Totals"), "No Totals when derived is None");
    assert!(!md.contains("## Archetype"), "No Archetype when None");
    assert!(!md.contains("## Topics"), "No Topics when None");
    assert!(!md.contains("## Git metrics"), "No Git when None");
}

#[test]
fn receipt_with_all_sections_renders_json_roundtrip() {
    let receipt = full_receipt();
    let json = render_text(&receipt, AnalysisFormat::Json);
    let parsed: AnalysisReceipt =
        serde_json::from_str(&json).expect("JSON should roundtrip to AnalysisReceipt");
    assert_eq!(parsed.schema_version, receipt.schema_version);
    assert_eq!(parsed.mode, receipt.mode);
    assert_eq!(parsed.args.preset, receipt.args.preset);
}

#[test]
fn receipt_with_warnings_renders_all_formats() {
    let mut receipt = base_receipt();
    receipt.warnings = vec![
        "File too large: src/generated.rs".into(),
        "Low coverage".into(),
        String::new(), // edge case: empty warning
    ];
    let formats = [
        AnalysisFormat::Md,
        AnalysisFormat::Json,
        AnalysisFormat::Xml,
    ];
    for fmt in formats {
        let result = render(&receipt, fmt);
        assert!(result.is_ok(), "Warnings should render as {fmt:?}");
    }
}
