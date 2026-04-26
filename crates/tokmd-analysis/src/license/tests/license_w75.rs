//! W75 security & identity tests for license radar scanning.
//!
//! Focuses on:
//! - Common license text detection (MIT, Apache, GPL, BSD, MPL, AGPL)
//! - No-license and dual-license scenarios
//! - License file naming patterns (LICENSE, COPYING, NOTICE variants)
//! - Metadata extraction from Cargo.toml, package.json, pyproject.toml
//! - Confidence scoring and effective license resolution

use crate::license::build_license_report;
use std::fs;
use std::path::PathBuf;
use tempfile::tempdir;
use tokmd_analysis_types::AnalysisLimits;
use tokmd_analysis_types::LicenseSourceKind;

fn default_limits() -> AnalysisLimits {
    AnalysisLimits::default()
}

// ===========================================================================
// 1. MIT license text detection
// ===========================================================================

#[test]
fn mit_full_text_detected_from_license_file() {
    let dir = tempdir().unwrap();
    fs::write(
        dir.path().join("LICENSE"),
        "MIT License\n\n\
         Copyright (c) 2024 Test Author\n\n\
         Permission is hereby granted, free of charge, to any person obtaining a copy \
         of this software and associated documentation files (the \"Software\"), to deal \
         in the Software without restriction, including without limitation the rights \
         to use, copy, modify, merge, publish, distribute, sublicense, and/or sell \
         copies of the Software, and to permit persons to whom the Software is furnished \
         to do so, subject to the following conditions:\n\n\
         THE SOFTWARE IS PROVIDED \"AS IS\", WITHOUT WARRANTY OF ANY KIND.",
    )
    .unwrap();

    let files = vec![PathBuf::from("LICENSE")];
    let report = build_license_report(dir.path(), &files, &default_limits()).unwrap();

    assert!(report.findings.iter().any(|f| f.spdx == "MIT"));
    assert_eq!(report.effective.as_deref(), Some("MIT"));
}

// ===========================================================================
// 2. Apache-2.0 text detection with multiple phrases
// ===========================================================================

#[test]
fn apache2_detected_with_multiple_phrases() {
    let dir = tempdir().unwrap();
    fs::write(
        dir.path().join("LICENSE"),
        "Apache License\nVersion 2.0, January 2004\n\
         http://www.apache.org/licenses/\n\n\
         TERMS AND CONDITIONS FOR USE, REPRODUCTION, AND DISTRIBUTION\n\n\
         Unless required by applicable law or agreed to in writing, software \
         distributed under the License is distributed on an \"AS IS\" BASIS, \
         WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND.\n\n\
         limitations under the License.",
    )
    .unwrap();

    let files = vec![PathBuf::from("LICENSE")];
    let report = build_license_report(dir.path(), &files, &default_limits()).unwrap();

    assert!(report.findings.iter().any(|f| f.spdx == "Apache-2.0"));
}

// ===========================================================================
// 3. GPL-3.0 text detection
// ===========================================================================

#[test]
fn gpl3_detected_from_copying_file() {
    let dir = tempdir().unwrap();
    fs::write(
        dir.path().join("COPYING"),
        "GNU General Public License\n\
         Version 3, 29 June 2007\n\n\
         Copyright (C) 2007 Free Software Foundation, Inc.\n\
         Everyone is permitted to copy and distribute verbatim copies\n\
         of this license document, but changing it is not allowed.\n\n\
         This program is free software: you can redistribute it and/or modify \
         it under the terms of the GNU General Public License as published by \
         the Free Software Foundation, either version 3 of the License, or \
         (at your option) any later version.",
    )
    .unwrap();

    let files = vec![PathBuf::from("COPYING")];
    let report = build_license_report(dir.path(), &files, &default_limits()).unwrap();

    assert!(report.findings.iter().any(|f| f.spdx == "GPL-3.0-or-later"));
}

// ===========================================================================
// 4. BSD-3-Clause text detection
// ===========================================================================

