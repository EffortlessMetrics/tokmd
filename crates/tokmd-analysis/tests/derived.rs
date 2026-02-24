use tokmd_analysis::{
    AnalysisContext, AnalysisLimits, AnalysisPreset, AnalysisRequest, ImportGranularity,
    NearDupScope, analyze,
};
use tokmd_analysis_types::{AnalysisArgsMeta, AnalysisSource};
use tokmd_types::{ChildIncludeMode, ExportData, FileKind, FileRow};

fn sample_export() -> ExportData {
    let rows = vec![
        FileRow {
            path: "src/lib.rs".to_string(),
            module: "src".to_string(),
            lang: "Rust".to_string(),
            kind: FileKind::Parent,
            code: 100,
            comments: 20,
            blanks: 10,
            lines: 130,
            bytes: 1000,
            tokens: 250,
        },
        FileRow {
            path: "tests/lib_test.rs".to_string(),
            module: "tests".to_string(),
            lang: "Rust".to_string(),
            kind: FileKind::Parent,
            code: 50,
            comments: 10,
            blanks: 5,
            lines: 65,
            bytes: 500,
            tokens: 125,
        },
        FileRow {
            path: "Cargo.toml".to_string(),
            module: "(root)".to_string(),
            lang: "TOML".to_string(),
            kind: FileKind::Parent,
            code: 20,
            comments: 0,
            blanks: 5,
            lines: 25,
            bytes: 200,
            tokens: 50,
        },
        FileRow {
            path: "src/lib.rs".to_string(),
            module: "src".to_string(),
            lang: "Rust".to_string(),
            kind: FileKind::Child,
            code: 10,
            comments: 2,
            blanks: 1,
            lines: 13,
            bytes: 0,
            tokens: 0,
        },
    ];

    ExportData {
        rows,
        module_roots: vec!["crates".to_string(), "packages".to_string()],
        module_depth: 2,
        children: ChildIncludeMode::Separate,
    }
}

#[test]
fn derived_metrics_basic() {
    let export = sample_export();
    let ctx = AnalysisContext {
        export,
        root: std::path::PathBuf::from("."),
        source: AnalysisSource {
            inputs: vec![".".to_string()],
            export_path: None,
            base_receipt_path: None,
            export_schema_version: None,
            export_generated_at_ms: None,
            base_signature: None,
            module_roots: vec!["crates".to_string(), "packages".to_string()],
            module_depth: 2,
            children: "separate".to_string(),
        },
    };
    let request = AnalysisRequest {
        preset: AnalysisPreset::Receipt,
        args: AnalysisArgsMeta {
            preset: "receipt".to_string(),
            format: "md".to_string(),
            window_tokens: None,
            git: None,
            max_files: None,
            max_bytes: None,
            max_file_bytes: None,
            max_commits: None,
            max_commit_files: None,
            import_granularity: "module".to_string(),
        },
        limits: AnalysisLimits::default(),
        window_tokens: None,
        git: None,
        import_granularity: ImportGranularity::Module,
        detail_functions: false,
        near_dup: false,
        near_dup_threshold: 0.80,
        near_dup_max_files: 2000,
        near_dup_scope: NearDupScope::Module,
        near_dup_max_pairs: None,
        near_dup_exclude: Vec::new(),
    };

    let receipt = analyze(ctx, request).expect("analysis");
    let derived = receipt.derived.expect("derived report");

    assert_eq!(derived.totals.files, 3);
    assert_eq!(derived.totals.code, 170);
    assert_eq!(derived.totals.comments, 30);
    assert_eq!(derived.totals.blanks, 20);
    assert_eq!(derived.totals.lines, 220);

    let doc_pct = derived.doc_density.total.ratio;
    assert!((doc_pct - 0.15).abs() < 0.0001);

    let test_ratio = derived.test_density.ratio;
    assert!((test_ratio - (50.0 / 170.0)).abs() < 0.0001);

    let infra_ratio = derived.boilerplate.ratio;
    assert!((infra_ratio - (25.0 / 220.0)).abs() < 0.0001);

    assert_eq!(derived.integrity.entries, 3);

    // New assertions
    // Nesting
    assert_eq!(derived.nesting.max, 2); // src/lib.rs -> depth 2
    assert!((derived.nesting.avg - 1.67).abs() < 0.01); // (2+2+1)/3 = 1.67 
    // paths: src/lib.rs (2), tests/lib_test.rs (2), Cargo.toml (1)
    // Avg = 5/3 = 1.666...
    // Let's recheck logic: path_depth("src/lib.rs") -> 2.
    // path_depth("Cargo.toml") -> 1.
    // 2+2+1 = 5. 5/3 = 1.67.

    // Polyglot
    assert_eq!(derived.polyglot.lang_count, 2); // Rust, TOML
    assert_eq!(derived.polyglot.dominant_lang, "Rust");
    assert_eq!(derived.polyglot.dominant_lines, 150); // 100 + 50
    assert!((derived.polyglot.dominant_pct - (150.0 / 170.0)).abs() < 0.001);

    // Distribution
    assert_eq!(derived.distribution.count, 3);
    assert_eq!(derived.distribution.min, 25);
    assert_eq!(derived.distribution.max, 130);
    // sizes: 25, 65, 130. Mean = 220/3 = 73.33. Median = 65.
    assert!((derived.distribution.mean - 73.33).abs() < 0.01);
    assert!((derived.distribution.median - 65.0).abs() < 0.01);

    // Top offenders
    assert_eq!(derived.top.largest_lines.len(), 3);
    assert_eq!(derived.top.largest_lines[0].path, "src/lib.rs");
    assert_eq!(derived.top.largest_lines[2].path, "Cargo.toml");
}
