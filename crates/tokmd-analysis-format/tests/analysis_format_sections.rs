//! Section-level rendering tests for analysis format.
//!
//! Verifies:
//! 1. Each analysis section renders correctly in isolation.
//! 2. Section headers are present and properly formatted.
//! 3. Numeric formatting is consistent (commas, decimals, percentages).

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

fn render_md(receipt: &AnalysisReceipt) -> String {
    match render(receipt, AnalysisFormat::Md).expect("render failed") {
        RenderedOutput::Text(s) => s,
        RenderedOutput::Binary(_) => panic!("expected text output"),
    }
}

// ===========================================================================
// Archetype section
// ===========================================================================

#[test]
fn archetype_section_renders_kind_and_evidence() {
    let mut receipt = base_receipt();
    receipt.archetype = Some(Archetype {
        kind: "web-application".into(),
        evidence: vec!["package.json".into(), "src/App.tsx".into()],
    });
    let md = render_md(&receipt);
    assert!(md.contains("## Archetype"), "Should have Archetype header");
    assert!(
        md.contains("Kind: `web-application`"),
        "Should render archetype kind"
    );
    assert!(md.contains("package.json"), "Should render evidence items");
    assert!(md.contains("src/App.tsx"), "Should render evidence items");
}

#[test]
fn archetype_section_empty_evidence() {
    let mut receipt = base_receipt();
    receipt.archetype = Some(Archetype {
        kind: "library".into(),
        evidence: vec![],
    });
    let md = render_md(&receipt);
    assert!(md.contains("## Archetype"));
    assert!(md.contains("Kind: `library`"));
    // No evidence line should be present
    assert!(!md.contains("Evidence:"));
}

// ===========================================================================
// Topics section
// ===========================================================================

#[test]
fn topics_section_renders_overall_terms() {
    let mut receipt = base_receipt();
    receipt.topics = Some(TopicClouds {
        overall: vec![
            TopicTerm {
                term: "parsing".into(),
                score: 0.9,
                tf: 12,
                df: 3,
            },
            TopicTerm {
                term: "ast".into(),
                score: 0.7,
                tf: 8,
                df: 2,
            },
        ],
        per_module: BTreeMap::new(),
    });
    let md = render_md(&receipt);
    assert!(md.contains("## Topics"), "Should have Topics header");
    assert!(md.contains("parsing"), "Should render overall topic terms");
    assert!(md.contains("ast"), "Should render overall topic terms");
}

#[test]
fn topics_section_renders_per_module() {
    let mut per_module = BTreeMap::new();
    per_module.insert(
        "src".into(),
        vec![TopicTerm {
            term: "core".into(),
            score: 0.8,
            tf: 10,
            df: 2,
        }],
    );
    let mut receipt = base_receipt();
    receipt.topics = Some(TopicClouds {
        overall: vec![],
        per_module,
    });
    let md = render_md(&receipt);
    assert!(md.contains("## Topics"));
    assert!(md.contains("`src`"), "Should render module name");
    assert!(md.contains("core"), "Should render module topic terms");
}

// ===========================================================================
// Entropy section
// ===========================================================================

#[test]
fn entropy_section_renders_suspects() {
    let mut receipt = base_receipt();
    receipt.entropy = Some(EntropyReport {
        suspects: vec![EntropyFinding {
            path: "secrets/key.env".into(),
            module: "secrets".into(),
            entropy_bits_per_byte: 7.5,
            sample_bytes: 128,
            class: EntropyClass::High,
        }],
    });
    let md = render_md(&receipt);
    assert!(
        md.contains("## Entropy profiling"),
        "Should have Entropy header"
    );
    assert!(md.contains("secrets/key.env"), "Should render suspect path");
    assert!(md.contains("7.50"), "Should render entropy with 2 decimals");
    assert!(md.contains("128"), "Should render sample bytes");
}

#[test]
fn entropy_section_empty_suspects() {
    let mut receipt = base_receipt();
    receipt.entropy = Some(EntropyReport { suspects: vec![] });
    let md = render_md(&receipt);
    assert!(md.contains("## Entropy profiling"));
    assert!(
        md.contains("No entropy outliers detected"),
        "Should show no-outliers message"
    );
}

// ===========================================================================
// License section
// ===========================================================================

