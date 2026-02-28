use std::fs;
use std::path::PathBuf;
use tempfile::tempdir;
use tokmd_analysis_license::build_license_report;
use tokmd_analysis_types::LicenseSourceKind;
use tokmd_analysis_util::AnalysisLimits;

fn default_limits() -> AnalysisLimits {
    AnalysisLimits::default()
}

// ---------------------------------------------------------------------------
// Scenario: Detect MIT license from Cargo.toml metadata
// ---------------------------------------------------------------------------

#[test]
fn given_cargo_toml_with_mit_then_finds_mit_metadata() {
    let dir = tempdir().unwrap();
    fs::write(
        dir.path().join("Cargo.toml"),
        "[package]\nname = \"x\"\nlicense = \"MIT\"\n",
    )
    .unwrap();

    let files = vec![PathBuf::from("Cargo.toml")];
    let report = build_license_report(dir.path(), &files, &default_limits()).unwrap();

    assert_eq!(report.findings.len(), 1);
    assert_eq!(report.findings[0].spdx, "MIT");
    assert_eq!(report.findings[0].source_kind, LicenseSourceKind::Metadata);
    assert_eq!(report.effective.as_deref(), Some("MIT"));
}

// ---------------------------------------------------------------------------
// Scenario: Detect Apache-2.0 from Cargo.toml metadata
// ---------------------------------------------------------------------------

#[test]
fn given_cargo_toml_with_apache_then_finds_apache_metadata() {
    let dir = tempdir().unwrap();
    fs::write(
        dir.path().join("Cargo.toml"),
        "[package]\nname = \"x\"\nlicense = \"Apache-2.0\"\n",
    )
    .unwrap();

    let files = vec![PathBuf::from("Cargo.toml")];
    let report = build_license_report(dir.path(), &files, &default_limits()).unwrap();

    assert_eq!(report.findings.len(), 1);
    assert_eq!(report.findings[0].spdx, "Apache-2.0");
    assert_eq!(report.effective.as_deref(), Some("Apache-2.0"));
}

// ---------------------------------------------------------------------------
// Scenario: Detect dual license expression from metadata
// ---------------------------------------------------------------------------

#[test]
fn given_dual_license_expression_then_finds_expression() {
    let dir = tempdir().unwrap();
    fs::write(
        dir.path().join("Cargo.toml"),
        "[package]\nname = \"x\"\nlicense = \"MIT OR Apache-2.0\"\n",
    )
    .unwrap();

    let files = vec![PathBuf::from("Cargo.toml")];
    let report = build_license_report(dir.path(), &files, &default_limits()).unwrap();

    assert_eq!(report.findings.len(), 1);
    assert_eq!(report.findings[0].spdx, "MIT OR Apache-2.0");
}

// ---------------------------------------------------------------------------
// Scenario: Detect MIT from LICENSE text file
// ---------------------------------------------------------------------------

#[test]
fn given_mit_license_text_then_finds_mit_text() {
    let dir = tempdir().unwrap();
    fs::write(
        dir.path().join("LICENSE"),
        "Permission is hereby granted, free of charge, to any person obtaining a copy.\n\
         The software is provided \"as is\", without warranty of any kind.",
    )
    .unwrap();

    let files = vec![PathBuf::from("LICENSE")];
    let report = build_license_report(dir.path(), &files, &default_limits()).unwrap();

    assert_eq!(report.findings.len(), 1);
    assert_eq!(report.findings[0].spdx, "MIT");
    assert_eq!(report.findings[0].source_kind, LicenseSourceKind::Text);
    assert!(report.findings[0].confidence >= 0.6);
    assert_eq!(report.effective.as_deref(), Some("MIT"));
}

// ---------------------------------------------------------------------------
// Scenario: Detect Apache-2.0 from LICENSE text file
// ---------------------------------------------------------------------------

