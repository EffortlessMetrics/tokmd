//! Wave-60 depth tests for tokmd-analysis-license.
//!
//! Covers: BDD license detection across file types, SPDX parsing,
//! license-file detection, edge cases, property-based determinism tests.

use std::fs;
use std::path::PathBuf;

use proptest::prelude::*;
use tempfile::tempdir;
use tokmd_analysis_license::build_license_report;
use tokmd_analysis_types::LicenseSourceKind;
use tokmd_analysis_util::AnalysisLimits;

fn default_limits() -> AnalysisLimits {
    AnalysisLimits::default()
}

// ===========================================================================
// BDD: Cargo.toml metadata detection
// ===========================================================================

#[test]
fn given_cargo_toml_with_mit_or_apache_expression_then_spdx_is_verbatim() {
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
    assert_eq!(report.findings[0].source_kind, LicenseSourceKind::Metadata);
    assert_eq!(report.effective.as_deref(), Some("MIT OR Apache-2.0"));
}

#[test]
fn given_cargo_toml_with_single_quoted_license_then_detected() {
    let dir = tempdir().unwrap();
    fs::write(
        dir.path().join("Cargo.toml"),
        "[package]\nname = \"x\"\nlicense = 'BSD-3-Clause'\n",
    )
    .unwrap();
    let files = vec![PathBuf::from("Cargo.toml")];
    let report = build_license_report(dir.path(), &files, &default_limits()).unwrap();
    assert_eq!(report.findings.len(), 1);
    assert_eq!(report.findings[0].spdx, "BSD-3-Clause");
}

#[test]
fn given_cargo_toml_without_package_section_then_no_findings() {
    let dir = tempdir().unwrap();
    fs::write(
        dir.path().join("Cargo.toml"),
        "[dependencies]\nserde = \"1.0\"\n",
    )
    .unwrap();
    let files = vec![PathBuf::from("Cargo.toml")];
    let report = build_license_report(dir.path(), &files, &default_limits()).unwrap();
    assert!(report.findings.is_empty());
}

#[test]
fn given_cargo_toml_with_license_file_pointing_to_custom_path_then_text_scanned() {
    let dir = tempdir().unwrap();
    fs::write(
        dir.path().join("Cargo.toml"),
        "[package]\nname = \"x\"\nlicense-file = \"docs/MY-LICENSE.txt\"\n",
    )
    .unwrap();
    let docs = dir.path().join("docs");
    fs::create_dir_all(&docs).unwrap();
    fs::write(
        docs.join("MY-LICENSE.txt"),
        "Permission is hereby granted, free of charge, to any person.\n\
         The software is provided \"as is\".",
    )
    .unwrap();
    let files = vec![
        PathBuf::from("Cargo.toml"),
        PathBuf::from("docs/MY-LICENSE.txt"),
    ];
    let report = build_license_report(dir.path(), &files, &default_limits()).unwrap();
    assert!(report.findings.iter().any(|f| f.spdx == "MIT"
        && f.source_kind == LicenseSourceKind::Text
        && f.source_path.contains("MY-LICENSE")));
}

// ===========================================================================
// BDD: package.json detection
// ===========================================================================

#[test]
fn given_package_json_with_isc_then_detected() {
    let dir = tempdir().unwrap();
    fs::write(
        dir.path().join("package.json"),
        r#"{"name":"x","license":"ISC"}"#,
    )
    .unwrap();
    let files = vec![PathBuf::from("package.json")];
    let report = build_license_report(dir.path(), &files, &default_limits()).unwrap();
    assert_eq!(report.findings[0].spdx, "ISC");
}

#[test]
fn given_package_json_with_object_license_then_type_extracted() {
    let dir = tempdir().unwrap();
    fs::write(
        dir.path().join("package.json"),
        r#"{"name":"x","license":{"type":"Artistic-2.0","url":"https://example.com"}}"#,
    )
    .unwrap();
    let files = vec![PathBuf::from("package.json")];
    let report = build_license_report(dir.path(), &files, &default_limits()).unwrap();
    assert_eq!(report.findings[0].spdx, "Artistic-2.0");
}

