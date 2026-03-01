//! Deeper tests for license radar scanning: detection, SPDX identification,
//! classification, and edge cases.

use std::fs;
use std::path::PathBuf;

use tempfile::tempdir;
use tokmd_analysis_license::build_license_report;
use tokmd_analysis_types::LicenseSourceKind;
use tokmd_analysis_util::AnalysisLimits;

fn default_limits() -> AnalysisLimits {
    AnalysisLimits::default()
}

// ── No license file scenarios ───────────────────────────────────

#[test]
fn no_files_at_all_yields_empty_report() {
    let dir = tempdir().unwrap();
    let files: Vec<PathBuf> = vec![];
    let report = build_license_report(dir.path(), &files, &default_limits()).unwrap();

    assert!(report.findings.is_empty());
    assert!(report.effective.is_none());
}

#[test]
fn only_source_files_yields_empty_report() {
    let dir = tempdir().unwrap();
    fs::write(dir.path().join("main.rs"), "fn main() {}").unwrap();
    fs::write(dir.path().join("lib.rs"), "pub fn foo() {}").unwrap();
    fs::write(dir.path().join("util.py"), "def bar(): pass").unwrap();

    let files = vec![
        PathBuf::from("main.rs"),
        PathBuf::from("lib.rs"),
        PathBuf::from("util.py"),
    ];
    let report = build_license_report(dir.path(), &files, &default_limits()).unwrap();

    assert!(report.findings.is_empty());
    assert!(report.effective.is_none());
}

// ── Multiple license files ──────────────────────────────────────

#[test]
fn multiple_license_files_all_detected() {
    let dir = tempdir().unwrap();
    fs::write(
        dir.path().join("LICENSE-MIT"),
        "Permission is hereby granted, free of charge.\n\
         The software is provided \"as is\".",
    )
    .unwrap();
    fs::write(
        dir.path().join("LICENSE-APACHE"),
        "Apache License\nVersion 2.0, January 2004\n\
         http://www.apache.org/licenses/\n\
         limitations under the license",
    )
    .unwrap();
    fs::write(
        dir.path().join("Cargo.toml"),
        "[package]\nname = \"x\"\nlicense = \"MIT OR Apache-2.0\"\n",
    )
    .unwrap();

    let files = vec![
        PathBuf::from("LICENSE-MIT"),
        PathBuf::from("LICENSE-APACHE"),
        PathBuf::from("Cargo.toml"),
    ];
    let report = build_license_report(dir.path(), &files, &default_limits()).unwrap();

    // Should find metadata + two text findings
    assert!(report.findings.len() >= 3);
    assert!(
        report
            .findings
            .iter()
            .any(|f| f.spdx == "MIT" && f.source_kind == LicenseSourceKind::Text)
    );
    assert!(
        report
            .findings
            .iter()
            .any(|f| f.spdx == "Apache-2.0" && f.source_kind == LicenseSourceKind::Text)
    );
    assert!(
        report
            .findings
            .iter()
            .any(|f| f.source_kind == LicenseSourceKind::Metadata)
    );
}

// ── Non-standard license filenames ──────────────────────────────

#[test]
fn copying_file_is_detected_as_license() {
    let dir = tempdir().unwrap();
    fs::write(
        dir.path().join("COPYING"),
        "Permission is hereby granted, free of charge.\n\
         The software is provided \"as is\".",
    )
    .unwrap();

    let files = vec![PathBuf::from("COPYING")];
    let report = build_license_report(dir.path(), &files, &default_limits()).unwrap();

    assert!(
        report.findings.iter().any(|f| f.spdx == "MIT"),
        "COPYING file should be scanned for license text"
    );
}

#[test]
fn notice_file_is_recognized_as_license_candidate() {
    let dir = tempdir().unwrap();
    fs::write(
        dir.path().join("NOTICE"),
        "Permission is hereby granted, free of charge.\n\
         The software is provided \"as is\".",
    )
    .unwrap();

    let files = vec![PathBuf::from("NOTICE")];
    let report = build_license_report(dir.path(), &files, &default_limits()).unwrap();

    assert!(
        report.findings.iter().any(|f| f.spdx == "MIT"),
        "NOTICE file should be scanned for license text"
    );
}

// ── SPDX identification for all supported licenses ──────────────

#[test]
fn gpl3_detection_from_text() {
    let dir = tempdir().unwrap();
    fs::write(
        dir.path().join("LICENSE"),
        "GNU General Public License\n\
         Version 3, 29 June 2007\n\
         any later version",
    )
    .unwrap();

    let files = vec![PathBuf::from("LICENSE")];
    let report = build_license_report(dir.path(), &files, &default_limits()).unwrap();

    assert!(report.findings.iter().any(|f| f.spdx == "GPL-3.0-or-later"));
}