#[test]
fn given_apache_license_text_then_finds_apache() {
    let dir = tempdir().unwrap();
    fs::write(
        dir.path().join("LICENSE"),
        "Apache License\nVersion 2.0, January 2004\n\
         http://www.apache.org/licenses/\n\
         Subject to the terms and limitations under the License.",
    )
    .unwrap();

    let files = vec![PathBuf::from("LICENSE")];
    let report = build_license_report(dir.path(), &files, &default_limits()).unwrap();

    assert!(report.findings.iter().any(|f| f.spdx == "Apache-2.0"));
    assert_eq!(report.effective.as_deref(), Some("Apache-2.0"));
}

// ---------------------------------------------------------------------------
// Scenario: Detect GPL-3.0-or-later from LICENSE text
// ---------------------------------------------------------------------------

#[test]
fn given_gpl3_license_text_then_finds_gpl() {
    let dir = tempdir().unwrap();
    fs::write(
        dir.path().join("LICENSE"),
        "GNU General Public License\nVersion 3, 29 June 2007\n\
         You can redistribute it and/or modify it under the terms of the GNU \
         General Public License as published by the Free Software Foundation, \
         either version 3 of the License, or (at your option) any later version.",
    )
    .unwrap();

    let files = vec![PathBuf::from("LICENSE")];
    let report = build_license_report(dir.path(), &files, &default_limits()).unwrap();

    assert!(report.findings.iter().any(|f| f.spdx == "GPL-3.0-or-later"));
}

// ---------------------------------------------------------------------------
// Scenario: Detect BSD-3-Clause from text
// ---------------------------------------------------------------------------

#[test]
fn given_bsd3_license_text_then_finds_bsd3() {
    let dir = tempdir().unwrap();
    fs::write(
        dir.path().join("LICENSE"),
        "Redistribution and use in source and binary forms, with or without modification.\n\
         Neither the name of the copyright holder nor the names of its contributors \
         may be used to endorse or promote products.",
    )
    .unwrap();

    let files = vec![PathBuf::from("LICENSE")];
    let report = build_license_report(dir.path(), &files, &default_limits()).unwrap();

    assert!(report.findings.iter().any(|f| f.spdx == "BSD-3-Clause"));
}

// ---------------------------------------------------------------------------
// Scenario: Detect MPL-2.0 from text
// ---------------------------------------------------------------------------

#[test]
fn given_mpl2_license_text_then_finds_mpl2() {
    let dir = tempdir().unwrap();
    fs::write(
        dir.path().join("LICENSE"),
        "Mozilla Public License Version 2.0\n\
         http://mozilla.org/MPL/2.0/\n\
         This Source Code Form is subject to the terms of the Mozilla Public License.",
    )
    .unwrap();

    let files = vec![PathBuf::from("LICENSE")];
    let report = build_license_report(dir.path(), &files, &default_limits()).unwrap();

    assert!(report.findings.iter().any(|f| f.spdx == "MPL-2.0"));
}

// ---------------------------------------------------------------------------
// Scenario: No license files → empty report
// ---------------------------------------------------------------------------

#[test]
fn given_no_license_files_then_empty_report() {
    let dir = tempdir().unwrap();
    // Create a random source file that is not a license indicator
    fs::write(dir.path().join("main.rs"), "fn main() {}").unwrap();

    let files = vec![PathBuf::from("main.rs")];
    let report = build_license_report(dir.path(), &files, &default_limits()).unwrap();

    assert!(report.findings.is_empty());
    assert!(report.effective.is_none());
}

// ---------------------------------------------------------------------------
// Scenario: Empty LICENSE file → no findings
// ---------------------------------------------------------------------------

#[test]
fn given_empty_license_file_then_no_findings() {
    let dir = tempdir().unwrap();
    fs::write(dir.path().join("LICENSE"), "").unwrap();

    let files = vec![PathBuf::from("LICENSE")];
    let report = build_license_report(dir.path(), &files, &default_limits()).unwrap();

    assert!(report.findings.is_empty());
    assert!(report.effective.is_none());
}

// ---------------------------------------------------------------------------
// Scenario: package.json string license field
// ---------------------------------------------------------------------------

