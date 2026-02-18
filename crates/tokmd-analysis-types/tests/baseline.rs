use tokmd_analysis_types::*;
use tokmd_types::{ScanStatus, ToolInfo};

fn base_receipt() -> AnalysisReceipt {
    AnalysisReceipt {
        schema_version: ANALYSIS_SCHEMA_VERSION,
        generated_at_ms: 0,
        tool: ToolInfo {
            name: "tokmd".to_string(),
            version: "0.0.0".to_string(),
        },
        mode: "analysis".to_string(),
        status: ScanStatus::Complete,
        warnings: Vec::new(),
        source: AnalysisSource {
            inputs: vec![".".to_string()],
            export_path: None,
            base_receipt_path: None,
            export_schema_version: None,
            export_generated_at_ms: None,
            base_signature: None,
            module_roots: vec!["src".to_string()],
            module_depth: 1,
            children: "separate".to_string(),
        },
        args: AnalysisArgsMeta {
            preset: "receipt".to_string(),
            format: "json".to_string(),
            window_tokens: None,
            git: None,
            max_files: None,
            max_bytes: None,
            max_commits: None,
            max_commit_files: None,
            max_file_bytes: None,
            import_granularity: "module".to_string(),
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

#[test]
fn complexity_histogram_to_ascii_formats_lines() {
    let histogram = ComplexityHistogram {
        buckets: vec![0, 5, 10],
        counts: vec![2, 0, 4],
        total: 6,
    };
    let ascii = histogram.to_ascii(10);
    let lines: Vec<&str> = ascii.trim_end().lines().collect();
    assert_eq!(lines.len(), 3);
    assert!(lines[0].contains("0-4"));
    assert!(lines[1].contains("5-9"));
    assert!(lines[2].contains("10+"));

    let counts: Vec<u32> = lines
        .iter()
        .map(|line| {
            line.split_whitespace()
                .last()
                .unwrap()
                .parse::<u32>()
                .unwrap()
        })
        .collect();
    assert_eq!(counts, vec![2, 0, 4]);
}

#[test]
fn complexity_baseline_new_defaults() {
    let baseline = ComplexityBaseline::new();
    assert_eq!(baseline.baseline_version, BASELINE_VERSION);
    assert!(baseline.generated_at.is_empty());
    assert!(baseline.commit.is_none());
    assert!(baseline.files.is_empty());
    assert!(baseline.complexity.is_none());
    assert_eq!(baseline.metrics.total_code_lines, 0);
    assert_eq!(baseline.metrics.total_files, 0);
    assert_eq!(baseline.metrics.max_cyclomatic, 0);
}

#[test]
fn complexity_baseline_from_analysis_with_complexity() {
    let mut receipt = base_receipt();
    receipt.complexity = Some(ComplexityReport {
        total_functions: 3,
        avg_function_length: 10.0,
        max_function_length: 20,
        avg_cyclomatic: 2.5,
        max_cyclomatic: 5,
        avg_cognitive: Some(1.2),
        max_cognitive: Some(3),
        avg_nesting_depth: Some(0.5),
        max_nesting_depth: Some(2),
        high_risk_files: 1,
        histogram: None,
        halstead: None,
        maintainability_index: None,
        technical_debt: None,
        files: vec![FileComplexity {
            path: "src/lib.rs".to_string(),
            module: "src".to_string(),
            function_count: 2,
            max_function_length: 20,
            cyclomatic_complexity: 5,
            cognitive_complexity: Some(3),
            max_nesting: Some(2),
            risk_level: ComplexityRisk::High,
            functions: Some(vec![FunctionComplexityDetail {
                name: "do_work".to_string(),
                line_start: 1,
                line_end: 5,
                length: 5,
                cyclomatic: 3,
                cognitive: Some(2),
                max_nesting: Some(1),
                param_count: Some(2),
            }]),
        }],
    });

    let baseline = ComplexityBaseline::from_analysis(&receipt);
    assert_eq!(baseline.baseline_version, BASELINE_VERSION);
    assert_eq!(baseline.generated_at, "1970-01-01T00:00:00.000Z");
    assert_eq!(baseline.metrics.max_cyclomatic, 5);
    assert_eq!(baseline.metrics.function_count, 3);
    assert_eq!(baseline.files.len(), 1);
    assert_eq!(baseline.files[0].path, "src/lib.rs");
    assert!(baseline.complexity.is_some());
}

#[test]
fn complexity_baseline_from_analysis_without_complexity() {
    let receipt = base_receipt();
    let baseline = ComplexityBaseline::from_analysis(&receipt);
    assert!(baseline.files.is_empty());
    assert!(baseline.complexity.is_none());
    assert_eq!(baseline.metrics.total_code_lines, 0);
    assert_eq!(baseline.metrics.total_files, 0);
}