#[test]
fn license_section_renders_findings() {
    let mut receipt = base_receipt();
    receipt.license = Some(LicenseReport {
        findings: vec![LicenseFinding {
            spdx: "MIT".into(),
            confidence: 0.95,
            source_path: "LICENSE".into(),
            source_kind: LicenseSourceKind::Text,
        }],
        effective: Some("MIT".into()),
    });
    let md = render_md(&receipt);
    assert!(
        md.contains("## License radar"),
        "Should have License header"
    );
    assert!(
        md.contains("Effective: `MIT`"),
        "Should render effective license"
    );
    assert!(md.contains("0.95"), "Should render confidence value");
}

// ===========================================================================
// Corporate fingerprint section
// ===========================================================================

#[test]
fn corporate_fingerprint_section_renders_domains() {
    let mut receipt = base_receipt();
    receipt.corporate_fingerprint = Some(CorporateFingerprint {
        domains: vec![DomainStat {
            domain: "example.com".into(),
            commits: 100,
            pct: 0.75,
        }],
    });
    let md = render_md(&receipt);
    assert!(
        md.contains("## Corporate fingerprint"),
        "Should have fingerprint header"
    );
    assert!(md.contains("example.com"), "Should render domain name");
    assert!(md.contains("100"), "Should render commit count");
    assert!(md.contains("75.0%"), "Should render percentage");
}

#[test]
fn corporate_fingerprint_empty_domains() {
    let mut receipt = base_receipt();
    receipt.corporate_fingerprint = Some(CorporateFingerprint { domains: vec![] });
    let md = render_md(&receipt);
    assert!(md.contains("## Corporate fingerprint"));
    assert!(md.contains("No commit domains detected"));
}

// ===========================================================================
// Predictive churn section
// ===========================================================================

#[test]
fn churn_section_renders_trends() {
    let mut receipt = base_receipt();
    let mut per_module = BTreeMap::new();
    per_module.insert(
        "src".into(),
        ChurnTrend {
            slope: 0.1234,
            r2: 0.89,
            recent_change: 42,
            classification: TrendClass::Rising,
        },
    );
    receipt.predictive_churn = Some(PredictiveChurnReport { per_module });
    let md = render_md(&receipt);
    assert!(
        md.contains("## Predictive churn"),
        "Should have churn header"
    );
    assert!(md.contains("0.1234"), "Should render slope with 4 decimals");
    assert!(md.contains("0.89"), "Should render R² with 2 decimals");
    assert!(md.contains("42"), "Should render recent change count");
    assert!(md.contains("Rising"), "Should render classification");
}

#[test]
fn churn_section_empty_modules() {
    let mut receipt = base_receipt();
    receipt.predictive_churn = Some(PredictiveChurnReport {
        per_module: BTreeMap::new(),
    });
    let md = render_md(&receipt);
    assert!(md.contains("## Predictive churn"));
    assert!(md.contains("No churn signals detected"));
}

// ===========================================================================
// Derived section: Totals, Ratios, Distribution
// ===========================================================================