#[test]
fn given_package_json_string_license_then_finds_metadata() {
    let dir = tempdir().unwrap();
    fs::write(
        dir.path().join("package.json"),
        r#"{"name": "x", "license": "ISC"}"#,
    )
    .unwrap();

    let files = vec![PathBuf::from("package.json")];
    let report = build_license_report(dir.path(), &files, &default_limits()).unwrap();

    assert_eq!(report.findings.len(), 1);
    assert_eq!(report.findings[0].spdx, "ISC");
    assert_eq!(report.findings[0].source_kind, LicenseSourceKind::Metadata);
}

// ---------------------------------------------------------------------------
// Scenario: package.json object license field
// ---------------------------------------------------------------------------

#[test]
fn given_package_json_object_license_then_finds_metadata() {
    let dir = tempdir().unwrap();
    fs::write(
        dir.path().join("package.json"),
        r#"{"name": "x", "license": {"type": "BSD-2-Clause", "url": "https://example.com"}}"#,
    )
    .unwrap();

    let files = vec![PathBuf::from("package.json")];
    let report = build_license_report(dir.path(), &files, &default_limits()).unwrap();

    assert_eq!(report.findings.len(), 1);
    assert_eq!(report.findings[0].spdx, "BSD-2-Clause");
}

// ---------------------------------------------------------------------------
// Scenario: pyproject.toml [project] section
// ---------------------------------------------------------------------------

#[test]
fn given_pyproject_toml_project_section_then_finds_metadata() {
    let dir = tempdir().unwrap();
    fs::write(
        dir.path().join("pyproject.toml"),
        "[project]\nname = \"x\"\nlicense = \"MIT\"\n",
    )
    .unwrap();

    let files = vec![PathBuf::from("pyproject.toml")];
    let report = build_license_report(dir.path(), &files, &default_limits()).unwrap();

    assert_eq!(report.findings.len(), 1);
    assert_eq!(report.findings[0].spdx, "MIT");
}

// ---------------------------------------------------------------------------
// Scenario: pyproject.toml [tool.poetry] fallback
// ---------------------------------------------------------------------------

#[test]
fn given_pyproject_toml_poetry_section_then_finds_metadata() {
    let dir = tempdir().unwrap();
    fs::write(
        dir.path().join("pyproject.toml"),
        "[tool.poetry]\nname = \"x\"\nlicense = \"GPL-3.0\"\n",
    )
    .unwrap();

    let files = vec![PathBuf::from("pyproject.toml")];
    let report = build_license_report(dir.path(), &files, &default_limits()).unwrap();

    assert_eq!(report.findings.len(), 1);
    assert_eq!(report.findings[0].spdx, "GPL-3.0");
}

// ---------------------------------------------------------------------------
// Scenario: Multiple sources — metadata + text file
// ---------------------------------------------------------------------------

#[test]
fn given_metadata_and_license_text_then_finds_both() {
    let dir = tempdir().unwrap();
    fs::write(
        dir.path().join("Cargo.toml"),
        "[package]\nname = \"x\"\nlicense = \"MIT\"\n",
    )
    .unwrap();
    fs::write(
        dir.path().join("LICENSE"),
        "Permission is hereby granted, free of charge, to any person obtaining a copy.\n\
         The software is provided \"as is\".",
    )
    .unwrap();

    let files = vec![PathBuf::from("Cargo.toml"), PathBuf::from("LICENSE")];
    let report = build_license_report(dir.path(), &files, &default_limits()).unwrap();

    assert!(report.findings.len() >= 2);
    assert!(
        report
            .findings
            .iter()
            .any(|f| f.source_kind == LicenseSourceKind::Metadata)
    );
    assert!(
        report
            .findings
            .iter()
            .any(|f| f.source_kind == LicenseSourceKind::Text)
    );
}

// ---------------------------------------------------------------------------
// Scenario: Findings are sorted by confidence descending
// ---------------------------------------------------------------------------

#[test]
fn findings_are_sorted_by_confidence_descending() {
    let dir = tempdir().unwrap();
    // Metadata has 0.95 confidence, text usually lower
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

    for pair in report.findings.windows(2) {
        assert!(
            pair[0].confidence >= pair[1].confidence,
            "findings not sorted: {} < {}",
            pair[0].confidence,
            pair[1].confidence
        );
    }
}