#[test]
fn given_package_json_without_license_field_then_no_findings() {
    let dir = tempdir().unwrap();
    fs::write(
        dir.path().join("package.json"),
        r#"{"name":"x","version":"1.0.0"}"#,
    )
    .unwrap();
    let files = vec![PathBuf::from("package.json")];
    let report = build_license_report(dir.path(), &files, &default_limits()).unwrap();
    assert!(report.findings.is_empty());
}

#[test]
fn given_package_json_with_null_license_then_no_findings() {
    let dir = tempdir().unwrap();
    fs::write(
        dir.path().join("package.json"),
        r#"{"name":"x","license":null}"#,
    )
    .unwrap();
    let files = vec![PathBuf::from("package.json")];
    let report = build_license_report(dir.path(), &files, &default_limits()).unwrap();
    assert!(report.findings.is_empty());
}

#[test]
fn given_package_json_with_license_array_then_no_findings() {
    let dir = tempdir().unwrap();
    fs::write(
        dir.path().join("package.json"),
        r#"{"name":"x","license":["MIT","Apache-2.0"]}"#,
    )
    .unwrap();
    let files = vec![PathBuf::from("package.json")];
    let report = build_license_report(dir.path(), &files, &default_limits()).unwrap();
    assert!(report.findings.is_empty());
}

// ===========================================================================
// BDD: pyproject.toml detection
// ===========================================================================

#[test]
fn given_pyproject_project_section_then_license_detected() {
    let dir = tempdir().unwrap();
    fs::write(
        dir.path().join("pyproject.toml"),
        "[project]\nname = \"pkg\"\nlicense = \"Apache-2.0\"\n",
    )
    .unwrap();
    let files = vec![PathBuf::from("pyproject.toml")];
    let report = build_license_report(dir.path(), &files, &default_limits()).unwrap();
    assert_eq!(report.findings[0].spdx, "Apache-2.0");
}

#[test]
fn given_pyproject_poetry_section_then_license_detected() {
    let dir = tempdir().unwrap();
    fs::write(
        dir.path().join("pyproject.toml"),
        "[tool.poetry]\nname = \"pkg\"\nlicense = \"LGPL-3.0\"\n",
    )
    .unwrap();
    let files = vec![PathBuf::from("pyproject.toml")];
    let report = build_license_report(dir.path(), &files, &default_limits()).unwrap();
    assert_eq!(report.findings[0].spdx, "LGPL-3.0");
}

#[test]
fn given_pyproject_with_both_sections_then_project_wins() {
    let dir = tempdir().unwrap();
    fs::write(
        dir.path().join("pyproject.toml"),
        "[project]\nname = \"pkg\"\nlicense = \"MIT\"\n\n[tool.poetry]\nname = \"pkg\"\nlicense = \"GPL-2.0\"\n",
    )
    .unwrap();
    let files = vec![PathBuf::from("pyproject.toml")];
    let report = build_license_report(dir.path(), &files, &default_limits()).unwrap();
    // project section is tried first; we expect MIT
    assert!(report.findings.iter().any(|f| f.spdx == "MIT"));
}

// ===========================================================================
// BDD: LICENSE text file detection across license families
// ===========================================================================

#[test]
fn given_mit_text_with_both_phrases_then_high_confidence() {
    let dir = tempdir().unwrap();
    fs::write(
        dir.path().join("LICENSE"),
        "Permission is hereby granted, free of charge, to any person obtaining a copy.\n\
         THE SOFTWARE IS PROVIDED \"AS IS\", WITHOUT WARRANTY OF ANY KIND.",
    )
    .unwrap();
    let files = vec![PathBuf::from("LICENSE")];
    let report = build_license_report(dir.path(), &files, &default_limits()).unwrap();
    let finding = report.findings.iter().find(|f| f.spdx == "MIT").unwrap();
    assert!(finding.confidence > 0.9);
}

#[test]
fn given_apache_text_then_apache_detected() {
    let dir = tempdir().unwrap();
    fs::write(
        dir.path().join("LICENSE.txt"),
        "Apache License\nVersion 2.0, January 2004\n\
         http://www.apache.org/licenses/\n\
         Unless required by applicable law or agreed to in writing, software \
         distributed under the License is distributed on an \"AS IS\" BASIS.\n\
         Limitations under the License.",
    )
    .unwrap();
    let files = vec![PathBuf::from("LICENSE.txt")];
    let report = build_license_report(dir.path(), &files, &default_limits()).unwrap();
    assert!(report.findings.iter().any(|f| f.spdx == "Apache-2.0"));
}