#[test]
fn derived_totals_section_header_and_values() {
    let mut receipt = base_receipt();
    receipt.derived = Some(DerivedReport {
        totals: DerivedTotals {
            files: 42,
            code: 10000,
            comments: 2500,
            blanks: 800,
            lines: 13300,
            bytes: 100000,
            tokens: 25000,
        },
        doc_density: RatioReport {
            total: RatioRow {
                key: "total".into(),
                numerator: 2500,
                denominator: 12500,
                ratio: 0.2,
            },
            by_lang: vec![],
            by_module: vec![],
        },
        whitespace: RatioReport {
            total: RatioRow {
                key: "total".into(),
                numerator: 800,
                denominator: 13300,
                ratio: 0.0602,
            },
            by_lang: vec![],
            by_module: vec![],
        },
        verbosity: RateReport {
            total: RateRow {
                key: "total".into(),
                numerator: 100000,
                denominator: 13300,
                rate: 7.52,
            },
            by_lang: vec![],
            by_module: vec![],
        },
        max_file: MaxFileReport {
            overall: FileStatRow {
                path: "src/lib.rs".into(),
                module: "src".into(),
                lang: "Rust".into(),
                code: 500,
                comments: 100,
                blanks: 50,
                lines: 650,
                bytes: 5000,
                tokens: 1200,
                doc_pct: Some(0.167),
                bytes_per_line: Some(7.69),
                depth: 1,
            },
            by_lang: vec![],
            by_module: vec![],
        },
        lang_purity: LangPurityReport { rows: vec![] },
        nesting: NestingReport {
            max: 4,
            avg: 2.0,
            by_module: vec![],
        },
        test_density: TestDensityReport {
            test_lines: 3000,
            prod_lines: 7000,
            test_files: 10,
            prod_files: 32,
            ratio: 0.4286,
        },
        boilerplate: BoilerplateReport {
            infra_lines: 500,
            logic_lines: 9500,
            ratio: 0.05,
            infra_langs: vec!["TOML".into(), "YAML".into()],
        },
        polyglot: PolyglotReport {
            lang_count: 3,
            entropy: 0.85,
            dominant_lang: "Rust".into(),
            dominant_lines: 7000,
            dominant_pct: 0.7,
        },
        distribution: DistributionReport {
            count: 42,
            min: 10,
            max: 650,
            mean: 316.67,
            median: 280.0,
            p90: 550.0,
            p99: 640.0,
            gini: 0.3214,
        },
        histogram: vec![
            HistogramBucket {
                label: "Small".into(),
                min: 0,
                max: Some(100),
                files: 15,
                pct: 0.357,
            },
            HistogramBucket {
                label: "Large".into(),
                min: 101,
                max: None,
                files: 27,
                pct: 0.643,
            },
        ],
        top: TopOffenders {
            largest_lines: vec![],
            largest_tokens: vec![],
            largest_bytes: vec![],
            least_documented: vec![],
            most_dense: vec![],
        },
        tree: None,
        reading_time: ReadingTimeReport {
            minutes: 665.0,
            lines_per_minute: 20,
            basis_lines: 13300,
        },
        context_window: None,
        cocomo: None,
        todo: None,
        integrity: IntegrityReport {
            algo: "blake3".into(),
            hash: "b".repeat(64),
            entries: 42,
        },
    });

    let md = render_md(&receipt);

    // Section headers
    assert!(md.contains("## Totals"), "Should have Totals header");
    assert!(md.contains("## Ratios"), "Should have Ratios header");
    assert!(
        md.contains("## Distribution"),
        "Should have Distribution header"
    );
    assert!(
        md.contains("## File size histogram"),
        "Should have histogram header"
    );
    assert!(
        md.contains("## Top offenders"),
        "Should have Top offenders header"
    );
    assert!(md.contains("## Structure"), "Should have Structure header");
    assert!(
        md.contains("## Test density"),
        "Should have Test density header"
    );
    assert!(
        md.contains("## Boilerplate ratio"),
        "Should have Boilerplate header"
    );
    assert!(md.contains("## Polyglot"), "Should have Polyglot header");
    assert!(
        md.contains("## Reading time"),
        "Should have Reading time header"
    );
    assert!(md.contains("## Integrity"), "Should have Integrity header");

    // Totals values
    assert!(md.contains("|42|"), "Should render file count");
    assert!(md.contains("|10000|"), "Should render code lines");
    assert!(md.contains("|25000|"), "Should render tokens");

    // Ratios — percentage formatting
    assert!(
        md.contains("20.0%"),
        "Doc density should be formatted as percentage"
    );
    assert!(
        md.contains("6.0%"),
        "Whitespace ratio should be formatted as percentage"
    );
    assert!(md.contains("7.52"), "Bytes per line with 2 decimals");

    // Distribution
    assert!(md.contains("316.67"), "Mean with 2 decimals");
    assert!(md.contains("280.00"), "Median with 2 decimals");
    assert!(md.contains("0.3214"), "Gini with 4 decimals");

    // Histogram — unbounded max renders as ∞
    assert!(md.contains("∞"), "Unbounded max should render as ∞");
}

// ===========================================================================
// COCOMO section
// ===========================================================================