// ---------------------------------------------------------------------------
// Scenario: Effective license is the highest-confidence finding
// ---------------------------------------------------------------------------

#[test]
fn effective_is_highest_confidence() {
    let dir = tempdir().unwrap();
    fs::write(
        dir.path().join("Cargo.toml"),
        "[package]\nname = \"x\"\nlicense = \"Apache-2.0\"\n",
    )
    .unwrap();

    let files = vec![PathBuf::from("Cargo.toml")];
    let report = build_license_report(dir.path(), &files, &default_limits()).unwrap();

    assert_eq!(
        report.effective,
        report.findings.first().map(|f| f.spdx.clone())
    );
}

// ---------------------------------------------------------------------------
// Scenario: LICENSE-MIT variant filename
// ---------------------------------------------------------------------------

#[test]
fn given_license_mit_filename_then_detected() {
    let dir = tempdir().unwrap();
    fs::write(
        dir.path().join("LICENSE-MIT"),
        "Permission is hereby granted, free of charge.\n\
         The software is provided \"as is\".",
    )
    .unwrap();

    let files = vec![PathBuf::from("LICENSE-MIT")];
    let report = build_license_report(dir.path(), &files, &default_limits()).unwrap();

    assert!(report.findings.iter().any(|f| f.spdx == "MIT"));
}

// ---------------------------------------------------------------------------
// Scenario: AGPL-3.0 detection
// ---------------------------------------------------------------------------

#[test]
fn given_agpl3_text_then_finds_agpl() {
    let dir = tempdir().unwrap();
    fs::write(
        dir.path().join("LICENSE"),
        "GNU Affero General Public License\n\
         Version 3, 19 November 2007\n\
         This program is free software: you can redistribute it and/or modify \
         it under the terms of the GNU Affero General Public License as published by \
         the Free Software Foundation, either version 3 of the License, or \
         (at your option) any later version.",
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

// ---------------------------------------------------------------------------
// Scenario: Unrecognized license text → no findings
// ---------------------------------------------------------------------------

#[test]
fn given_unrecognized_license_text_then_no_findings() {
    let dir = tempdir().unwrap();
    fs::write(
        dir.path().join("LICENSE"),
        "This is a proprietary license. All rights reserved. No copying.",
    )
    .unwrap();

    let files = vec![PathBuf::from("LICENSE")];
    let report = build_license_report(dir.path(), &files, &default_limits()).unwrap();

    assert!(report.findings.is_empty());
}

// ---------------------------------------------------------------------------
// Scenario: Cargo.toml license-file field triggers text scan
// ---------------------------------------------------------------------------

#[test]
fn given_cargo_toml_license_file_field_then_scans_referenced_file() {
    let dir = tempdir().unwrap();
    fs::write(
        dir.path().join("Cargo.toml"),
        "[package]\nname = \"x\"\nlicense-file = \"CUSTOM-LICENSE\"\n",
    )
    .unwrap();
    fs::write(
        dir.path().join("CUSTOM-LICENSE"),
        "Permission is hereby granted, free of charge.\n\
         The software is provided \"as is\".",
    )
    .unwrap();

    let files = vec![PathBuf::from("Cargo.toml"), PathBuf::from("CUSTOM-LICENSE")];
    let report = build_license_report(dir.path(), &files, &default_limits()).unwrap();

    assert!(report.findings.iter().any(|f| f.spdx == "MIT"
        && f.source_kind == LicenseSourceKind::Text
        && f.source_path == "CUSTOM-LICENSE"));
}

// ---------------------------------------------------------------------------
// Scenario: Source paths are forward-slash normalized
// ---------------------------------------------------------------------------

#[test]
fn source_paths_use_forward_slashes() {
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
            "path should not contain backslash: {}",
            f.source_path
        );
    }
}

// ---------------------------------------------------------------------------
// Scenario: Confidence range is always 0.0..=1.0
// ---------------------------------------------------------------------------

#[test]
fn confidence_in_valid_range() {
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
            "confidence out of range: {}",
            f.confidence
        );
    }
}