#[test]
fn agpl3_detection_from_text() {
    let dir = tempdir().unwrap();
    fs::write(
        dir.path().join("LICENSE"),
        "GNU Affero General Public License\n\
         Version 3\n\
         any later version",
    )
    .unwrap();

    let files = vec![PathBuf::from("LICENSE")];
    let report = build_license_report(dir.path(), &files, &default_limits()).unwrap();

    assert!(
        report
            .findings
            .iter()
            .any(|f| f.spdx == "AGPL-3.0-or-later")
    );
}

#[test]
fn mpl2_detection_from_text() {
    let dir = tempdir().unwrap();
    fs::write(
        dir.path().join("LICENSE"),
        "Mozilla Public License Version 2.0\n\
         http://mozilla.org/MPL/2.0/",
    )
    .unwrap();

    let files = vec![PathBuf::from("LICENSE")];
    let report = build_license_report(dir.path(), &files, &default_limits()).unwrap();

    assert!(report.findings.iter().any(|f| f.spdx == "MPL-2.0"));
}

#[test]
fn bsd3_detection_from_text() {
    let dir = tempdir().unwrap();
    fs::write(
        dir.path().join("LICENSE"),
        "Redistribution and use in source and binary forms\n\
         Neither the name of the copyright holder\n\
         contributors may be used to endorse",
    )
    .unwrap();

    let files = vec![PathBuf::from("LICENSE")];
    let report = build_license_report(dir.path(), &files, &default_limits()).unwrap();

    assert!(report.findings.iter().any(|f| f.spdx == "BSD-3-Clause"));
}

// ── License classification (effective) ──────────────────────────

#[test]
fn effective_license_is_highest_confidence_finding() {
    let dir = tempdir().unwrap();
    // Metadata finding has 0.95 confidence, text finding usually lower
    fs::write(
        dir.path().join("Cargo.toml"),
        "[package]\nname = \"x\"\nlicense = \"MIT\"\n",
    )
    .unwrap();
    fs::write(
        dir.path().join("LICENSE"),
        "Permission is hereby granted, free of charge.",
    )
    .unwrap();

    let files = vec![PathBuf::from("Cargo.toml"), PathBuf::from("LICENSE")];
    let report = build_license_report(dir.path(), &files, &default_limits()).unwrap();

    assert_eq!(
        report.effective,
        report.findings.first().map(|f| f.spdx.clone()),
        "effective license must be the first (highest confidence) finding"
    );
}

#[test]
fn findings_sorted_deterministically() {
    let dir = tempdir().unwrap();
    fs::write(
        dir.path().join("Cargo.toml"),
        "[package]\nname = \"x\"\nlicense = \"Apache-2.0\"\n",
    )
    .unwrap();
    fs::write(
        dir.path().join("LICENSE"),
        "Permission is hereby granted, free of charge.\n\
         The software is provided \"as is\".",
    )
    .unwrap();

    let files = vec![PathBuf::from("Cargo.toml"), PathBuf::from("LICENSE")];

    // Run twice; results must be identical
    let r1 = build_license_report(dir.path(), &files, &default_limits()).unwrap();
    let r2 = build_license_report(dir.path(), &files, &default_limits()).unwrap();

    assert_eq!(r1.findings.len(), r2.findings.len());
    for (a, b) in r1.findings.iter().zip(r2.findings.iter()) {
        assert_eq!(a.spdx, b.spdx);
        assert_eq!(a.source_path, b.source_path);
        assert_eq!(a.source_kind, b.source_kind);
        assert!((a.confidence - b.confidence).abs() < f32::EPSILON);
    }
}

// ── Edge cases ──────────────────────────────────────────────────

#[test]
fn empty_license_file_yields_no_findings() {
    let dir = tempdir().unwrap();
    fs::write(dir.path().join("LICENSE"), "").unwrap();

    let files = vec![PathBuf::from("LICENSE")];
    let report = build_license_report(dir.path(), &files, &default_limits()).unwrap();

    assert!(report.findings.is_empty());
    assert!(report.effective.is_none());
}

#[test]
fn proprietary_text_yields_no_findings() {
    let dir = tempdir().unwrap();
    fs::write(
        dir.path().join("LICENSE"),
        "All rights reserved. Proprietary and confidential.\n\
         No part of this software may be reproduced.",
    )
    .unwrap();

    let files = vec![PathBuf::from("LICENSE")];
    let report = build_license_report(dir.path(), &files, &default_limits()).unwrap();

    assert!(report.findings.is_empty());
}

#[test]
fn package_json_without_license_field_yields_nothing() {
    let dir = tempdir().unwrap();
    fs::write(
        dir.path().join("package.json"),
        r#"{"name": "test", "version": "1.0.0", "main": "index.js"}"#,
    )
    .unwrap();

    let files = vec![PathBuf::from("package.json")];
    let report = build_license_report(dir.path(), &files, &default_limits()).unwrap();

    assert!(report.findings.is_empty());
}