#[test]
fn cocomo_section_renders_with_correct_formatting() {
    let mut receipt = base_receipt();
    let mut derived = minimal_derived();
    derived.cocomo = Some(CocomoReport {
        mode: "organic".into(),
        kloc: 1.5000,
        effort_pm: 4.12,
        duration_months: 3.50,
        staff: 1.18,
        a: 2.4,
        b: 1.05,
        c: 2.5,
        d: 0.38,
    });
    receipt.derived = Some(derived);
    let md = render_md(&receipt);
    assert!(
        md.contains("## COCOMO estimate"),
        "Should have COCOMO header"
    );
    assert!(md.contains("Mode: `organic`"), "Should render mode");
    assert!(md.contains("1.5000"), "KLOC with 4 decimals");
    assert!(md.contains("4.12"), "Effort with 2 decimals");
    assert!(md.contains("3.50"), "Duration with 2 decimals");
    assert!(md.contains("1.18"), "Staff with 2 decimals");
}

// ===========================================================================
// TODO section
// ===========================================================================

#[test]
fn todo_section_renders_tags_and_density() {
    let mut receipt = base_receipt();
    let mut derived = minimal_derived();
    derived.todo = Some(TodoReport {
        total: 25,
        density_per_kloc: 8.33,
        tags: vec![
            TodoTagRow {
                tag: "TODO".into(),
                count: 15,
            },
            TodoTagRow {
                tag: "FIXME".into(),
                count: 10,
            },
        ],
    });
    receipt.derived = Some(derived);
    let md = render_md(&receipt);
    assert!(md.contains("## TODOs"), "Should have TODO header");
    assert!(md.contains("Total: `25`"), "Should render total");
    assert!(md.contains("8.33"), "Density with 2 decimals");
    assert!(md.contains("|TODO|15|"), "Should render TODO tag row");
    assert!(md.contains("|FIXME|10|"), "Should render FIXME tag row");
}

// ===========================================================================
// Context window section
// ===========================================================================

#[test]
fn context_window_section_renders() {
    let mut receipt = base_receipt();
    let mut derived = minimal_derived();
    derived.context_window = Some(ContextWindowReport {
        window_tokens: 128000,
        total_tokens: 25000,
        pct: 0.1953,
        fits: true,
    });
    receipt.derived = Some(derived);
    let md = render_md(&receipt);
    assert!(
        md.contains("## Context window"),
        "Should have Context window header"
    );
    assert!(md.contains("128000"), "Should render window tokens");
    assert!(md.contains("25000"), "Should render total tokens");
    assert!(md.contains("19.5%"), "Should render pct as percentage");
    assert!(md.contains("true"), "Should render fits flag");
}

// ===========================================================================
// Assets section
// ===========================================================================

#[test]
fn assets_section_renders_categories() {
    let mut receipt = base_receipt();
    receipt.assets = Some(AssetReport {
        total_files: 15,
        total_bytes: 50000,
        categories: vec![AssetCategoryRow {
            category: "images".into(),
            files: 10,
            bytes: 40000,
            extensions: vec!["png".into(), "jpg".into()],
        }],
        top_files: vec![AssetFileRow {
            path: "assets/logo.png".into(),
            bytes: 15000,
            category: "images".into(),
            extension: "png".into(),
        }],
    });
    let md = render_md(&receipt);
    assert!(md.contains("## Assets"), "Should have Assets header");
    assert!(
        md.contains("Total files: `15`"),
        "Should render total files"
    );
    assert!(md.contains("images"), "Should render category");
    assert!(md.contains("png, jpg"), "Should render extensions");
    assert!(
        md.contains("assets/logo.png"),
        "Should render top file path"
    );
}

// ===========================================================================
// Dependencies section
// ===========================================================================

#[test]
fn deps_section_renders_lockfiles() {
    let mut receipt = base_receipt();
    receipt.deps = Some(DependencyReport {
        total: 150,
        lockfiles: vec![LockfileReport {
            path: "Cargo.lock".into(),
            kind: "cargo".into(),
            dependencies: 150,
        }],
    });
    let md = render_md(&receipt);
    assert!(
        md.contains("## Dependencies"),
        "Should have Dependencies header"
    );
    assert!(md.contains("Total: `150`"), "Should render total deps");
    assert!(md.contains("Cargo.lock"), "Should render lockfile path");
    assert!(md.contains("cargo"), "Should render lockfile kind");
}

// ===========================================================================
// Git section
// ===========================================================================

