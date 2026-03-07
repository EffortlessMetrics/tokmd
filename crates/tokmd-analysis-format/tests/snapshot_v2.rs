//! Extended golden snapshot tests for analysis format rendering (v2).
//!
//! Covers additional receipt combinations: complexity, topics, git,
//! duplicates, and multi-section receipts across output formats.

use tokmd_analysis_format::{RenderedOutput, render};
use tokmd_analysis_types::*;
use tokmd_types::{AnalysisFormat, ScanStatus, ToolInfo};

// ---------------------------------------------------------------------------
// Fixtures
// ---------------------------------------------------------------------------

fn minimal_receipt() -> AnalysisReceipt {
    AnalysisReceipt {
        effort: None,
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

fn text(output: RenderedOutput) -> String {
    match output {
        RenderedOutput::Text(s) => s,
        RenderedOutput::Binary(_) => panic!("expected text output"),
    }
}

// ---------------------------------------------------------------------------
// Complexity receipt snapshots
// ---------------------------------------------------------------------------

#[test]
fn snapshot_v2_md_with_complexity() {
    let mut receipt = minimal_receipt();
    receipt.complexity = Some(ComplexityReport {
        total_functions: 42,
        avg_function_length: 18.5,
        max_function_length: 120,
        avg_cyclomatic: 3.2,
        max_cyclomatic: 15,
        avg_cognitive: Some(4.1),
        max_cognitive: Some(22),
        avg_nesting_depth: Some(2.3),
        max_nesting_depth: Some(6),
        high_risk_files: 2,
        histogram: None,
        halstead: None,
        maintainability_index: None,
        technical_debt: None,
        files: vec![
            FileComplexity {
                path: "src/parser.rs".into(),
                module: "src".into(),
                function_count: 12,
                max_function_length: 120,
                cyclomatic_complexity: 15,
                cognitive_complexity: Some(22),
                max_nesting: Some(6),
                risk_level: ComplexityRisk::High,
                functions: None,
            },
            FileComplexity {
                path: "src/lib.rs".into(),
                module: "src".into(),
                function_count: 8,
                max_function_length: 45,
                cyclomatic_complexity: 4,
                cognitive_complexity: Some(3),
                max_nesting: Some(2),
                risk_level: ComplexityRisk::Low,
                functions: None,
            },
        ],
    });
    let rendered = text(render(&receipt, AnalysisFormat::Md).unwrap());
    insta::assert_snapshot!("v2_md_complexity", rendered);
}

#[test]
fn snapshot_v2_json_with_complexity() {
    let mut receipt = minimal_receipt();
    receipt.complexity = Some(ComplexityReport {
        total_functions: 10,
        avg_function_length: 25.0,
        max_function_length: 80,
        avg_cyclomatic: 5.0,
        max_cyclomatic: 12,
        avg_cognitive: None,
        max_cognitive: None,
        avg_nesting_depth: None,
        max_nesting_depth: None,
        high_risk_files: 1,
        histogram: None,
        halstead: None,
        maintainability_index: None,
        technical_debt: None,
        files: vec![FileComplexity {
            path: "src/main.rs".into(),
            module: "src".into(),
            function_count: 10,
            max_function_length: 80,
            cyclomatic_complexity: 12,
            cognitive_complexity: None,
            max_nesting: None,
            risk_level: ComplexityRisk::Moderate,
            functions: None,
        }],
    });
    let rendered = text(render(&receipt, AnalysisFormat::Json).unwrap());
    insta::assert_snapshot!("v2_json_complexity", rendered);
}

// ---------------------------------------------------------------------------
// Topics receipt snapshots
// ---------------------------------------------------------------------------

#[test]
fn snapshot_v2_md_with_topics() {
    let mut receipt = minimal_receipt();
    let mut per_module = std::collections::BTreeMap::new();
    per_module.insert(
        "src".into(),
        vec![
            TopicTerm {
                term: "parsing".into(),
                score: 0.85,
                tf: 12,
                df: 3,
            },
            TopicTerm {
                term: "ast".into(),
                score: 0.72,
                tf: 8,
                df: 2,
            },
        ],
    );
    receipt.topics = Some(TopicClouds {
        overall: vec![
            TopicTerm {
                term: "compiler".into(),
                score: 0.90,
                tf: 20,
                df: 5,
            },
            TopicTerm {
                term: "optimization".into(),
                score: 0.65,
                tf: 6,
                df: 2,
            },
        ],
        per_module,
    });
    let rendered = text(render(&receipt, AnalysisFormat::Md).unwrap());
    insta::assert_snapshot!("v2_md_topics", rendered);
}

// ---------------------------------------------------------------------------
// Duplicate report snapshots
// ---------------------------------------------------------------------------

#[test]
fn snapshot_v2_md_with_duplicates() {
    let mut receipt = minimal_receipt();
    receipt.dup = Some(DuplicateReport {
        groups: vec![DuplicateGroup {
            hash: "abc123".into(),
            bytes: 512,
            files: vec!["src/a.rs".into(), "src/b.rs".into()],
        }],
        wasted_bytes: 512,
        strategy: "blake3".into(),
        density: None,
        near: None,
    });
    let rendered = text(render(&receipt, AnalysisFormat::Md).unwrap());
    insta::assert_snapshot!("v2_md_duplicates", rendered);
}

#[test]
fn snapshot_v2_json_with_duplicates() {
    let mut receipt = minimal_receipt();
    receipt.dup = Some(DuplicateReport {
        groups: vec![
            DuplicateGroup {
                hash: "abc123".into(),
                bytes: 512,
                files: vec!["src/a.rs".into(), "src/b.rs".into()],
            },
            DuplicateGroup {
                hash: "def456".into(),
                bytes: 1024,
                files: vec!["lib/x.rs".into(), "lib/y.rs".into(), "lib/z.rs".into()],
            },
        ],
        wasted_bytes: 2560,
        strategy: "blake3".into(),
        density: None,
        near: None,
    });
    let rendered = text(render(&receipt, AnalysisFormat::Json).unwrap());
    insta::assert_snapshot!("v2_json_duplicates", rendered);
}

// ---------------------------------------------------------------------------
// License report snapshots
// ---------------------------------------------------------------------------

#[test]
fn snapshot_v2_md_with_license() {
    let mut receipt = minimal_receipt();
    receipt.license = Some(LicenseReport {
        effective: Some("Apache-2.0".into()),
        findings: vec![
            LicenseFinding {
                spdx: "Apache-2.0".into(),
                confidence: 0.98,
                source_path: "LICENSE".into(),
                source_kind: LicenseSourceKind::Text,
            },
            LicenseFinding {
                spdx: "MIT".into(),
                confidence: 0.85,
                source_path: "Cargo.toml".into(),
                source_kind: LicenseSourceKind::Metadata,
            },
        ],
    });
    let rendered = text(render(&receipt, AnalysisFormat::Md).unwrap());
    insta::assert_snapshot!("v2_md_license", rendered);
}

// ---------------------------------------------------------------------------
// Warnings receipt snapshot
// ---------------------------------------------------------------------------

#[test]
fn snapshot_v2_md_with_warnings() {
    let mut receipt = minimal_receipt();
    receipt.warnings = vec![
        "Skipped 3 files exceeding max_file_bytes".into(),
        "Git history unavailable".into(),
    ];
    let rendered = text(render(&receipt, AnalysisFormat::Md).unwrap());
    insta::assert_snapshot!("v2_md_warnings", rendered);
}

// ---------------------------------------------------------------------------
// XML format with complexity
// ---------------------------------------------------------------------------

#[test]
fn snapshot_v2_xml_with_complexity() {
    let mut receipt = minimal_receipt();
    receipt.complexity = Some(ComplexityReport {
        total_functions: 5,
        avg_function_length: 30.0,
        max_function_length: 60,
        avg_cyclomatic: 4.0,
        max_cyclomatic: 8,
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
    let rendered = text(render(&receipt, AnalysisFormat::Xml).unwrap());
    insta::assert_snapshot!("v2_xml_complexity", rendered);
}

// ---------------------------------------------------------------------------
// JSON-LD with eco label
// ---------------------------------------------------------------------------

#[test]
fn snapshot_v2_jsonld_with_eco_label() {
    let mut receipt = minimal_receipt();
    receipt.fun = Some(FunReport {
        eco_label: Some(EcoLabel {
            score: 88.0,
            label: "A".into(),
            bytes: 250_000,
            notes: "Size-based eco label (0.24 MB)".into(),
        }),
    });
    let rendered = text(render(&receipt, AnalysisFormat::Jsonld).unwrap());
    insta::assert_snapshot!("v2_jsonld_eco_label", rendered);
}

