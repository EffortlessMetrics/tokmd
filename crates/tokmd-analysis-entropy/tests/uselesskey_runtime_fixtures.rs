use std::fs;
use std::path::PathBuf;

use tempfile::tempdir;
use tokmd_analysis_entropy::build_entropy_report;
use tokmd_analysis_types::EntropyClass;
use tokmd_analysis_util::AnalysisLimits;
use tokmd_types::{ChildIncludeMode, ExportData, FileKind, FileRow};
use uselesskey::{Factory, RsaFactoryExt, RsaSpec, Seed};

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

fn deterministic_factory() -> Factory {
    let seed = Seed::from_env_value(module_path!()).expect("module path should be a valid seed");
    Factory::deterministic(seed)
}

#[test]
fn uselesskey_generates_reproducible_rsa_der_fixtures() {
    let first_factory = deterministic_factory();
    let second_factory = deterministic_factory();

    let first = first_factory.rsa("entropy-fixture", RsaSpec::rs256());
    let second = second_factory.rsa("entropy-fixture", RsaSpec::rs256());

    assert_eq!(
        first.private_key_pkcs8_der(),
        second.private_key_pkcs8_der()
    );

    let different = first_factory.rsa("entropy-fixture-alt", RsaSpec::rs256());

    assert_ne!(
        first.private_key_pkcs8_der(),
        different.private_key_pkcs8_der()
    );
}

#[test]
fn entropy_report_detects_uselesskey_generated_private_key_der() {
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

    let fixture = deterministic_factory().rsa("entropy-report-fixture", RsaSpec::rs256());
    fs::write(&output_path, fixture.private_key_pkcs8_der())
        .expect("rsa fixture bytes should be written");

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
        "generated DER fixture should be strongly entropic, got {}",
        suspect.entropy_bits_per_byte
    );
    assert_eq!(suspect.class, EntropyClass::High);
}
