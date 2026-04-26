//! Snapshot tests for tokmd-format analysis – wave 50.
//!
//! Covers: Markdown (receipt preset), JSON, XML, empty receipt,
//! multiple enrichment sections, archetype+topics, license+entropy,
//! and derived-only rendering.

use std::collections::BTreeMap;

use tokmd_analysis_types::{
    ANALYSIS_SCHEMA_VERSION, AnalysisArgsMeta, AnalysisReceipt, AnalysisSource, Archetype,
    BoilerplateReport, DerivedReport, DerivedTotals, DistributionReport, EntropyClass,
    EntropyFinding, EntropyReport, FileStatRow, HistogramBucket, ImportEdge, ImportReport,
    IntegrityReport, LangPurityReport, LicenseFinding, LicenseReport, LicenseSourceKind,
    MaxFileReport, NestingReport, PolyglotReport, RateReport, RateRow, RatioReport, RatioRow,
    ReadingTimeReport, TestDensityReport, TodoReport, TodoTagRow, TopOffenders, TopicClouds,
    TopicTerm,
};
use tokmd_format::analysis::{RenderedOutput, render};
use tokmd_types::{AnalysisFormat, ScanStatus, ToolInfo};

// ---------------------------------------------------------------------------
// Shared fixtures
// ---------------------------------------------------------------------------

fn fixed_tool() -> ToolInfo {
    ToolInfo {
        name: "tokmd".to_string(),
        version: "0.0.0-test".to_string(),
    }
}

fn minimal_source() -> AnalysisSource {
    AnalysisSource {
        inputs: vec![".".to_string()],
        export_path: None,
        base_receipt_path: None,
        export_schema_version: None,
        export_generated_at_ms: None,
        base_signature: None,
        module_roots: vec![],
        module_depth: 1,
        children: "collapse".to_string(),
    }
}

fn minimal_args() -> AnalysisArgsMeta {
    AnalysisArgsMeta {
        preset: "receipt".to_string(),
        format: "md".to_string(),
        window_tokens: None,
        git: None,
        max_files: None,
        max_bytes: None,
        max_commits: None,
        max_commit_files: None,
        max_file_bytes: None,
        import_granularity: "module".to_string(),
    }
}

