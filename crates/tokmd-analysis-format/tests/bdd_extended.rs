//! Extended BDD-style tests for tokmd-analysis-format rendering.
//!
//! Additional coverage for HTML output, JSON with git section, markdown
//! consistency, and format dispatch edge cases.

use tokmd_analysis_format::{RenderedOutput, render};
use tokmd_analysis_types::*;
use tokmd_types::{AnalysisFormat, ScanStatus, ToolInfo};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn minimal_receipt() -> AnalysisReceipt {
    AnalysisReceipt {
        effort: None,
        schema_version: ANALYSIS_SCHEMA_VERSION,
        generated_at_ms: 0,
        tool: ToolInfo {
            name: "tokmd".into(),
            version: "test".into(),
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

fn extract_text(output: RenderedOutput) -> String {
    match output {
        RenderedOutput::Text(s) => s,
        RenderedOutput::Binary(_) => panic!("expected text output"),
    }
}

// ===========================================================================
// Scenario: HTML format produces valid HTML structure
// ===========================================================================
#[test]
fn given_minimal_receipt_when_rendering_html_then_produces_html_tags() {
    let receipt = minimal_receipt();
    let output = render(&receipt, AnalysisFormat::Html).unwrap();
    let text = extract_text(output);

    assert!(text.contains("<html"), "should contain opening html tag");
    assert!(text.contains("</html>"), "should contain closing html tag");
    assert!(text.contains("tokmd"), "should reference tokmd");
}

// ===========================================================================
// Scenario: HTML with derived data includes totals
// ===========================================================================
#[test]
fn given_receipt_with_derived_when_rendering_html_then_includes_totals() {
    let mut receipt = minimal_receipt();
    receipt.derived = Some(sample_derived());
    let output = render(&receipt, AnalysisFormat::Html).unwrap();
    let text = extract_text(output);

    assert!(text.contains("500"), "should include code line count");
    assert!(text.contains("html"), "should be HTML format");
}

// ===========================================================================
// Scenario: JSON with git section roundtrips correctly
// ===========================================================================
#[test]
fn given_receipt_with_git_section_when_json_roundtrip_then_data_preserved() {
    let mut receipt = minimal_receipt();
    receipt.git = Some(GitReport {
        commits_scanned: 42,
        files_seen: 10,
        hotspots: vec![HotspotRow {
            path: "src/main.rs".into(),
            commits: 15,
            lines: 200,
            score: 3000,
        }],
        bus_factor: vec![BusFactorRow {
            module: "src".into(),
            authors: 3,
        }],
        freshness: FreshnessReport {
            threshold_days: 365,
            stale_files: 2,
            total_files: 10,
            stale_pct: 0.2,
            by_module: vec![],
        },
        coupling: vec![],
        age_distribution: None,
        intent: None,
    });

    let output = render(&receipt, AnalysisFormat::Json).unwrap();
    let text = extract_text(output);
    let parsed: AnalysisReceipt = serde_json::from_str(&text).unwrap();

    let git = parsed.git.expect("git section should be present");
    assert_eq!(git.commits_scanned, 42);
    assert_eq!(git.files_seen, 10);
    assert_eq!(git.hotspots.len(), 1);
    assert_eq!(git.hotspots[0].score, 3000);
    assert_eq!(git.bus_factor[0].authors, 3);
}

// ===========================================================================
// Scenario: JSON with imports section preserved in roundtrip
// ===========================================================================
#[test]
fn given_receipt_with_imports_when_json_roundtrip_then_edges_preserved() {
    let mut receipt = minimal_receipt();
    receipt.imports = Some(ImportReport {
        granularity: "file".into(),
        edges: vec![
            ImportEdge {
                from: "src/main.rs".into(),
                to: "serde".into(),
                count: 5,
            },
            ImportEdge {
                from: "src/main.rs".into(),
                to: "anyhow".into(),
                count: 2,
            },
        ],
    });

    let output = render(&receipt, AnalysisFormat::Json).unwrap();
    let text = extract_text(output);
    let parsed: AnalysisReceipt = serde_json::from_str(&text).unwrap();

    let imports = parsed.imports.expect("imports section present");
    assert_eq!(imports.granularity, "file");
    assert_eq!(imports.edges.len(), 2);
    assert_eq!(imports.edges[0].count, 5);
}

// ===========================================================================
// Scenario: Markdown with git section renders hotspots table
// ===========================================================================
#[test]
fn given_receipt_with_git_when_rendering_md_then_hotspot_table_present() {
    let mut receipt = minimal_receipt();
    receipt.git = Some(GitReport {
        commits_scanned: 10,
        files_seen: 3,
        hotspots: vec![
            HotspotRow {
                path: "src/main.rs".into(),
                commits: 8,
                lines: 100,
                score: 800,
            },
            HotspotRow {
                path: "src/lib.rs".into(),
                commits: 5,
                lines: 50,
                score: 250,
            },
        ],
        bus_factor: vec![BusFactorRow {
            module: "src".into(),
            authors: 2,
        }],
        freshness: FreshnessReport {
            threshold_days: 365,
            stale_files: 0,
            total_files: 3,
            stale_pct: 0.0,
            by_module: vec![],
        },
        coupling: vec![CouplingRow {
            left: "api".into(),
            right: "db".into(),
            count: 5,
            jaccard: Some(0.75),
            lift: Some(1.2),
            n_left: Some(6),
            n_right: Some(7),
        }],
        age_distribution: None,
        intent: None,
    });

    let output = render(&receipt, AnalysisFormat::Md).unwrap();
    let text = extract_text(output);

    assert!(text.contains("## Git"), "should have Git section header");
    assert!(text.contains("src/main.rs"), "should list hotspot path");
    assert!(text.contains("|src|2|"), "should render bus factor row");
    assert!(text.contains("|api|db|5|"), "should render coupling row");
}

// ===========================================================================
// Scenario: Markdown renders dup density by_module table
// ===========================================================================
#[test]
fn given_receipt_with_dup_density_when_rendering_md_then_module_table_present() {
    let mut receipt = minimal_receipt();
    receipt.dup = Some(DuplicateReport {
        groups: vec![DuplicateGroup {
            hash: "abc123def".into(),
            bytes: 500,
            files: vec!["a.rs".into(), "b.rs".into()],
        }],
        wasted_bytes: 500,
        strategy: "exact-blake3".into(),
        density: Some(DuplicationDensityReport {
            duplicate_groups: 1,
            duplicate_files: 2,
            duplicated_bytes: 1000,
            wasted_bytes: 500,
            wasted_pct_of_codebase: 0.05,
            by_module: vec![ModuleDuplicationDensityRow {
                module: "core".into(),
                duplicate_files: 2,
                wasted_files: 1,
                duplicated_bytes: 1000,
                wasted_bytes: 500,
                module_bytes: 10_000,
                density: 0.05,
            }],
        }),
        near: None,
    });

    let output = render(&receipt, AnalysisFormat::Md).unwrap();
    let text = extract_text(output);

    assert!(
        text.contains("## Duplicates"),
        "should have Duplicates header"
    );
    assert!(
        text.contains("### Duplication density"),
        "should have density subsection"
    );
    assert!(text.contains("|core|"), "should render module row");
    assert!(text.contains("exact-blake3"), "should show strategy");
}

// ===========================================================================
// Scenario: Mermaid without imports shows no graph edges
// ===========================================================================
#[test]
fn given_receipt_without_imports_when_rendering_mermaid_then_no_edge_arrows() {
    let receipt = minimal_receipt();
    let output = render(&receipt, AnalysisFormat::Mermaid).unwrap();
    let text = extract_text(output);

    assert!(text.contains("graph"), "mermaid should produce a graph");
    assert!(
        !text.contains("-->"),
        "should not have edge arrows without imports"
    );
}

// ===========================================================================
// Scenario: Mermaid with imports includes edge arrows
// ===========================================================================
#[test]
fn given_receipt_with_imports_when_rendering_mermaid_then_edges_shown() {
    let mut receipt = minimal_receipt();
    receipt.imports = Some(ImportReport {
        granularity: "module".into(),
        edges: vec![ImportEdge {
            from: "api".into(),
            to: "utils".into(),
            count: 3,
        }],
    });

    let output = render(&receipt, AnalysisFormat::Mermaid).unwrap();
    let text = extract_text(output);

    assert!(text.contains("graph"), "should produce mermaid graph");
    assert!(text.contains("-->"), "should have edge arrow");
    assert!(text.contains("api"), "should reference source module");
    assert!(text.contains("utils"), "should reference target module");
}

// ===========================================================================
// Scenario: XML format renders analysis tags even without derived
// ===========================================================================
#[test]
fn given_minimal_receipt_when_rendering_xml_then_has_analysis_tags() {
    let receipt = minimal_receipt();
    let output = render(&receipt, AnalysisFormat::Xml).unwrap();
    let text = extract_text(output);

    assert!(
        text.contains("<analysis>"),
        "should have opening analysis tag"
    );
    assert!(
        text.contains("</analysis>"),
        "should have closing analysis tag"
    );
}

// ===========================================================================
// Scenario: XML with derived data includes totals attributes
// ===========================================================================
#[test]
fn given_receipt_with_derived_when_rendering_xml_then_totals_present() {
    let mut receipt = minimal_receipt();
    receipt.derived = Some(sample_derived());
    let output = render(&receipt, AnalysisFormat::Xml).unwrap();
    let text = extract_text(output);

    assert!(text.contains("files=\"5\""), "should include file count");
    assert!(text.contains("code=\"500\""), "should include code count");
}

// ===========================================================================
// Scenario: JSON output is valid JSON for all optional section combos
// ===========================================================================
#[test]
fn given_receipt_with_multiple_sections_when_json_then_valid_json() {
    let mut receipt = minimal_receipt();
    receipt.archetype = Some(Archetype {
        kind: "monorepo".into(),
        evidence: vec!["Cargo.toml".into(), "package.json".into()],
    });
    receipt.dup = Some(DuplicateReport {
        groups: vec![],
        wasted_bytes: 0,
        strategy: "exact-blake3".into(),
        density: None,
        near: None,
    });
    receipt.imports = Some(ImportReport {
        granularity: "module".into(),
        edges: vec![],
    });

    let output = render(&receipt, AnalysisFormat::Json).unwrap();
    let text = extract_text(output);

    // Verify it's valid JSON by parsing it
    let val: serde_json::Value = serde_json::from_str(&text).expect("should be valid JSON");
    assert_eq!(val["archetype"]["kind"], "monorepo");
    assert_eq!(val["dup"]["strategy"], "exact-blake3");
    assert_eq!(val["imports"]["granularity"], "module");
}

// ---------------------------------------------------------------------------
// Helper: minimal DerivedReport for tests that need it
// ---------------------------------------------------------------------------

fn sample_derived() -> DerivedReport {
    DerivedReport {
        totals: DerivedTotals {
            files: 5,
            code: 500,
            comments: 80,
            blanks: 40,
            lines: 620,
            bytes: 5000,
            tokens: 1200,
        },
        doc_density: RatioReport {
            total: RatioRow {
                key: "total".into(),
                numerator: 80,
                denominator: 580,
                ratio: 0.1379,
            },
            by_lang: vec![],
            by_module: vec![],
        },
        whitespace: RatioReport {
            total: RatioRow {
                key: "total".into(),
                numerator: 40,
                denominator: 620,
                ratio: 0.0645,
            },
            by_lang: vec![],
            by_module: vec![],
        },
        verbosity: RateReport {
            total: RateRow {
                key: "total".into(),
                numerator: 5000,
                denominator: 620,
                rate: 8.06,
            },
            by_lang: vec![],
            by_module: vec![],
        },
        max_file: MaxFileReport {
            overall: FileStatRow {
                path: "src/main.rs".into(),
                module: "src".into(),
                lang: "Rust".into(),
                code: 200,
                comments: 30,
                blanks: 15,
                lines: 245,
                bytes: 2000,
                tokens: 500,
                doc_pct: Some(0.13),
                bytes_per_line: Some(8.16),
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
            infra_lines: 50,
            logic_lines: 570,
            ratio: 0.0877,
            infra_langs: vec!["TOML".into()],
        },
        polyglot: PolyglotReport {
            lang_count: 2,
            entropy: 0.72,
            dominant_lang: "Rust".into(),
            dominant_lines: 450,
            dominant_pct: 0.9,
        },
        distribution: DistributionReport {
            count: 5,
            min: 25,
            max: 245,
            mean: 124.0,
            median: 100.0,
            p90: 245.0,
            p99: 245.0,
            gini: 0.32,
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
            minutes: 31.0,
            lines_per_minute: 20,
            basis_lines: 620,
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