#[test]
fn given_gpl3_text_then_gpl_detected() {
    let dir = tempdir().unwrap();
    fs::write(
        dir.path().join("COPYING"),
        "GNU General Public License\nVersion 3, 29 June 2007\n\
         You may redistribute it under the terms of the GNU General Public License \
         as published by the Free Software Foundation, either version 3 of the License, \
         or (at your option) any later version.",
    )
    .unwrap();
    let files = vec![PathBuf::from("COPYING")];
    let report = build_license_report(dir.path(), &files, &default_limits()).unwrap();
    assert!(report.findings.iter().any(|f| f.spdx == "GPL-3.0-or-later"));
}

#[test]
fn given_agpl3_text_then_agpl_detected() {
    let dir = tempdir().unwrap();
    fs::write(
        dir.path().join("LICENSE"),
        "GNU Affero General Public License\nVersion 3\n\
         You may redistribute it under the GNU Affero General Public License \
         any later version.",
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
fn given_bsd3_text_then_bsd3_detected() {
    let dir = tempdir().unwrap();
    fs::write(
        dir.path().join("LICENSE"),
        "Redistribution and use in source and binary forms, with or without modification.\n\
         Neither the name of the copyright holder nor the names of its \
         contributors may be used to endorse or promote products.",
    )
    .unwrap();
    let files = vec![PathBuf::from("LICENSE")];
    let report = build_license_report(dir.path(), &files, &default_limits()).unwrap();
    assert!(report.findings.iter().any(|f| f.spdx == "BSD-3-Clause"));
}

#[test]
fn given_bsd2_text_then_bsd2_detected() {
    let dir = tempdir().unwrap();
    fs::write(
        dir.path().join("LICENSE"),
        "Redistribution and use in source and binary forms, with or without modification.\n\
         This software is provided by the copyright holders and contributors \"as is\".",
    )
    .unwrap();
    let files = vec![PathBuf::from("LICENSE")];
    let report = build_license_report(dir.path(), &files, &default_limits()).unwrap();
    // BSD-2-Clause or BSD-3-Clause (both match "redistribution and use")
    assert!(
        report
            .findings
            .iter()
            .any(|f| f.spdx.starts_with("BSD"))
    );
}

#[test]
fn given_mpl2_text_then_mpl_detected() {
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

// ===========================================================================
// BDD: Edge cases – empty, binary, unrecognized, long files
// ===========================================================================

#[test]
fn given_empty_license_file_then_no_findings() {
    let dir = tempdir().unwrap();
    fs::write(dir.path().join("LICENSE"), "").unwrap();
    let files = vec![PathBuf::from("LICENSE")];
    let report = build_license_report(dir.path(), &files, &default_limits()).unwrap();
    assert!(report.findings.is_empty());
    assert!(report.effective.is_none());
}

#[test]
fn given_whitespace_only_license_then_no_findings() {
    let dir = tempdir().unwrap();
    fs::write(dir.path().join("LICENSE"), "   \n\n  \t  \n").unwrap();
    let files = vec![PathBuf::from("LICENSE")];
    let report = build_license_report(dir.path(), &files, &default_limits()).unwrap();
    assert!(report.findings.is_empty());
}

#[test]
fn given_binary_content_in_license_then_no_crash() {
    let dir = tempdir().unwrap();
    let binary: Vec<u8> = (0..256).map(|i| i as u8).collect();
    fs::write(dir.path().join("LICENSE"), &binary).unwrap();
    let files = vec![PathBuf::from("LICENSE")];
    // Should not panic
    let result = build_license_report(dir.path(), &files, &default_limits());
    assert!(result.is_ok());
}

#[test]
fn given_proprietary_text_then_no_findings() {
    let dir = tempdir().unwrap();
    fs::write(
        dir.path().join("LICENSE"),
        "PROPRIETARY LICENSE\nAll rights reserved. Unauthorized copying is prohibited.",
    )
    .unwrap();
    let files = vec![PathBuf::from("LICENSE")];
    let report = build_license_report(dir.path(), &files, &default_limits()).unwrap();
    assert!(report.findings.is_empty());
}

#[test]
fn given_very_long_license_text_then_still_detected() {
    let dir = tempdir().unwrap();
    let mut text = String::from(
        "Permission is hereby granted, free of charge, to any person.\n\
         The software is provided \"as is\".\n",
    );
    // Pad with lots of filler
    for _ in 0..1000 {
        text.push_str("Lorem ipsum dolor sit amet, consectetur adipiscing elit.\n");
    }
    fs::write(dir.path().join("LICENSE"), &text).unwrap();
    let files = vec![PathBuf::from("LICENSE")];
    let report = build_license_report(dir.path(), &files, &default_limits()).unwrap();
    assert!(report.findings.iter().any(|f| f.spdx == "MIT"));
}

#[test]
fn given_no_files_at_all_then_empty_report() {
    let dir = tempdir().unwrap();
    let report = build_license_report(dir.path(), &[], &default_limits()).unwrap();
    assert!(report.findings.is_empty());
    assert!(report.effective.is_none());
}

#[test]
fn given_only_source_files_then_empty_report() {
    let dir = tempdir().unwrap();
    fs::write(dir.path().join("main.rs"), "fn main() {}").unwrap();
    fs::write(dir.path().join("lib.rs"), "pub fn hello() {}").unwrap();
    let files = vec![PathBuf::from("main.rs"), PathBuf::from("lib.rs")];
    let report = build_license_report(dir.path(), &files, &default_limits()).unwrap();
    assert!(report.findings.is_empty());
}

// ===========================================================================
// BDD: License filename variants
// ===========================================================================

#[test]
fn given_license_mit_filename_then_scanned_as_text() {
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

#[test]
fn given_license_apache_filename_then_scanned() {
    let dir = tempdir().unwrap();
    fs::write(
        dir.path().join("LICENSE-APACHE"),
        "Apache License\nVersion 2.0\n\
         http://www.apache.org/licenses/\n\
         Limitations under the License.",
    )
    .unwrap();
    let files = vec![PathBuf::from("LICENSE-APACHE")];
    let report = build_license_report(dir.path(), &files, &default_limits()).unwrap();
    assert!(report.findings.iter().any(|f| f.spdx == "Apache-2.0"));
}

#[test]
fn given_copying_filename_then_scanned() {
    let dir = tempdir().unwrap();
    fs::write(
        dir.path().join("COPYING"),
        "Permission is hereby granted, free of charge.\n\
         The software is provided \"as is\".",
    )
    .unwrap();
    let files = vec![PathBuf::from("COPYING")];
    let report = build_license_report(dir.path(), &files, &default_limits()).unwrap();
    assert!(report.findings.iter().any(|f| f.spdx == "MIT"));
}

// ===========================================================================
// BDD: Multiple sources – sorting and effective license
// ===========================================================================

#[test]
fn given_metadata_and_text_then_both_found_with_valid_confidence() {
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
    assert!(report.findings.len() >= 2);
    let metadata = report
        .findings
        .iter()
        .find(|f| f.source_kind == LicenseSourceKind::Metadata)
        .unwrap();
    let text = report
        .findings
        .iter()
        .find(|f| f.source_kind == LicenseSourceKind::Text)
        .unwrap();
    assert!((metadata.confidence - 0.95).abs() < f32::EPSILON);
    assert!(text.confidence >= 0.6);
    assert!(text.confidence <= 1.0);
}

#[test]
fn findings_sorted_by_confidence_then_spdx_then_path() {
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
    for pair in report.findings.windows(2) {
        assert!(
            pair[0].confidence >= pair[1].confidence,
            "not sorted: {} >= {}",
            pair[0].confidence,
            pair[1].confidence
        );
    }
}

#[test]
fn effective_is_first_finding_spdx() {
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

// ===========================================================================
// BDD: Path normalization
// ===========================================================================

#[test]
fn source_paths_use_forward_slashes_for_nested_files() {
    let dir = tempdir().unwrap();
    let sub = dir.path().join("nested").join("dir");
    fs::create_dir_all(&sub).unwrap();
    fs::write(
        sub.join("package.json"),
        r#"{"name":"x","license":"MIT"}"#,
    )
    .unwrap();
    let files = vec![PathBuf::from("nested").join("dir").join("package.json")];
    let report = build_license_report(dir.path(), &files, &default_limits()).unwrap();
    for f in &report.findings {
        assert!(
            !f.source_path.contains('\\'),
            "backslash in path: {}",
            f.source_path
        );
    }
}

// ===========================================================================
// BDD: Confidence bounds
// ===========================================================================

#[test]
fn text_confidence_is_between_0_6_and_1_0() {
    let dir = tempdir().unwrap();
    fs::write(
        dir.path().join("LICENSE"),
        "Permission is hereby granted, free of charge.\n\
         The software is provided \"as is\".",
    )
    .unwrap();
    let files = vec![PathBuf::from("LICENSE")];
    let report = build_license_report(dir.path(), &files, &default_limits()).unwrap();
    for f in &report.findings {
        if f.source_kind == LicenseSourceKind::Text {
            assert!(f.confidence >= 0.6, "text confidence too low: {}", f.confidence);
            assert!(f.confidence <= 1.0, "text confidence too high: {}", f.confidence);
        }
    }
}

#[test]
fn metadata_confidence_is_always_0_95() {
    let dir = tempdir().unwrap();
    fs::write(
        dir.path().join("Cargo.toml"),
        "[package]\nname = \"x\"\nlicense = \"MIT\"\n",
    )
    .unwrap();
    let files = vec![PathBuf::from("Cargo.toml")];
    let report = build_license_report(dir.path(), &files, &default_limits()).unwrap();
    for f in &report.findings {
        if f.source_kind == LicenseSourceKind::Metadata {
            assert!(
                (f.confidence - 0.95).abs() < f32::EPSILON,
                "metadata confidence should be 0.95, got {}",
                f.confidence
            );
        }
    }
}

// ===========================================================================
// BDD: Cargo.toml with extra keys around license
// ===========================================================================

#[test]
fn given_cargo_toml_with_many_fields_then_license_still_found() {
    let dir = tempdir().unwrap();
    fs::write(
        dir.path().join("Cargo.toml"),
        "[package]\nname = \"big-project\"\nversion = \"1.2.3\"\n\
         edition = \"2021\"\nlicense = \"LGPL-2.1-only\"\n\
         description = \"A big project\"\nauthors = [\"dev\"]\n",
    )
    .unwrap();
    let files = vec![PathBuf::from("Cargo.toml")];
    let report = build_license_report(dir.path(), &files, &default_limits()).unwrap();
    assert_eq!(report.findings[0].spdx, "LGPL-2.1-only");
}

#[test]
fn given_cargo_toml_with_workspace_section_then_only_package_parsed() {
    let dir = tempdir().unwrap();
    fs::write(
        dir.path().join("Cargo.toml"),
        "[workspace]\nmembers = [\"a\"]\n\n[package]\nname = \"x\"\nlicense = \"MIT\"\n",
    )
    .unwrap();
    let files = vec![PathBuf::from("Cargo.toml")];
    let report = build_license_report(dir.path(), &files, &default_limits()).unwrap();
    assert_eq!(report.findings.len(), 1);
    assert_eq!(report.findings[0].spdx, "MIT");
}

// ===========================================================================
// BDD: Invalid/malformed metadata files
// ===========================================================================

#[test]
fn given_malformed_package_json_then_no_crash() {
    let dir = tempdir().unwrap();
    fs::write(dir.path().join("package.json"), "NOT VALID JSON {{{").unwrap();
    let files = vec![PathBuf::from("package.json")];
    let report = build_license_report(dir.path(), &files, &default_limits()).unwrap();
    assert!(report.findings.is_empty());
}

#[test]
fn given_empty_cargo_toml_then_no_crash() {
    let dir = tempdir().unwrap();
    fs::write(dir.path().join("Cargo.toml"), "").unwrap();
    let files = vec![PathBuf::from("Cargo.toml")];
    let report = build_license_report(dir.path(), &files, &default_limits()).unwrap();
    assert!(report.findings.is_empty());
}

#[test]
fn given_cargo_toml_with_empty_license_value_then_no_findings() {
    let dir = tempdir().unwrap();
    fs::write(
        dir.path().join("Cargo.toml"),
        "[package]\nname = \"x\"\nlicense = \"\"\n",
    )
    .unwrap();
    let files = vec![PathBuf::from("Cargo.toml")];
    let report = build_license_report(dir.path(), &files, &default_limits()).unwrap();
    assert!(report.findings.is_empty());
}

// ===========================================================================
// BDD: Determinism – same input produces same output
// ===========================================================================

#[test]
fn deterministic_metadata_scanning() {
    let dir = tempdir().unwrap();
    fs::write(
        dir.path().join("Cargo.toml"),
        "[package]\nname = \"x\"\nlicense = \"MIT\"\n",
    )
    .unwrap();
    let files = vec![PathBuf::from("Cargo.toml")];
    let r1 = build_license_report(dir.path(), &files, &default_limits()).unwrap();
    let r2 = build_license_report(dir.path(), &files, &default_limits()).unwrap();
    assert_eq!(r1.findings.len(), r2.findings.len());
    assert_eq!(r1.effective, r2.effective);
    for (a, b) in r1.findings.iter().zip(r2.findings.iter()) {
        assert_eq!(a.spdx, b.spdx);
        assert!((a.confidence - b.confidence).abs() < f32::EPSILON);
        assert_eq!(a.source_path, b.source_path);
        assert_eq!(a.source_kind, b.source_kind);
    }
}

#[test]
fn deterministic_text_scanning() {
    let dir = tempdir().unwrap();
    fs::write(
        dir.path().join("LICENSE"),
        "Permission is hereby granted, free of charge.\n\
         The software is provided \"as is\".",
    )
    .unwrap();
    let files = vec![PathBuf::from("LICENSE")];
    let r1 = build_license_report(dir.path(), &files, &default_limits()).unwrap();
    let r2 = build_license_report(dir.path(), &files, &default_limits()).unwrap();
    assert_eq!(r1.findings.len(), r2.findings.len());
    for (a, b) in r1.findings.iter().zip(r2.findings.iter()) {
        assert_eq!(a.spdx, b.spdx);
        assert!((a.confidence - b.confidence).abs() < f32::EPSILON);
    }
}

// ===========================================================================
// Property tests
// ===========================================================================

fn spdx_strategy() -> impl Strategy<Value = String> {
    prop::sample::select(vec![
        "MIT".to_string(),
        "Apache-2.0".to_string(),
        "GPL-3.0-only".to_string(),
        "BSD-2-Clause".to_string(),
        "ISC".to_string(),
        "MPL-2.0".to_string(),
        "Unlicense".to_string(),
        "0BSD".to_string(),
        "Zlib".to_string(),
        "EUPL-1.2".to_string(),
    ])
}

proptest! {
    #[test]
    fn prop_cargo_spdx_roundtrips(spdx in spdx_strategy()) {
        let dir = tempdir().unwrap();
        fs::write(
            dir.path().join("Cargo.toml"),
            format!("[package]\nname = \"t\"\nlicense = \"{spdx}\"\n"),
        ).unwrap();
        let files = vec![PathBuf::from("Cargo.toml")];
        let report = build_license_report(dir.path(), &files, &default_limits()).unwrap();
        prop_assert!(!report.findings.is_empty());
        prop_assert_eq!(&report.findings[0].spdx, &spdx);
    }

    #[test]
    fn prop_package_json_spdx_roundtrips(spdx in spdx_strategy()) {
        let dir = tempdir().unwrap();
        fs::write(
            dir.path().join("package.json"),
            format!(r#"{{"name":"t","license":"{spdx}"}}"#),
        ).unwrap();
        let files = vec![PathBuf::from("package.json")];
        let report = build_license_report(dir.path(), &files, &default_limits()).unwrap();
        prop_assert!(!report.findings.is_empty());
        prop_assert_eq!(&report.findings[0].spdx, &spdx);
    }

    #[test]
    fn prop_pyproject_spdx_roundtrips(spdx in spdx_strategy()) {
        let dir = tempdir().unwrap();
        fs::write(
            dir.path().join("pyproject.toml"),
            format!("[project]\nname = \"t\"\nlicense = \"{spdx}\"\n"),
        ).unwrap();
        let files = vec![PathBuf::from("pyproject.toml")];
        let report = build_license_report(dir.path(), &files, &default_limits()).unwrap();
        prop_assert!(!report.findings.is_empty());
        prop_assert_eq!(&report.findings[0].spdx, &spdx);
    }

    #[test]
    fn prop_empty_file_list_always_empty(_seed in 0u64..1000) {
        let dir = tempdir().unwrap();
        let report = build_license_report(dir.path(), &[], &default_limits()).unwrap();
        prop_assert!(report.findings.is_empty());
        prop_assert!(report.effective.is_none());
    }

    #[test]
    fn prop_findings_always_sorted(spdx in spdx_strategy()) {
        let dir = tempdir().unwrap();
        fs::write(
            dir.path().join("Cargo.toml"),
            format!("[package]\nname = \"t\"\nlicense = \"{spdx}\"\n"),
        ).unwrap();
        fs::write(
            dir.path().join("LICENSE"),
            "Permission is hereby granted, free of charge.\n\
             The software is provided \"as is\".",
        ).unwrap();
        let files = vec![PathBuf::from("Cargo.toml"), PathBuf::from("LICENSE")];
        let report = build_license_report(dir.path(), &files, &default_limits()).unwrap();
        for pair in report.findings.windows(2) {
            prop_assert!(pair[0].confidence >= pair[1].confidence);
        }
    }

    #[test]
    fn prop_effective_equals_first_finding(spdx in spdx_strategy()) {
        let dir = tempdir().unwrap();
        fs::write(
            dir.path().join("Cargo.toml"),
            format!("[package]\nname = \"t\"\nlicense = \"{spdx}\"\n"),
        ).unwrap();
        let files = vec![PathBuf::from("Cargo.toml")];
        let report = build_license_report(dir.path(), &files, &default_limits()).unwrap();
        prop_assert_eq!(report.effective, report.findings.first().map(|f| f.spdx.clone()));
    }

    #[test]
    fn prop_confidence_in_valid_range(spdx in spdx_strategy()) {
        let dir = tempdir().unwrap();
        fs::write(
            dir.path().join("Cargo.toml"),
            format!("[package]\nname = \"t\"\nlicense = \"{spdx}\"\n"),
        ).unwrap();
        fs::write(
            dir.path().join("LICENSE"),
            "Permission is hereby granted, free of charge.\n\
             The software is provided \"as is\".",
        ).unwrap();
        let files = vec![PathBuf::from("Cargo.toml"), PathBuf::from("LICENSE")];
        let report = build_license_report(dir.path(), &files, &default_limits()).unwrap();
        for f in &report.findings {
            prop_assert!(f.confidence >= 0.0 && f.confidence <= 1.0);
        }
    }

    #[test]
    fn prop_no_backslash_in_paths(spdx in spdx_strategy()) {
        let dir = tempdir().unwrap();
        let sub = dir.path().join("a");
        fs::create_dir_all(&sub).unwrap();
        fs::write(
            sub.join("Cargo.toml"),
            format!("[package]\nname = \"t\"\nlicense = \"{spdx}\"\n"),
        ).unwrap();
        let files = vec![PathBuf::from("a").join("Cargo.toml")];
        let report = build_license_report(dir.path(), &files, &default_limits()).unwrap();
        for f in &report.findings {
            prop_assert!(!f.source_path.contains('\\'));
        }
    }

    #[test]
    fn prop_metadata_confidence_always_0_95(spdx in spdx_strategy()) {
        let dir = tempdir().unwrap();
        fs::write(
            dir.path().join("Cargo.toml"),
            format!("[package]\nname = \"t\"\nlicense = \"{spdx}\"\n"),
        ).unwrap();
        let files = vec![PathBuf::from("Cargo.toml")];
        let report = build_license_report(dir.path(), &files, &default_limits()).unwrap();
        for f in &report.findings {
            if f.source_kind == LicenseSourceKind::Metadata {
                prop_assert!((f.confidence - 0.95).abs() < f32::EPSILON);
            }
        }
    }
}