fn empty_receipt() -> AnalysisReceipt {
    AnalysisReceipt {
        effort: None,
        schema_version: ANALYSIS_SCHEMA_VERSION,
        generated_at_ms: 0,
        tool: fixed_tool(),
        mode: "analyze".to_string(),
        status: ScanStatus::Complete,
        warnings: vec![],
        source: minimal_source(),
        args: minimal_args(),
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

fn stub_file_stat() -> FileStatRow {
    FileStatRow {
        path: "src/lib.rs".to_string(),
        module: "src".to_string(),
        lang: "Rust".to_string(),
        code: 500,
        comments: 80,
        blanks: 40,
        lines: 620,
        bytes: 18000,
        tokens: 1250,
        doc_pct: Some(0.16),
        bytes_per_line: Some(29.03),
        depth: 1,
    }
}

fn sample_derived() -> DerivedReport {
    DerivedReport {
        totals: DerivedTotals {
            files: 10,
            code: 2000,
            comments: 300,
            blanks: 200,
            lines: 2500,
            bytes: 75000,
            tokens: 5000,
        },
        doc_density: RatioReport {
            total: RatioRow {
                key: "total".into(),
                numerator: 300,
                denominator: 2000,
                ratio: 0.15,
            },
            by_lang: vec![RatioRow {
                key: "Rust".into(),
                numerator: 300,
                denominator: 2000,
                ratio: 0.15,
            }],
            by_module: vec![],
        },
        whitespace: RatioReport {
            total: RatioRow {
                key: "total".into(),
                numerator: 200,
                denominator: 2300,
                ratio: 0.087,
            },
            by_lang: vec![],
            by_module: vec![],
        },
        verbosity: RateReport {
            total: RateRow {
                key: "total".into(),
                numerator: 75000,
                denominator: 2500,
                rate: 30.0,
            },
            by_lang: vec![],
            by_module: vec![],
        },
        max_file: MaxFileReport {
            overall: stub_file_stat(),
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
            test_lines: 400,
            prod_lines: 2100,
            test_files: 3,
            prod_files: 7,
            ratio: 0.19,
        },
        boilerplate: BoilerplateReport {
            infra_lines: 200,
            logic_lines: 1800,
            ratio: 0.10,
            infra_langs: vec!["TOML".into()],
        },
        polyglot: PolyglotReport {
            lang_count: 2,
            entropy: 0.45,
            dominant_lang: "Rust".into(),
            dominant_lines: 1800,
            dominant_pct: 0.90,
        },
        distribution: DistributionReport {
            count: 10,
            min: 20,
            max: 500,
            mean: 200.0,
            median: 180.0,
            p90: 450.0,
            p99: 500.0,
            gini: 0.35,
        },
        histogram: vec![
            HistogramBucket {
                label: "0–100".into(),
                min: 0,
                max: Some(100),
                files: 4,
                pct: 0.40,
            },
            HistogramBucket {
                label: "101–500".into(),
                min: 101,
                max: Some(500),
                files: 6,
                pct: 0.60,
            },
        ],
        top: TopOffenders {
            largest_lines: vec![stub_file_stat()],
            largest_tokens: vec![stub_file_stat()],
            largest_bytes: vec![stub_file_stat()],
            least_documented: vec![stub_file_stat()],
            most_dense: vec![stub_file_stat()],
        },
        tree: None,
        reading_time: ReadingTimeReport {
            minutes: 12.5,
            lines_per_minute: 200,
            basis_lines: 2500,
        },
        context_window: None,
        cocomo: None,
        todo: None,
        integrity: IntegrityReport {
            algo: "blake3".into(),
            hash: "abc123def456".into(),
            entries: 10,
        },
    }
}

fn extract_text(output: RenderedOutput) -> String {
    match output {
        RenderedOutput::Text(t) => t,
        RenderedOutput::Binary(_) => panic!("expected text output"),
    }
}

// ── 1. Markdown receipt preset with derived ───────────────────────────

#[test]
fn snapshot_md_receipt_preset() {
    let mut receipt = empty_receipt();
    receipt.derived = Some(sample_derived());
    let text = extract_text(render(&receipt, AnalysisFormat::Md).unwrap());
    insta::assert_snapshot!(text);
}

// ── 2. JSON rendering ────────────────────────────────────────────────

#[test]
fn snapshot_json_receipt() {
    let mut receipt = empty_receipt();
    receipt.derived = Some(sample_derived());
    let text = extract_text(render(&receipt, AnalysisFormat::Json).unwrap());
    let v: serde_json::Value = serde_json::from_str(&text).unwrap();
    insta::assert_json_snapshot!(v);
}

// ── 3. XML rendering ────────────────────────────────────────────────

#[test]
fn snapshot_xml_receipt() {
    let mut receipt = empty_receipt();
    receipt.derived = Some(sample_derived());
    let text = extract_text(render(&receipt, AnalysisFormat::Xml).unwrap());
    insta::assert_snapshot!(text);
}

// ── 4. Empty receipt rendering (all None) ────────────────────────────

#[test]
fn snapshot_md_empty_receipt() {
    let receipt = empty_receipt();
    let text = extract_text(render(&receipt, AnalysisFormat::Md).unwrap());
    insta::assert_snapshot!(text);
}

#[test]
fn snapshot_json_empty_receipt() {
    let receipt = empty_receipt();
    let text = extract_text(render(&receipt, AnalysisFormat::Json).unwrap());
    let v: serde_json::Value = serde_json::from_str(&text).unwrap();
    insta::assert_json_snapshot!(v);
}

#[test]
fn snapshot_xml_empty_receipt() {
    let receipt = empty_receipt();
    let text = extract_text(render(&receipt, AnalysisFormat::Xml).unwrap());
    insta::assert_snapshot!(text);
}

// ── 5. Multiple enrichment sections ──────────────────────────────────

#[test]
fn snapshot_md_multi_enrichment() {
    let mut receipt = empty_receipt();
    receipt.derived = Some(sample_derived());
    receipt.archetype = Some(Archetype {
        kind: "web-app".into(),
        evidence: vec!["express".into(), "react".into(), "package.json".into()],
    });
    receipt.topics = Some(TopicClouds {
        per_module: {
            let mut m = BTreeMap::new();
            m.insert(
                "src".to_string(),
                vec![
                    TopicTerm {
                        term: "parser".into(),
                        score: 0.85,
                        tf: 12,
                        df: 3,
                    },
                    TopicTerm {
                        term: "token".into(),
                        score: 0.72,
                        tf: 8,
                        df: 5,
                    },
                ],
            );
            m
        },
        overall: vec![TopicTerm {
            term: "parser".into(),
            score: 0.85,
            tf: 12,
            df: 3,
        }],
    });
    receipt.entropy = Some(EntropyReport {
        suspects: vec![EntropyFinding {
            path: "secrets.env".into(),
            module: ".".into(),
            entropy_bits_per_byte: 7.2,
            sample_bytes: 256,
            class: EntropyClass::High,
        }],
    });
    receipt.license = Some(LicenseReport {
        findings: vec![LicenseFinding {
            spdx: "MIT".into(),
            confidence: 0.95,
            source_path: "LICENSE".into(),
            source_kind: LicenseSourceKind::Text,
        }],
        effective: Some("MIT".into()),
    });
    let text = extract_text(render(&receipt, AnalysisFormat::Md).unwrap());
    insta::assert_snapshot!(text);
}

// ── 6. JSON with multiple enrichments ────────────────────────────────

#[test]
fn snapshot_json_multi_enrichment() {
    let mut receipt = empty_receipt();
    receipt.derived = Some(sample_derived());
    receipt.archetype = Some(Archetype {
        kind: "cli-tool".into(),
        evidence: vec!["clap".into(), "main.rs".into()],
    });
    receipt.imports = Some(ImportReport {
        granularity: "module".to_string(),
        edges: vec![
            ImportEdge {
                from: "src/main.rs".into(),
                to: "src/lib.rs".into(),
                count: 1,
            },
            ImportEdge {
                from: "src/lib.rs".into(),
                to: "src/utils.rs".into(),
                count: 3,
            },
        ],
    });
    let text = extract_text(render(&receipt, AnalysisFormat::Json).unwrap());
    let v: serde_json::Value = serde_json::from_str(&text).unwrap();
    insta::assert_json_snapshot!(v);
}

// ── 7. Derived with TODO report ──────────────────────────────────────

#[test]
fn snapshot_md_derived_with_todos() {
    let mut receipt = empty_receipt();
    let mut derived = sample_derived();
    derived.todo = Some(TodoReport {
        total: 15,
        density_per_kloc: 7.5,
        tags: vec![
            TodoTagRow {
                tag: "TODO".into(),
                count: 10,
            },
            TodoTagRow {
                tag: "FIXME".into(),
                count: 3,
            },
            TodoTagRow {
                tag: "HACK".into(),
                count: 2,
            },
        ],
    });
    receipt.derived = Some(derived);
    let text = extract_text(render(&receipt, AnalysisFormat::Md).unwrap());
    insta::assert_snapshot!(text);
}

// ── 8. Tree format with derived ──────────────────────────────────────

#[test]
fn snapshot_tree_derived() {
    let mut receipt = empty_receipt();
    receipt.derived = Some(sample_derived());
    let text = extract_text(render(&receipt, AnalysisFormat::Tree).unwrap());
    insta::assert_snapshot!(text);
}

// ── 9. Mermaid with imports ──────────────────────────────────────────

#[test]
fn snapshot_mermaid_imports_w50() {
    let mut receipt = empty_receipt();
    receipt.imports = Some(ImportReport {
        granularity: "file".to_string(),
        edges: vec![
            ImportEdge {
                from: "src/main.rs".into(),
                to: "src/config.rs".into(),
                count: 2,
            },
            ImportEdge {
                from: "src/config.rs".into(),
                to: "src/types.rs".into(),
                count: 1,
            },
            ImportEdge {
                from: "src/main.rs".into(),
                to: "src/types.rs".into(),
                count: 3,
            },
            ImportEdge {
                from: "src/types.rs".into(),
                to: "src/utils.rs".into(),
                count: 1,
            },
        ],
    });
    let text = extract_text(render(&receipt, AnalysisFormat::Mermaid).unwrap());
    insta::assert_snapshot!(text);
}

// ── 10. XML with archetype and license ───────────────────────────────

#[test]
fn snapshot_xml_archetype_license() {
    let mut receipt = empty_receipt();
    receipt.archetype = Some(Archetype {
        kind: "library".into(),
        evidence: vec!["lib.rs".into(), "Cargo.toml".into()],
    });
    receipt.license = Some(LicenseReport {
        findings: vec![
            LicenseFinding {
                spdx: "Apache-2.0".into(),
                confidence: 0.98,
                source_path: "LICENSE-APACHE".into(),
                source_kind: LicenseSourceKind::Text,
            },
            LicenseFinding {
                spdx: "MIT".into(),
                confidence: 0.95,
                source_path: "LICENSE-MIT".into(),
                source_kind: LicenseSourceKind::Text,
            },
        ],
        effective: Some("Apache-2.0 OR MIT".into()),
    });
    let text = extract_text(render(&receipt, AnalysisFormat::Xml).unwrap());
    insta::assert_snapshot!(text);
}
