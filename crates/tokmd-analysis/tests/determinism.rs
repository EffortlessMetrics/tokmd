use std::path::PathBuf;

use tokmd_analysis::{
    AnalysisContext, AnalysisLimits, AnalysisPreset, AnalysisRequest, ImportGranularity, analyze,
};
use tokmd_analysis_types::{AnalysisArgsMeta, AnalysisSource};
use tokmd_types::{ChildIncludeMode, ExportData, FileKind, FileRow};

fn simple_shuffle<T>(vec: &mut [T], seed: u64) {
    let len = vec.len();
    if len <= 1 {
        return;
    }
    let mut rng = seed;
    for i in (1..len).rev() {
        // Linear Congruential Generator
        rng = rng.wrapping_mul(6364136223846793005).wrapping_add(1);
        let j = (rng as usize) % (i + 1);
        vec.swap(i, j);
    }
}

fn create_dummy_row(i: usize) -> FileRow {
    FileRow {
        path: format!("src/file_{}.rs", i),
        module: if i % 2 == 0 {
            "src".to_string()
        } else {
            "tests".to_string()
        },
        lang: if i % 3 == 0 {
            "Rust".to_string()
        } else {
            "TOML".to_string()
        },
        kind: FileKind::Parent,
        code: (i * 10) % 100 + 1,
        comments: (i * 5) % 50,
        blanks: (i * 2) % 20,
        lines: 0, // will be calc
        bytes: (i * 100) % 1000 + 10,
        tokens: (i * 25) % 250 + 2,
    }
}

#[test]
fn test_derive_report_determinism() {
    let mut rows: Vec<FileRow> = (0..50).map(create_dummy_row).collect();
    for r in &mut rows {
        r.lines = r.code + r.comments + r.blanks;
    }

    let source = AnalysisSource {
        inputs: vec!["test".to_string()],
        export_path: None,
        base_receipt_path: None,
        export_schema_version: None,
        export_generated_at_ms: None,
        base_signature: None,
        module_roots: vec!["src".to_string()],
        module_depth: 1,
        children: "separate".to_string(),
    };

    let args_meta = AnalysisArgsMeta {
        preset: "receipt".to_string(),
        format: "json".to_string(),
        window_tokens: None,
        git: Some(false),
        max_files: None,
        max_bytes: None,
        max_commits: None,
        max_commit_files: None,
        max_file_bytes: None,
        import_granularity: "module".to_string(),
    };

    let limits = AnalysisLimits::default();

    let req = AnalysisRequest {
        preset: AnalysisPreset::Receipt,
        args: args_meta,
        limits,
        window_tokens: None,
        git: Some(false),
        import_granularity: ImportGranularity::Module,
    };

    let base_export = ExportData {
        rows: rows.clone(),
        module_roots: vec!["src".to_string()],
        module_depth: 1,
        children: ChildIncludeMode::Separate,
    };

    let ctx = AnalysisContext {
        export: base_export,
        root: PathBuf::from("."),
        source: source.clone(),
    };

    // Run base analysis
    let base_receipt = analyze(ctx, req.clone()).expect("analyze failed");
    // We only care about the derived section for this test
    let base_json = serde_json::to_string_pretty(&base_receipt.derived).unwrap();

    // Run with shuffled rows multiple times
    for seed in 1..=20 {
        let mut shuffled_rows = rows.clone();
        simple_shuffle(&mut shuffled_rows, seed);

        let shuffled_export = ExportData {
            rows: shuffled_rows,
            module_roots: vec!["src".to_string()],
            module_depth: 1,
            children: ChildIncludeMode::Separate,
        };

        let ctx = AnalysisContext {
            export: shuffled_export,
            root: PathBuf::from("."),
            source: source.clone(),
        };

        let receipt = analyze(ctx, req.clone()).expect("analyze failed");
        let json = serde_json::to_string_pretty(&receipt.derived).unwrap();

        assert_eq!(base_json, json, "Output differs for seed {}", seed);
    }
}
