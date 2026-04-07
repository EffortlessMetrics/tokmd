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
        effort: None,
        complexity: None,
        api_surface: None,
        fun: None,
    }
}

#[test]
fn complexity_baseline_from_analysis_fallback_totals() {
    let mut receipt = base_receipt();

    // Instead of instantiating DerivedReport manually, we can parse it from json
    // because the struct is large and has many fields
    let derived_json = r#"{
        "totals": {
            "files": 42,
            "code": 1337,
            "comments": 0,
            "blanks": 0,
            "lines": 0,
            "bytes": 0,
            "tokens": 0
        },
        "doc_density": { "total": {"key": "", "numerator": 0, "denominator": 0, "ratio": 0.0}, "by_lang": [], "by_module": [] },
        "whitespace": { "total": {"key": "", "numerator": 0, "denominator": 0, "ratio": 0.0}, "by_lang": [], "by_module": [] },
        "verbosity": { "total": {"key": "", "numerator": 0, "denominator": 0, "rate": 0.0}, "by_lang": [], "by_module": [] },
        "max_file": { "overall": {"path": "", "module": "", "lang": "", "code": 0, "comments": 0, "blanks": 0, "lines": 0, "bytes": 0, "tokens": 0, "depth": 0}, "by_lang": [], "by_module": [] },
        "lang_purity": { "rows": [] },
        "nesting": { "max": 0, "avg": 0.0, "by_module": [] },
        "test_density": { "test_lines": 0, "prod_lines": 0, "test_files": 0, "prod_files": 0, "ratio": 0.0 },
        "boilerplate": { "infra_lines": 0, "logic_lines": 0, "ratio": 0.0, "infra_langs": [] },
        "polyglot": { "lang_count": 0, "entropy": 0.0, "dominant_lang": "", "dominant_lines": 0, "dominant_pct": 0.0 },
        "distribution": { "count": 0, "min": 0, "max": 0, "mean": 0.0, "median": 0.0, "p90": 0.0, "p95": 0.0, "p99": 0.0, "gini": 0.0, "pareto": 0.0 },
        "histogram": [],
        "top": { "largest_lines": [], "largest_bytes": [], "largest_tokens": [], "most_complex": [], "least_documented": [], "most_dense": [] },
        "tree": null,
        "reading_time": { "minutes": 0.0, "lines_per_minute": 0, "basis_lines": 0 },
        "context_window": null,
        "cocomo": null,
        "todo": null,
        "integrity": { "algo": "", "hash": "", "entries": 0 }
    }"#;

    let derived: DerivedReport = serde_json::from_str(derived_json).unwrap();
    receipt.derived = Some(derived);

    // Explicitly set complexity to None to trigger fallback logic
    receipt.complexity = None;

    let baseline = ComplexityBaseline::from_analysis(&receipt);

    // Complexity should be None but fallback metrics should be present
    assert!(baseline.complexity.is_none());
    assert!(baseline.files.is_empty());

    // Metrics should have structural counts from derived totals
    assert_eq!(baseline.metrics.total_files, 42);
    assert_eq!(baseline.metrics.total_code_lines, 1337);

    // Other metrics should be zero/default
    assert_eq!(baseline.metrics.function_count, 0);
    assert_eq!(baseline.metrics.max_cyclomatic, 0);
}
