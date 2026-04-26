//! Deep tests for analysis license module (w68).

use crate::license::build_license_report;
use std::fs;
use std::path::PathBuf;
use tempfile::tempdir;
use tokmd_analysis_types::AnalysisLimits;
use tokmd_analysis_types::LicenseSourceKind;

fn default_limits() -> AnalysisLimits {
    AnalysisLimits::default()
}

// ---------------------------------------------------------------------------
// Metadata detection – Cargo.toml
// ---------------------------------------------------------------------------

#[test]
fn cargo_toml_mit_license() {
    let dir = tempdir().unwrap();
    fs::write(
        dir.path().join("Cargo.toml"),
        "[package]\nname = \"x\"\nlicense = \"MIT\"\n",
    )
    .unwrap();
    let report = build_license_report(
        dir.path(),
        &[PathBuf::from("Cargo.toml")],
        &default_limits(),
    )
    .unwrap();
    assert_eq!(report.findings.len(), 1);
    assert_eq!(report.findings[0].spdx, "MIT");
    assert_eq!(report.findings[0].source_kind, LicenseSourceKind::Metadata);
    assert_eq!(report.effective, Some("MIT".to_string()));
}

#[test]
fn cargo_toml_apache2_license() {
    let dir = tempdir().unwrap();
    fs::write(
        dir.path().join("Cargo.toml"),
        "[package]\nname = \"y\"\nlicense = \"Apache-2.0\"\n",
    )
    .unwrap();
    let report = build_license_report(
        dir.path(),
        &[PathBuf::from("Cargo.toml")],
        &default_limits(),
    )
    .unwrap();
    assert!(report.findings.iter().any(|f| f.spdx == "Apache-2.0"));
}

#[test]
fn cargo_toml_dual_license_expression() {
    let dir = tempdir().unwrap();
    fs::write(
        dir.path().join("Cargo.toml"),
        "[package]\nname = \"z\"\nlicense = \"MIT OR Apache-2.0\"\n",
    )
    .unwrap();
    let report = build_license_report(
        dir.path(),
        &[PathBuf::from("Cargo.toml")],
        &default_limits(),
    )
    .unwrap();
    assert_eq!(report.findings.len(), 1);
    assert_eq!(report.findings[0].spdx, "MIT OR Apache-2.0");
}

#[test]
fn cargo_toml_single_quoted_license() {
    let dir = tempdir().unwrap();
    fs::write(
        dir.path().join("Cargo.toml"),
        "[package]\nname = \"q\"\nlicense = 'BSD-3-Clause'\n",
    )
    .unwrap();
    let report = build_license_report(
        dir.path(),
        &[PathBuf::from("Cargo.toml")],
        &default_limits(),
    )
    .unwrap();
    assert_eq!(report.findings[0].spdx, "BSD-3-Clause");
}

// ---------------------------------------------------------------------------
// Metadata detection – package.json
// ---------------------------------------------------------------------------

#[test]
fn package_json_string_license() {
    let dir = tempdir().unwrap();
    fs::write(
        dir.path().join("package.json"),
        r#"{"name":"x","license":"ISC"}"#,
    )
    .unwrap();
    let report = build_license_report(
        dir.path(),
        &[PathBuf::from("package.json")],
        &default_limits(),
    )
    .unwrap();
    assert_eq!(report.findings.len(), 1);
    assert_eq!(report.findings[0].spdx, "ISC");
    assert_eq!(report.findings[0].source_kind, LicenseSourceKind::Metadata);
}

#[test]
fn package_json_object_license() {
    let dir = tempdir().unwrap();
    fs::write(
        dir.path().join("package.json"),
        r#"{"name":"x","license":{"type":"MIT","url":"https://mit.example"}}"#,
    )
    .unwrap();
    let report = build_license_report(
        dir.path(),
        &[PathBuf::from("package.json")],
        &default_limits(),
    )
    .unwrap();
    assert_eq!(report.findings[0].spdx, "MIT");
}

// ---------------------------------------------------------------------------
// Metadata detection – pyproject.toml
// ---------------------------------------------------------------------------