#[test]
fn git_section_renders_hotspots_and_bus_factor() {
    let mut receipt = base_receipt();
    receipt.git = Some(GitReport {
        commits_scanned: 500,
        files_seen: 80,
        hotspots: vec![HotspotRow {
            path: "src/parser.rs".into(),
            commits: 45,
            lines: 800,
            score: 36000,
        }],
        bus_factor: vec![BusFactorRow {
            module: "src".into(),
            authors: 2,
        }],
        freshness: FreshnessReport {
            threshold_days: 90,
            stale_files: 5,
            total_files: 80,
            stale_pct: 0.0625,
            by_module: vec![ModuleFreshnessRow {
                module: "src".into(),
                avg_days: 30.5,
                p90_days: 75.0,
                stale_pct: 0.05,
            }],
        },
        coupling: vec![],
        age_distribution: None,
        intent: None,
    });
    let md = render_md(&receipt);
    assert!(
        md.contains("## Git metrics"),
        "Should have Git metrics header"
    );
    assert!(
        md.contains("### Hotspots"),
        "Should have Hotspots subheader"
    );
    assert!(md.contains("src/parser.rs"), "Should render hotspot path");
    assert!(md.contains("36000"), "Should render hotspot score");
    assert!(
        md.contains("### Bus factor"),
        "Should have Bus factor subheader"
    );
    assert!(
        md.contains("### Freshness"),
        "Should have Freshness subheader"
    );
    assert!(md.contains("6.2%"), "Should render stale_pct as percentage");
    assert!(
        md.contains("30.50"),
        "Should render avg_days with 2 decimals"
    );
    assert!(
        md.contains("75.00"),
        "Should render p90_days with 2 decimals"
    );
}

// ===========================================================================
// Imports section
// ===========================================================================

#[test]
fn imports_section_renders_edges() {
    let mut receipt = base_receipt();
    receipt.imports = Some(ImportReport {
        granularity: "file".into(),
        edges: vec![ImportEdge {
            from: "src/main.rs".into(),
            to: "src/lib.rs".into(),
            count: 3,
        }],
    });
    let md = render_md(&receipt);
    assert!(md.contains("## Imports"), "Should have Imports header");
    assert!(
        md.contains("Granularity: `file`"),
        "Should render granularity"
    );
    assert!(md.contains("src/main.rs"), "Should render edge source");
    assert!(md.contains("|3|"), "Should render edge count");
}

// ===========================================================================
// Duplicates section
// ===========================================================================

#[test]
fn duplicates_section_renders_groups() {
    let mut receipt = base_receipt();
    receipt.dup = Some(DuplicateReport {
        groups: vec![DuplicateGroup {
            hash: "abc123".into(),
            bytes: 1024,
            files: vec!["src/a.rs".into(), "src/b.rs".into()],
        }],
        wasted_bytes: 1024,
        strategy: "blake3".into(),
        density: None,
        near: None,
    });
    let md = render_md(&receipt);
    assert!(
        md.contains("## Duplicates"),
        "Should have Duplicates header"
    );
    assert!(
        md.contains("Wasted bytes: `1024`"),
        "Should render wasted bytes"
    );
    assert!(md.contains("abc123"), "Should render group hash");
    assert!(md.contains("|2|"), "Should render file count");
}

// ===========================================================================
// Complexity section
// ===========================================================================

#[test]
fn complexity_section_renders_metrics() {
    let mut receipt = base_receipt();
    receipt.complexity = Some(ComplexityReport {
        total_functions: 100,
        avg_function_length: 15.5,
        max_function_length: 200,
        avg_cyclomatic: 3.75,
        max_cyclomatic: 30,
        avg_cognitive: Some(5.25),
        max_cognitive: Some(28),
        avg_nesting_depth: Some(2.33),
        max_nesting_depth: Some(6),
        high_risk_files: 5,
        histogram: None,
        halstead: None,
        maintainability_index: None,
        technical_debt: None,
        files: vec![FileComplexity {
            path: "src/parser.rs".into(),
            module: "src".into(),
            function_count: 20,
            max_function_length: 200,
            cyclomatic_complexity: 30,
            cognitive_complexity: Some(28),
            max_nesting: Some(6),
            risk_level: ComplexityRisk::Critical,
            functions: None,
        }],
    });
    let md = render_md(&receipt);
    assert!(
        md.contains("## Complexity"),
        "Should have Complexity header"
    );
    assert!(md.contains("|Total functions|100|"), "Total functions");
    assert!(md.contains("15.5"), "Avg function length with 1 decimal");
    assert!(md.contains("3.75"), "Avg cyclomatic with 2 decimals");
    assert!(md.contains("5.25"), "Avg cognitive with 2 decimals");
    assert!(md.contains("2.33"), "Avg nesting depth with 2 decimals");
    assert!(md.contains("|High risk files|5|"), "High risk files");
    assert!(
        md.contains("### Top complex files"),
        "Should have subheader for files"
    );
    assert!(
        md.contains("src/parser.rs"),
        "Should render complex file path"
    );
}