#[test]
fn package_json_with_object_license() {
    let dir = tempdir().unwrap();
    fs::write(
        dir.path().join("package.json"),
        r#"{"name": "x", "license": {"type": "MIT", "url": "https://example.com"}}"#,
    )
    .unwrap();

    let files = vec![PathBuf::from("package.json")];
    let report = build_license_report(dir.path(), &files, &default_limits()).unwrap();

    assert_eq!(report.findings.len(), 1);
    assert_eq!(report.findings[0].spdx, "MIT");
    assert_eq!(report.findings[0].source_kind, LicenseSourceKind::Metadata);
}

#[test]
fn pyproject_toml_poetry_fallback() {
    let dir = tempdir().unwrap();
    fs::write(
        dir.path().join("pyproject.toml"),
        "[tool.poetry]\nname = \"x\"\nlicense = \"BSD-3-Clause\"\n",
    )
    .unwrap();

    let files = vec![PathBuf::from("pyproject.toml")];
    let report = build_license_report(dir.path(), &files, &default_limits()).unwrap();

    assert_eq!(report.findings.len(), 1);
    assert_eq!(report.findings[0].spdx, "BSD-3-Clause");
}

#[test]
fn cargo_toml_license_file_field_triggers_text_scan() {
    let dir = tempdir().unwrap();
    fs::write(
        dir.path().join("Cargo.toml"),
        "[package]\nname = \"x\"\nlicense-file = \"MY-LICENSE.txt\"\n",
    )
    .unwrap();
    fs::write(
        dir.path().join("MY-LICENSE.txt"),
        "Permission is hereby granted, free of charge.\n\
         The software is provided \"as is\".",
    )
    .unwrap();

    let files = vec![PathBuf::from("Cargo.toml"), PathBuf::from("MY-LICENSE.txt")];
    let report = build_license_report(dir.path(), &files, &default_limits()).unwrap();

    assert!(report.findings.iter().any(|f| f.spdx == "MIT"
        && f.source_kind == LicenseSourceKind::Text
        && f.source_path == "MY-LICENSE.txt"));
}

// ── Path normalization ──────────────────────────────────────────

#[test]
fn source_paths_are_forward_slash_normalized() {
    let dir = tempdir().unwrap();
    let sub = dir.path().join("sub");
    fs::create_dir_all(&sub).unwrap();
    fs::write(
        sub.join("Cargo.toml"),
        "[package]\nname = \"x\"\nlicense = \"MIT\"\n",
    )
    .unwrap();

    let files = vec![PathBuf::from("sub").join("Cargo.toml")];
    let report = build_license_report(dir.path(), &files, &default_limits()).unwrap();

    for f in &report.findings {
        assert!(
            !f.source_path.contains('\\'),
            "path should use forward slashes: {}",
            f.source_path
        );
    }
}

// ── Confidence range invariant ──────────────────────────────────

#[test]
fn all_confidences_within_zero_to_one() {
    let dir = tempdir().unwrap();
    fs::write(
        dir.path().join("Cargo.toml"),
        "[package]\nname = \"x\"\nlicense = \"MIT\"\n",
    )
    .unwrap();
    fs::write(
        dir.path().join("LICENSE"),
        "Permission is hereby granted, free of charge.\n\
         The software is provided \"as is\".",
    )
    .unwrap();

    let files = vec![PathBuf::from("Cargo.toml"), PathBuf::from("LICENSE")];
    let report = build_license_report(dir.path(), &files, &default_limits()).unwrap();

    for f in &report.findings {
        assert!(
            (0.0..=1.0).contains(&f.confidence),
            "confidence {} out of [0.0, 1.0] range",
            f.confidence
        );
    }
}

// ── Full vs partial text confidence ─────────────────────────────

#[test]
fn full_mit_text_yields_higher_confidence_than_partial() {
    let dir_full = tempdir().unwrap();
    fs::write(
        dir_full.path().join("LICENSE"),
        "Permission is hereby granted, free of charge, to any person.\n\
         THE SOFTWARE IS PROVIDED \"AS IS\", WITHOUT WARRANTY.",
    )
    .unwrap();

    let dir_partial = tempdir().unwrap();
    fs::write(
        dir_partial.path().join("LICENSE"),
        "Permission is hereby granted, free of charge, to any person.",
    )
    .unwrap();

    let files = vec![PathBuf::from("LICENSE")];

    let full = build_license_report(dir_full.path(), &files, &default_limits()).unwrap();
    let partial = build_license_report(dir_partial.path(), &files, &default_limits()).unwrap();

    assert!(!full.findings.is_empty());
    assert!(!partial.findings.is_empty());
    assert!(
        full.findings[0].confidence >= partial.findings[0].confidence,
        "full text confidence ({}) should be >= partial ({})",
        full.findings[0].confidence,
        partial.findings[0].confidence
    );
}