#[test]
fn pyproject_toml_project_section() {
    let dir = tempdir().unwrap();
    fs::write(
        dir.path().join("pyproject.toml"),
        "[project]\nname = \"pkg\"\nlicense = \"GPL-3.0-or-later\"\n",
    )
    .unwrap();
    let report = build_license_report(
        dir.path(),
        &[PathBuf::from("pyproject.toml")],
        &default_limits(),
    )
    .unwrap();
    assert_eq!(report.findings[0].spdx, "GPL-3.0-or-later");
}

#[test]
fn pyproject_toml_poetry_section() {
    let dir = tempdir().unwrap();
    fs::write(
        dir.path().join("pyproject.toml"),
        "[tool.poetry]\nname = \"pkg\"\nlicense = \"MIT\"\n",
    )
    .unwrap();
    let report = build_license_report(
        dir.path(),
        &[PathBuf::from("pyproject.toml")],
        &default_limits(),
    )
    .unwrap();
    assert_eq!(report.findings[0].spdx, "MIT");
}

// ---------------------------------------------------------------------------
// Text-based license detection
// ---------------------------------------------------------------------------

#[test]
fn license_text_mit() {
    let dir = tempdir().unwrap();
    fs::write(
        dir.path().join("LICENSE"),
        "Permission is hereby granted, free of charge, to any person obtaining \
         a copy of this software. THE SOFTWARE IS PROVIDED \"AS IS\".",
    )
    .unwrap();
    let report =
        build_license_report(dir.path(), &[PathBuf::from("LICENSE")], &default_limits()).unwrap();
    let text_finding = report
        .findings
        .iter()
        .find(|f| f.source_kind == LicenseSourceKind::Text)
        .unwrap();
    assert_eq!(text_finding.spdx, "MIT");
    assert!(text_finding.confidence >= 0.6);
}

#[test]
fn license_text_apache2() {
    let dir = tempdir().unwrap();
    fs::write(
        dir.path().join("LICENSE"),
        "Apache License\nVersion 2.0\nhttp://www.apache.org/licenses/\n\
         limitations under the license",
    )
    .unwrap();
    let report =
        build_license_report(dir.path(), &[PathBuf::from("LICENSE")], &default_limits()).unwrap();
    assert!(report.findings.iter().any(|f| f.spdx == "Apache-2.0"));
}

#[test]
fn license_text_gpl3() {
    let dir = tempdir().unwrap();
    fs::write(
        dir.path().join("LICENSE"),
        "GNU General Public License\nVersion 3\nany later version",
    )
    .unwrap();
    let report =
        build_license_report(dir.path(), &[PathBuf::from("LICENSE")], &default_limits()).unwrap();
    assert!(report.findings.iter().any(|f| f.spdx == "GPL-3.0-or-later"));
}

#[test]
fn license_text_bsd3() {
    let dir = tempdir().unwrap();
    fs::write(
        dir.path().join("LICENSE"),
        "Redistribution and use in source and binary forms, with or without modification.\n\
         Neither the name of\n\
         contributors may be used",
    )
    .unwrap();
    let report =
        build_license_report(dir.path(), &[PathBuf::from("LICENSE")], &default_limits()).unwrap();
    assert!(report.findings.iter().any(|f| f.spdx == "BSD-3-Clause"));
}

#[test]
fn license_text_mpl2() {
    let dir = tempdir().unwrap();
    fs::write(
        dir.path().join("LICENSE"),
        "Mozilla Public License\nVersion 2.0\nhttp://mozilla.org/MPL/2.0/",
    )
    .unwrap();
    let report =
        build_license_report(dir.path(), &[PathBuf::from("LICENSE")], &default_limits()).unwrap();
    assert!(report.findings.iter().any(|f| f.spdx == "MPL-2.0"));
}

// ---------------------------------------------------------------------------
// Empty / missing / edge cases
// ---------------------------------------------------------------------------

#[test]
fn empty_license_file_no_findings() {
    let dir = tempdir().unwrap();
    fs::write(dir.path().join("LICENSE"), "").unwrap();
    let report =
        build_license_report(dir.path(), &[PathBuf::from("LICENSE")], &default_limits()).unwrap();
    assert!(report.findings.is_empty());
    assert_eq!(report.effective, None);
}

#[test]
fn no_files_yields_empty_report() {
    let dir = tempdir().unwrap();
    let report = build_license_report(dir.path(), &[], &default_limits()).unwrap();
    assert!(report.findings.is_empty());
    assert_eq!(report.effective, None);
}