#[test]
fn bsd3_clause_detected_from_text() {
    let dir = tempdir().unwrap();
    fs::write(
        dir.path().join("LICENSE"),
        "Copyright (c) 2024, Example Corp.\n\n\
         Redistribution and use in source and binary forms, with or without \
         modification, are permitted provided that the following conditions are met:\n\n\
         1. Redistributions of source code must retain the above copyright notice.\n\
         2. Redistributions in binary form must reproduce the above copyright notice.\n\
         3. Neither the name of the copyright holder nor the names of its \
         contributors may be used to endorse or promote products derived from \
         this software without specific prior written permission.",
    )
    .unwrap();

    let files = vec![PathBuf::from("LICENSE")];
    let report = build_license_report(dir.path(), &files, &default_limits()).unwrap();

    assert!(
        report.findings.iter().any(|f| f.spdx == "BSD-3-Clause"),
        "should detect BSD-3-Clause: {:?}",
        report.findings
    );
}

// ===========================================================================
// 5. No license detected from unrecognized text
// ===========================================================================

#[test]
fn no_license_detected_from_unrecognized_text() {
    let dir = tempdir().unwrap();
    fs::write(
        dir.path().join("LICENSE"),
        "This is a custom proprietary license.\n\
         You may not use, copy, or distribute this software.\n\
         All rights reserved by the author.",
    )
    .unwrap();

    let files = vec![PathBuf::from("LICENSE")];
    let report = build_license_report(dir.path(), &files, &default_limits()).unwrap();

    assert!(
        report.findings.is_empty(),
        "proprietary text should not match any known license"
    );
    assert!(report.effective.is_none());
}

// ===========================================================================
// 6. No license when no files provided
// ===========================================================================

#[test]
fn no_license_when_no_files_provided() {
    let dir = tempdir().unwrap();
    let report = build_license_report(dir.path(), &[], &default_limits()).unwrap();

    assert!(report.findings.is_empty());
    assert!(report.effective.is_none());
}

// ===========================================================================
// 7. Dual license: metadata MIT + text Apache
// ===========================================================================

#[test]
fn dual_license_metadata_and_text_produce_separate_findings() {
    let dir = tempdir().unwrap();
    fs::write(
        dir.path().join("Cargo.toml"),
        "[package]\nname = \"x\"\nlicense = \"MIT OR Apache-2.0\"\n",
    )
    .unwrap();
    fs::write(
        dir.path().join("LICENSE-APACHE"),
        "Apache License\nVersion 2.0\n\
         http://www.apache.org/licenses/\n\
         limitations under the License.",
    )
    .unwrap();
    fs::write(
        dir.path().join("LICENSE-MIT"),
        "Permission is hereby granted, free of charge.\n\
         The software is provided \"as is\".",
    )
    .unwrap();

    let files = vec![
        PathBuf::from("Cargo.toml"),
        PathBuf::from("LICENSE-APACHE"),
        PathBuf::from("LICENSE-MIT"),
    ];
    let report = build_license_report(dir.path(), &files, &default_limits()).unwrap();

    // Should have metadata finding + at least one text finding
    assert!(
        report
            .findings
            .iter()
            .any(|f| f.source_kind == LicenseSourceKind::Metadata),
        "should have metadata finding"
    );
    assert!(
        report
            .findings
            .iter()
            .any(|f| f.source_kind == LicenseSourceKind::Text),
        "should have text finding"
    );
    assert!(report.findings.len() >= 2);
}

// ===========================================================================
// 8. SPDX OR expression in metadata preserved verbatim
// ===========================================================================

