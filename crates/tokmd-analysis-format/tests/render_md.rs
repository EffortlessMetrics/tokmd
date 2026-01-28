use tokmd_analysis::{
    AnalysisContext, AnalysisLimits, AnalysisPreset, AnalysisRequest, ImportGranularity, analyze,
};
use tokmd_analysis_format::render;
use tokmd_analysis_types::{AnalysisArgsMeta, AnalysisSource};
use tokmd_config::AnalysisFormat;
use tokmd_types::{ExportData, FileKind, FileRow};

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
    ];

    ExportData {
        rows,
        module_roots: vec!["crates".to_string(), "packages".to_string()],
        module_depth: 2,
        children: tokmd_config::ChildIncludeMode::Separate,
    }
}

#[test]
fn render_md_snapshot() {
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
    };

    let receipt = analyze(ctx, request).expect("analysis");
    let rendered = render(&receipt, AnalysisFormat::Md).expect("render");
    let text = match rendered {
        tokmd_analysis_format::RenderedOutput::Text(s) => s,
        tokmd_analysis_format::RenderedOutput::Binary(_) => panic!("expected text"),
    };

    insta::assert_snapshot!(text);
}