// ===========================================================================
// API surface section
// ===========================================================================

#[test]
fn api_surface_section_renders_metrics() {
    let mut receipt = base_receipt();
    let mut by_language = BTreeMap::new();
    by_language.insert(
        "Rust".into(),
        LangApiSurface {
            total_items: 80,
            public_items: 30,
            internal_items: 50,
            public_ratio: 0.375,
        },
    );
    receipt.api_surface = Some(ApiSurfaceReport {
        total_items: 80,
        public_items: 30,
        internal_items: 50,
        public_ratio: 0.375,
        documented_ratio: 0.60,
        by_language,
        by_module: vec![ModuleApiRow {
            module: "src".into(),
            total_items: 80,
            public_items: 30,
            public_ratio: 0.375,
        }],
        top_exporters: vec![ApiExportItem {
            path: "src/lib.rs".into(),
            lang: "Rust".into(),
            public_items: 15,
            total_items: 40,
        }],
    });
    let md = render_md(&receipt);
    assert!(
        md.contains("## API surface"),
        "Should have API surface header"
    );
    assert!(md.contains("|Total items|80|"), "Total items");
    assert!(md.contains("|Public items|30|"), "Public items");
    assert!(md.contains("37.5%"), "Public ratio as percentage");
    assert!(md.contains("60.0%"), "Documented ratio as percentage");
    assert!(
        md.contains("### By language"),
        "Should have language subheader"
    );
    assert!(md.contains("### By module"), "Should have module subheader");
    assert!(
        md.contains("### Top exporters"),
        "Should have exporters subheader"
    );
}

// ===========================================================================
// Numeric formatting consistency
// ===========================================================================

#[test]
fn percentage_formatting_consistency() {
    let mut receipt = base_receipt();
    receipt.derived = Some(minimal_derived_with_ratios(0.3333, 0.0, 1.0));
    let md = render_md(&receipt);
    // 0.3333 → "33.3%"
    assert!(md.contains("33.3%"), "Should format 0.3333 as 33.3%");
    // 0.0 → "0.0%"
    assert!(md.contains("0.0%"), "Should format 0.0 as 0.0%");
    // 1.0 → "100.0%"
    assert!(md.contains("100.0%"), "Should format 1.0 as 100.0%");
}

#[test]
fn float_formatting_consistency() {
    let mut receipt = base_receipt();
    let mut derived = minimal_derived();
    // Test rate with specific decimal format
    derived.verbosity.total.rate = 12.3456;
    receipt.derived = Some(derived);
    let md = render_md(&receipt);
    // Rate is rendered with 2 decimal places
    assert!(
        md.contains("12.35"),
        "Should render rate with 2 decimals (rounded)"
    );
}

// ===========================================================================
// Mermaid format section test
// ===========================================================================

#[test]
fn mermaid_renders_import_edges() {
    let mut receipt = base_receipt();
    receipt.imports = Some(ImportReport {
        granularity: "module".into(),
        edges: vec![
            ImportEdge {
                from: "core".into(),
                to: "utils".into(),
                count: 10,
            },
            ImportEdge {
                from: "utils".into(),
                to: "types".into(),
                count: 5,
            },
        ],
    });
    let text = match render(&receipt, AnalysisFormat::Mermaid).unwrap() {
        RenderedOutput::Text(s) => s,
        RenderedOutput::Binary(_) => panic!("expected text"),
    };
    assert!(text.starts_with("graph TD\n"), "Should start with graph TD");
    assert!(text.contains("core"), "Should contain source node");
    assert!(text.contains("utils"), "Should contain target node");
    assert!(text.contains("|10|"), "Should contain edge weight");
}

// ===========================================================================
// JSON-LD section test
// ===========================================================================