#[test]
fn unrecognized_text_no_finding() {
    let dir = tempdir().unwrap();
    fs::write(
        dir.path().join("LICENSE"),
        "This is a custom proprietary license with no matching phrases.",
    )
    .unwrap();
    let report =
        build_license_report(dir.path(), &[PathBuf::from("LICENSE")], &default_limits()).unwrap();
    assert!(
        report
            .findings
            .iter()
            .all(|f| f.source_kind != LicenseSourceKind::Text)
    );
}

// ---------------------------------------------------------------------------
// Sorting / determinism
// ---------------------------------------------------------------------------

#[test]
fn findings_sorted_by_confidence_desc() {
    let dir = tempdir().unwrap();
    // Metadata findings have 0.95 confidence; text findings vary.
    fs::write(
        dir.path().join("Cargo.toml"),
        "[package]\nname = \"x\"\nlicense = \"MIT\"\n",
    )
    .unwrap();
    fs::write(
        dir.path().join("LICENSE"),
        "Permission is hereby granted, free of charge, to any person. \
         THE SOFTWARE IS PROVIDED \"AS IS\".",
    )
    .unwrap();
    let files = vec![PathBuf::from("Cargo.toml"), PathBuf::from("LICENSE")];
    let report = build_license_report(dir.path(), &files, &default_limits()).unwrap();
    assert!(report.findings.len() >= 2);
    for w in report.findings.windows(2) {
        assert!(w[0].confidence >= w[1].confidence);
    }
}

#[test]
fn effective_is_highest_confidence() {
    let dir = tempdir().unwrap();
    fs::write(
        dir.path().join("Cargo.toml"),
        "[package]\nname = \"x\"\nlicense = \"MIT\"\n",
    )
    .unwrap();
    let report = build_license_report(
        dir.path(),
        &[PathBuf::from("Cargo.toml")],
        &default_limits(),
    )
    .unwrap();
    assert_eq!(report.effective, Some("MIT".to_string()));
}

#[test]
fn deterministic_across_runs() {
    let dir = tempdir().unwrap();
    fs::write(
        dir.path().join("Cargo.toml"),
        "[package]\nname = \"x\"\nlicense = \"MIT\"\n",
    )
    .unwrap();
    fs::write(
        dir.path().join("LICENSE"),
        "Permission is hereby granted, free of charge. \
         The software is provided \"as is\".",
    )
    .unwrap();
    let files = vec![PathBuf::from("Cargo.toml"), PathBuf::from("LICENSE")];
    let r1 = build_license_report(dir.path(), &files, &default_limits()).unwrap();
    let r2 = build_license_report(dir.path(), &files, &default_limits()).unwrap();
    assert_eq!(r1.findings.len(), r2.findings.len());
    for (a, b) in r1.findings.iter().zip(&r2.findings) {
        assert_eq!(a.spdx, b.spdx);
        assert_eq!(a.confidence, b.confidence);
        assert_eq!(a.source_path, b.source_path);
    }
}

// ---------------------------------------------------------------------------
// Confidence scoring
// ---------------------------------------------------------------------------

#[test]
fn confidence_increases_with_more_phrase_hits() {
    let dir = tempdir().unwrap();
    // One-phrase MIT hit
    fs::write(
        dir.path().join("LICENSE-A"),
        "Permission is hereby granted, free of charge.",
    )
    .unwrap();
    // Two-phrase MIT hit (higher confidence)
    fs::write(
        dir.path().join("LICENSE-B"),
        "Permission is hereby granted, free of charge. The software is provided \"as is\".",
    )
    .unwrap();
    let r_a =
        build_license_report(dir.path(), &[PathBuf::from("LICENSE-A")], &default_limits()).unwrap();
    let r_b =
        build_license_report(dir.path(), &[PathBuf::from("LICENSE-B")], &default_limits()).unwrap();
    let conf_a = r_a
        .findings
        .iter()
        .find(|f| f.spdx == "MIT")
        .map(|f| f.confidence)
        .unwrap_or(0.0);
    let conf_b = r_b
        .findings
        .iter()
        .find(|f| f.spdx == "MIT")
        .map(|f| f.confidence)
        .unwrap_or(0.0);
    assert!(conf_b >= conf_a);
}