#[test]
fn spdx_or_expression_preserved_in_metadata() {
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

// ===========================================================================
// 9. LICENSE-APACHE naming pattern recognized
// ===========================================================================

#[test]
fn license_apache_naming_pattern_recognized() {
    let dir = tempdir().unwrap();
    fs::write(
        dir.path().join("LICENSE-APACHE"),
        "Apache License\nVersion 2.0\n\
         http://www.apache.org/licenses/\n\
         limitations under the License.",
    )
    .unwrap();

    let files = vec![PathBuf::from("LICENSE-APACHE")];
    let report = build_license_report(dir.path(), &files, &default_limits()).unwrap();

    assert!(
        report.findings.iter().any(|f| f.spdx == "Apache-2.0"),
        "LICENSE-APACHE file should be recognized: {:?}",
        report.findings
    );
}

// ===========================================================================
// 10. COPYING naming pattern recognized
// ===========================================================================

#[test]
fn copying_naming_pattern_recognized() {
    let dir = tempdir().unwrap();
    fs::write(
        dir.path().join("COPYING.txt"),
        "Permission is hereby granted, free of charge.\n\
         The software is provided \"as is\".",
    )
    .unwrap();

    let files = vec![PathBuf::from("COPYING.txt")];
    let report = build_license_report(dir.path(), &files, &default_limits()).unwrap();

    assert!(
        report.findings.iter().any(|f| f.spdx == "MIT"),
        "COPYING.txt should be scanned: {:?}",
        report.findings
    );
}

// ===========================================================================
// 11. NOTICE file naming pattern recognized
// ===========================================================================

#[test]
fn notice_naming_pattern_recognized() {
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

// ===========================================================================
// 12. pyproject.toml with [project] section license
// ===========================================================================

#[test]
fn pyproject_toml_project_section_license() {
    let dir = tempdir().unwrap();
    fs::write(
        dir.path().join("pyproject.toml"),
        "[project]\nname = \"demo\"\nlicense = \"MIT\"\n",
    )
    .unwrap();

    let files = vec![PathBuf::from("pyproject.toml")];
    let report = build_license_report(dir.path(), &files, &default_limits()).unwrap();

    assert_eq!(report.findings.len(), 1);
    assert_eq!(report.findings[0].spdx, "MIT");
    assert_eq!(report.findings[0].source_kind, LicenseSourceKind::Metadata);
}

// ===========================================================================
// 13. AGPL-3.0 text detection
// ===========================================================================

#[test]
fn agpl3_detected_from_text() {
    let dir = tempdir().unwrap();
    fs::write(
        dir.path().join("LICENSE"),
        "GNU Affero General Public License\n\
         Version 3, 19 November 2007\n\n\
         Copyright (C) 2007 Free Software Foundation, Inc.\n\
         Everyone is permitted to copy and distribute verbatim copies\n\
         of this license document, but changing it is not allowed.\n\n\
         This program is free software: you can redistribute it and/or modify \
         it under the terms of the GNU Affero General Public License as \
         published by the Free Software Foundation, either version 3 of the License, \
         or (at your option) any later version.",
    )
    .unwrap();

    let files = vec![PathBuf::from("LICENSE")];
    let report = build_license_report(dir.path(), &files, &default_limits()).unwrap();

    assert!(
        report
            .findings
            .iter()
            .any(|f| f.spdx == "AGPL-3.0-or-later"),
        "should detect AGPL-3.0: {:?}",
        report.findings
    );
}

// ===========================================================================
// 14. Findings sorted by confidence descending
// ===========================================================================

#[test]
fn findings_sorted_by_confidence_descending() {
    let dir = tempdir().unwrap();
    // Metadata finding (confidence 0.95) + text finding (variable confidence)
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
    let report = build_license_report(dir.path(), &files, &default_limits()).unwrap();

    assert!(report.findings.len() >= 2);
    for pair in report.findings.windows(2) {
        assert!(
            pair[0].confidence >= pair[1].confidence,
            "findings should be sorted by confidence descending: {} >= {}",
            pair[0].confidence,
            pair[1].confidence
        );
    }
}

// ===========================================================================
// 15. Effective license is the first (highest confidence) finding
// ===========================================================================

#[test]
fn effective_license_matches_first_finding() {
    let dir = tempdir().unwrap();
    fs::write(
        dir.path().join("LICENSE"),
        "Permission is hereby granted, free of charge.\n\
         The software is provided \"as is\".",
    )
    .unwrap();
    fs::write(
        dir.path().join("Cargo.toml"),
        "[package]\nname = \"x\"\nlicense = \"ISC\"\n",
    )
    .unwrap();

    let files = vec![PathBuf::from("LICENSE"), PathBuf::from("Cargo.toml")];
    let report = build_license_report(dir.path(), &files, &default_limits()).unwrap();

    assert!(report.effective.is_some());
    assert_eq!(
        report.effective.as_deref(),
        report.findings.first().map(|f| f.spdx.as_str()),
        "effective license should match the first (highest confidence) finding"
    );
}