#[test]
fn jsonld_renders_schema_org_structure() {
    let mut receipt = base_receipt();
    receipt.derived = Some(minimal_derived());
    let text = match render(&receipt, AnalysisFormat::Jsonld).unwrap() {
        RenderedOutput::Text(s) => s,
        RenderedOutput::Binary(_) => panic!("expected text"),
    };
    assert!(
        text.contains("schema.org"),
        "Should contain schema.org context"
    );
    assert!(
        text.contains("SoftwareSourceCode"),
        "Should contain SoftwareSourceCode type"
    );
    assert!(
        text.contains("codeLines"),
        "Should contain codeLines property"
    );
}

// ===========================================================================
// XML section test
// ===========================================================================

#[test]
fn xml_renders_totals_attributes() {
    let mut receipt = base_receipt();
    receipt.derived = Some(minimal_derived());
    let text = match render(&receipt, AnalysisFormat::Xml).unwrap() {
        RenderedOutput::Text(s) => s,
        RenderedOutput::Binary(_) => panic!("expected text"),
    };
    assert!(
        text.starts_with("<analysis>") && text.ends_with("</analysis>"),
        "Should be well-formed XML"
    );
    assert!(text.contains("files="), "Should contain files attribute");
    assert!(text.contains("code="), "Should contain code attribute");
    assert!(text.contains("tokens="), "Should contain tokens attribute");
}

// ===========================================================================
// Helpers for minimal derived
// ===========================================================================

fn minimal_derived() -> DerivedReport {
    DerivedReport {
        totals: DerivedTotals {
            files: 1,
            code: 100,
            comments: 10,
            blanks: 5,
            lines: 115,
            bytes: 1000,
            tokens: 250,
        },
        doc_density: RatioReport {
            total: RatioRow {
                key: "total".into(),
                numerator: 10,
                denominator: 110,
                ratio: 0.0909,
            },
            by_lang: vec![],
            by_module: vec![],
        },
        whitespace: RatioReport {
            total: RatioRow {
                key: "total".into(),
                numerator: 5,
                denominator: 115,
                ratio: 0.0435,
            },
            by_lang: vec![],
            by_module: vec![],
        },
        verbosity: RateReport {
            total: RateRow {
                key: "total".into(),
                numerator: 1000,
                denominator: 115,
                rate: 8.70,
            },
            by_lang: vec![],
            by_module: vec![],
        },
        max_file: MaxFileReport {
            overall: FileStatRow {
                path: "src/lib.rs".into(),
                module: "src".into(),
                lang: "Rust".into(),
                code: 100,
                comments: 10,
                blanks: 5,
                lines: 115,
                bytes: 1000,
                tokens: 250,
                doc_pct: Some(0.09),
                bytes_per_line: Some(8.7),
                depth: 1,
            },
            by_lang: vec![],
            by_module: vec![],
        },
        lang_purity: LangPurityReport { rows: vec![] },
        nesting: NestingReport {
            max: 1,
            avg: 1.0,
            by_module: vec![],
        },
        test_density: TestDensityReport {
            test_lines: 0,
            prod_lines: 100,
            test_files: 0,
            prod_files: 1,
            ratio: 0.0,
        },
        boilerplate: BoilerplateReport {
            infra_lines: 0,
            logic_lines: 100,
            ratio: 0.0,
            infra_langs: vec![],
        },
        polyglot: PolyglotReport {
            lang_count: 1,
            entropy: 0.0,
            dominant_lang: "Rust".into(),
            dominant_lines: 100,
            dominant_pct: 1.0,
        },
        distribution: DistributionReport {
            count: 1,
            min: 115,
            max: 115,
            mean: 115.0,
            median: 115.0,
            p90: 115.0,
            p99: 115.0,
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
            minutes: 5.75,
            lines_per_minute: 20,
            basis_lines: 115,
        },
        context_window: None,
        cocomo: None,
        todo: None,
        integrity: IntegrityReport {
            algo: "blake3".into(),
            hash: "0".repeat(64),
            entries: 1,
        },
    }
}

fn minimal_derived_with_ratios(
    doc_ratio: f64,
    whitespace_ratio: f64,
    test_ratio: f64,
) -> DerivedReport {
    let mut d = minimal_derived();
    d.doc_density.total.ratio = doc_ratio;
    d.whitespace.total.ratio = whitespace_ratio;
    d.test_density.ratio = test_ratio;
    d
}
