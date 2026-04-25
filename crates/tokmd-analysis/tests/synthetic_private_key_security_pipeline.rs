#![cfg(all(feature = "content", feature = "walk"))]

use std::path::Path;

use tokmd_analysis::{
    AnalysisContext, AnalysisLimits, AnalysisPreset, AnalysisRequest, ImportGranularity,
    NearDupScope, analyze,
};
use tokmd_analysis_types::{AnalysisArgsMeta, AnalysisSource, EntropyClass};
use tokmd_test_support::crypto;
use tokmd_types::{ChildIncludeMode, ExportData, FileKind, FileRow};

fn make_request(preset: AnalysisPreset) -> AnalysisRequest {
    AnalysisRequest {
        preset,
        args: AnalysisArgsMeta {
            preset: preset.as_str().to_string(),
            format: "json".to_string(),
            window_tokens: None,
            git: Some(false),
            max_files: None,
            max_bytes: None,
            max_file_bytes: None,
            max_commits: None,
            max_commit_files: None,
            import_granularity: "module".to_string(),
        },
        limits: AnalysisLimits::default(),
        #[cfg(feature = "effort")]
        effort: None,
        window_tokens: None,
        git: Some(false),
        import_granularity: ImportGranularity::Module,
        detail_functions: false,
        near_dup: false,
        near_dup_threshold: 0.80,
        near_dup_max_files: 1_000,
        near_dup_scope: NearDupScope::Module,
        near_dup_max_pairs: None,
        near_dup_exclude: Vec::new(),
    }
}

fn make_context(root: &Path, export: ExportData) -> AnalysisContext {
    AnalysisContext {
        export,
        root: root.to_path_buf(),
        source: AnalysisSource {
            inputs: vec![root.to_string_lossy().to_string()],
            export_path: None,
            base_receipt_path: None,
            export_schema_version: None,
            export_generated_at_ms: None,
            base_signature: None,
            module_roots: vec!["fixtures".to_string()],
            module_depth: 2,
            children: "separate".to_string(),
        },
    }
}

fn export_for_private_key_fixture(path: &str) -> ExportData {
    ExportData {
        rows: vec![FileRow {
            path: path.to_string(),
            module: "fixtures/generated".to_string(),
            lang: "Binary".to_string(),
            kind: FileKind::Parent,
            code: 1,
            comments: 0,
            blanks: 0,
            lines: 1,
            bytes: 2048,
            tokens: 2,
        }],
        module_roots: vec!["fixtures".to_string()],
        module_depth: 2,
        children: ChildIncludeMode::Separate,
    }
}

#[test]
fn security_preset_detects_synthetic_private_key_fixture() {
    let dir = tempfile::tempdir().expect("tempdir should be created");
    let relative_path = crypto::GENERATED_PRIVATE_KEY_RELATIVE_PATH;
    crypto::write_generated_private_key(dir.path(), crypto::label::SECURITY_SUSPECT)
        .expect("synthetic fixture bytes should be written");

    let receipt = analyze(
        make_context(dir.path(), export_for_private_key_fixture(relative_path)),
        make_request(AnalysisPreset::Security),
    )
    .expect("security analysis should succeed");

    let entropy = receipt
        .entropy
        .expect("security preset with content should produce entropy");
    assert_eq!(entropy.suspects.len(), 1);

    let suspect = &entropy.suspects[0];
    assert_eq!(suspect.path, relative_path);
    assert_eq!(suspect.module, "fixtures/generated");
    assert!(suspect.entropy_bits_per_byte > 7.0);
    assert_eq!(suspect.class, EntropyClass::High);
}
