use std::fs;
use std::path::PathBuf;

use tempfile::tempdir;
use tokmd_analysis_entropy::build_entropy_report;
use tokmd_analysis_types::EntropyClass;
use tokmd_analysis_util::AnalysisLimits;
use tokmd_types::{ChildIncludeMode, ExportData, FileKind, FileRow};

fn export_for_paths(paths: &[&str]) -> ExportData {
    let rows = paths
        .iter()
        .map(|path| FileRow {
            path: (*path).to_string(),
            module: path
                .rsplit_once('/')
                .map(|(module, _)| module)
                .unwrap_or("(root)")
                .to_string(),
            lang: "Binary".to_string(),
            kind: FileKind::Parent,
            code: 1,
            comments: 0,
            blanks: 0,
            lines: 1,
            bytes: 10,
            tokens: 2,
        })
        .collect();

    ExportData {
        rows,
        module_roots: vec![],
        module_depth: 2,
        children: ChildIncludeMode::Separate,
    }
}

// Generates pseudo-random bytes to simulate a high-entropy file (e.g. RSA DER private key).
fn generate_static_high_entropy_fixture(len: usize) -> Vec<u8> {
    let mut data = Vec::with_capacity(len);
    let mut state: u32 = 0x12345678;
    for _ in 0..len {
        state ^= state << 13;
        state ^= state >> 17;
        state ^= state << 5;
        data.push((state & 0xFF) as u8);
    }
    data
}

#[test]
fn entropy_report_detects_static_high_entropy_blob() {
    let dir = tempdir().expect("tempdir should be created");
    let relative_path = "fixtures/generated/private-key.pk8";
    let output_path = dir
        .path()
        .join("fixtures")
        .join("generated")
        .join("private-key.pk8");
    fs::create_dir_all(
        output_path
            .parent()
            .expect("generated fixture file should have a parent directory"),
    )
    .expect("fixture directory should be created");

    // Replace dynamic uselesskey generation with a static blob
    let fixture_bytes = generate_static_high_entropy_fixture(2048);
    fs::write(&output_path, fixture_bytes).expect("static fixture bytes should be written");

    let export = export_for_paths(&[relative_path]);
    let files = vec![PathBuf::from(relative_path)];
    let report = build_entropy_report(dir.path(), &files, &export, &AnalysisLimits::default())
        .expect("entropy report should be built");

    assert_eq!(report.suspects.len(), 1);

    let suspect = &report.suspects[0];
    assert_eq!(suspect.path, relative_path);
    assert_eq!(suspect.module, "fixtures/generated");
    assert!(
        suspect.entropy_bits_per_byte > 7.0,
        "generated fixture should be strongly entropic, got {}",
        suspect.entropy_bits_per_byte
    );
    assert_eq!(suspect.class, EntropyClass::High);
}
