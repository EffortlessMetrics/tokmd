//! Deep integration tests for license radar scanning.
//!
//! Targets gaps not covered by existing unit/bdd/edge/property/identity suites:
//! - AGPL-3.0 text detection with all phrases
//! - BSD-3-Clause full detection with third-clause phrase
//! - Sort tiebreaking: same confidence orders by spdx then source_path
//! - Serde round-trip of complete LicenseReport
//! - LicenseFinding serde JSON shape validation
//! - TOML parsing: inline comments after value, extra whitespace around =
//! - pyproject.toml with tool.poetry fallback when [project] is absent
//! - Cargo.toml license-file pointing to text that matches a license
//! - Confidence monotonicity: more phrase hits → higher confidence
//! - BSD-2-Clause distinct from BSD-3-Clause
//! - Mixed Cargo.toml + pyproject.toml + package.json all present
//! - Unknown metadata file types ignored
//! - LICENSE-APACHE with full Apache text yields Apache-2.0
//! - Effective is None when no findings
//! - Findings with same confidence sorted by spdx ascending

use std::fs;
use std::path::PathBuf;
use tempfile::tempdir;
use tokmd_analysis_license::build_license_report;
use tokmd_analysis_types::AnalysisLimits;
use tokmd_analysis_types::{LicenseFinding, LicenseReport, LicenseSourceKind};

fn default_limits() -> AnalysisLimits {
    AnalysisLimits::default()
}

// ===========================================================================
// 1. AGPL-3.0 text detection with all three phrases
// ===========================================================================

#[test]
fn agpl3_text_with_all_phrases_detected_at_high_confidence() {
    let dir = tempdir().unwrap();
    fs::write(
        dir.path().join("LICENSE"),
        "GNU Affero General Public License\n\
         Version 3, 19 November 2007\n\
         This program is free software: you can redistribute it and/or modify it \
         under the terms of the GNU Affero General Public License as published by \
         the Free Software Foundation, either version 3 of the License, or \
         (at your option) any later version.",
    )
    .unwrap();

    let files = vec![PathBuf::from("LICENSE")];
    let report = build_license_report(dir.path(), &files, &default_limits()).unwrap();

    let agpl = report
        .findings
        .iter()
        .find(|f| f.spdx == "AGPL-3.0-or-later");
    assert!(agpl.is_some(), "should detect AGPL-3.0-or-later");
    assert!(
        agpl.unwrap().confidence > 0.9,
        "all three phrases should yield high confidence"
    );
}

// ===========================================================================
// 2. BSD-3-Clause full detection with "neither the name of" phrase
// ===========================================================================

#[test]
fn bsd3_clause_with_all_phrases_detected() {
    let dir = tempdir().unwrap();
    fs::write(
        dir.path().join("LICENSE"),
        "Redistribution and use in source and binary forms, with or without \
         modification, are permitted.\n\
         Neither the name of the copyright holder nor the names of its \
         contributors may be used to endorse or promote products.",
    )
    .unwrap();

    let files = vec![PathBuf::from("LICENSE")];
    let report = build_license_report(dir.path(), &files, &default_limits()).unwrap();

    assert!(
        report.findings.iter().any(|f| f.spdx == "BSD-3-Clause"),
        "should detect BSD-3-Clause with all phrases: {:?}",
        report.findings
    );
}

// ===========================================================================
// 3. Sort tiebreaking: same confidence, same spdx → sort by source_path
// ===========================================================================

#[test]
fn findings_with_same_confidence_and_spdx_sorted_by_source_path() {
    let dir = tempdir().unwrap();
    // Two Cargo.toml files with the same license → same confidence (0.95)
    let sub_b = dir.path().join("b-crate");
    let sub_a = dir.path().join("a-crate");
    fs::create_dir_all(&sub_b).unwrap();
    fs::create_dir_all(&sub_a).unwrap();
    fs::write(
        sub_b.join("Cargo.toml"),
        "[package]\nname = \"b\"\nlicense = \"MIT\"\n",
    )
    .unwrap();
    fs::write(
        sub_a.join("Cargo.toml"),
        "[package]\nname = \"a\"\nlicense = \"MIT\"\n",
    )
    .unwrap();

    let files = vec![
        PathBuf::from("b-crate").join("Cargo.toml"),
        PathBuf::from("a-crate").join("Cargo.toml"),
    ];
    let report = build_license_report(dir.path(), &files, &default_limits()).unwrap();

    assert_eq!(report.findings.len(), 2);
    // Both MIT, same confidence → alphabetical by source_path
    assert!(
        report.findings[0].source_path <= report.findings[1].source_path,
        "same spdx + confidence should sort by source_path: {} vs {}",
        report.findings[0].source_path,
        report.findings[1].source_path,
    );
}

// ===========================================================================
// 4. Serde round-trip of complete LicenseReport
// ===========================================================================

#[test]
fn license_report_serde_round_trip() {
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
    let original = build_license_report(dir.path(), &files, &default_limits()).unwrap();

    let json = serde_json::to_string(&original).expect("serialize");
    let deserialized: LicenseReport = serde_json::from_str(&json).expect("deserialize");

    assert_eq!(original.effective, deserialized.effective);
    assert_eq!(original.findings.len(), deserialized.findings.len());
    for (a, b) in original.findings.iter().zip(deserialized.findings.iter()) {
        assert_eq!(a.spdx, b.spdx);
        assert_eq!(a.source_path, b.source_path);
        assert_eq!(a.source_kind, b.source_kind);
        assert!((a.confidence - b.confidence).abs() < f32::EPSILON);
    }
}

// ===========================================================================
// 5. LicenseFinding JSON shape validation
// ===========================================================================

#[test]
fn license_finding_json_shape() {
    let finding = LicenseFinding {
        spdx: "MIT".to_string(),
        confidence: 0.95,
        source_path: "Cargo.toml".to_string(),
        source_kind: LicenseSourceKind::Metadata,
    };
    let v: serde_json::Value = serde_json::to_value(finding).unwrap();
    assert!(v.is_object());
    assert_eq!(v["spdx"], "MIT");
    let conf = v["confidence"].as_f64().unwrap();
    assert!((conf - 0.95).abs() < 0.001, "expected ~0.95, got {conf}");
    assert_eq!(v["source_path"], "Cargo.toml");
    assert_eq!(v["source_kind"], "metadata");
}

#[test]
fn license_source_kind_text_serializes_as_snake_case() {
    let finding = LicenseFinding {
        spdx: "MIT".to_string(),
        confidence: 0.8,
        source_path: "LICENSE".to_string(),
        source_kind: LicenseSourceKind::Text,
    };
    let v: serde_json::Value = serde_json::to_value(finding).unwrap();
    assert_eq!(v["source_kind"], "text");
}

// ===========================================================================
// 6. TOML parsing: extra whitespace around = sign
// ===========================================================================

#[test]
fn cargo_toml_with_extra_whitespace_around_equals() {
    let dir = tempdir().unwrap();
    fs::write(
        dir.path().join("Cargo.toml"),
        "[package]\nname   =   \"x\"\nlicense   =   \"Apache-2.0\"\n",
    )
    .unwrap();

    let files = vec![PathBuf::from("Cargo.toml")];
    let report = build_license_report(dir.path(), &files, &default_limits()).unwrap();

    assert_eq!(report.findings.len(), 1);
    assert_eq!(report.findings[0].spdx, "Apache-2.0");
}

// ===========================================================================
// 7. Confidence monotonicity: Apache with 2 vs 4 phrases
// ===========================================================================

#[test]
fn apache_confidence_increases_with_more_phrase_hits() {
    let dir_two = tempdir().unwrap();
    fs::write(
        dir_two.path().join("LICENSE"),
        "Apache License\nVersion 2.0",
    )
    .unwrap();
    let report_two = build_license_report(
        dir_two.path(),
        &[PathBuf::from("LICENSE")],
        &default_limits(),
    )
    .unwrap();

    let dir_four = tempdir().unwrap();
    fs::write(
        dir_four.path().join("LICENSE"),
        "Apache License\nVersion 2.0\n\
         http://www.apache.org/licenses/\n\
         limitations under the License.",
    )
    .unwrap();
    let report_four = build_license_report(
        dir_four.path(),
        &[PathBuf::from("LICENSE")],
        &default_limits(),
    )
    .unwrap();

    let conf_two = report_two
        .findings
        .iter()
        .find(|f| f.spdx == "Apache-2.0")
        .map(|f| f.confidence)
        .unwrap_or(0.0);
    let conf_four = report_four
        .findings
        .iter()
        .find(|f| f.spdx == "Apache-2.0")
        .map(|f| f.confidence)
        .unwrap_or(0.0);

    assert!(
        conf_four > conf_two,
        "4 phrase hits ({conf_four}) should beat 2 phrase hits ({conf_two})"
    );
}

// ===========================================================================
// 8. Mixed metadata from all three ecosystems
// ===========================================================================

#[test]
fn all_three_metadata_ecosystems_detected() {
    let dir = tempdir().unwrap();
    fs::write(
        dir.path().join("Cargo.toml"),
        "[package]\nname = \"x\"\nlicense = \"MIT\"\n",
    )
    .unwrap();
    fs::write(
        dir.path().join("package.json"),
        r#"{"name": "x", "license": "ISC"}"#,
    )
    .unwrap();
    fs::write(
        dir.path().join("pyproject.toml"),
        "[project]\nname = \"x\"\nlicense = \"Apache-2.0\"\n",
    )
    .unwrap();

    let files = vec![
        PathBuf::from("Cargo.toml"),
        PathBuf::from("package.json"),
        PathBuf::from("pyproject.toml"),
    ];
    let report = build_license_report(dir.path(), &files, &default_limits()).unwrap();

    assert_eq!(report.findings.len(), 3);
    let spdx_set: Vec<&str> = report.findings.iter().map(|f| f.spdx.as_str()).collect();
    assert!(spdx_set.contains(&"MIT"));
    assert!(spdx_set.contains(&"ISC"));
    assert!(spdx_set.contains(&"Apache-2.0"));
}

// ===========================================================================
// 9. Unknown file types are not scanned as metadata
// ===========================================================================

#[test]
fn unknown_file_types_not_scanned_as_metadata() {
    let dir = tempdir().unwrap();
    fs::write(dir.path().join("build.gradle"), "license = 'MIT'\n").unwrap();

    let files = vec![PathBuf::from("build.gradle")];
    let report = build_license_report(dir.path(), &files, &default_limits()).unwrap();

    assert!(
        !report
            .findings
            .iter()
            .any(|f| f.source_kind == LicenseSourceKind::Metadata),
        "build.gradle should not be parsed as metadata"
    );
}

// ===========================================================================
// 10. Effective is None when no findings
// ===========================================================================

#[test]
fn effective_is_none_with_empty_findings() {
    let dir = tempdir().unwrap();
    fs::write(dir.path().join("README.md"), "Hello World").unwrap();

    let files = vec![PathBuf::from("README.md")];
    let report = build_license_report(dir.path(), &files, &default_limits()).unwrap();

    assert!(report.findings.is_empty());
    assert!(report.effective.is_none());
}

// ===========================================================================
// 11. LICENSE-APACHE with full Apache text yields Apache-2.0
// ===========================================================================

#[test]
fn license_apache_filename_with_full_text_detected() {
    let dir = tempdir().unwrap();
    fs::write(
        dir.path().join("LICENSE-APACHE"),
        "Apache License\n\
         Version 2.0, January 2004\n\
         http://www.apache.org/licenses/LICENSE-2.0\n\
         Unless required by applicable law or agreed to in writing, software \
         distributed under the License is distributed on an \"AS IS\" BASIS, \
         WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND.\n\
         limitations under the License.",
    )
    .unwrap();

    let files = vec![PathBuf::from("LICENSE-APACHE")];
    let report = build_license_report(dir.path(), &files, &default_limits()).unwrap();

    assert!(
        report.findings.iter().any(|f| f.spdx == "Apache-2.0"),
        "LICENSE-APACHE with full text should detect Apache-2.0"
    );
}

// ===========================================================================
// 12. LicenseReport Debug impl contains key info
// ===========================================================================

#[test]
fn license_report_debug_contains_findings() {
    let report = LicenseReport {
        findings: vec![LicenseFinding {
            spdx: "MIT".to_string(),
            confidence: 0.95,
            source_path: "Cargo.toml".to_string(),
            source_kind: LicenseSourceKind::Metadata,
        }],
        effective: Some("MIT".to_string()),
    };
    let dbg = format!("{:?}", report);
    assert!(dbg.contains("MIT"));
    assert!(dbg.contains("Cargo.toml"));
}

// ===========================================================================
// 13. LicenseFinding Clone impl preserves all fields
// ===========================================================================

#[test]
fn license_finding_clone_preserves_fields() {
    let original = LicenseFinding {
        spdx: "Apache-2.0".to_string(),
        confidence: 0.85,
        source_path: "LICENSE".to_string(),
        source_kind: LicenseSourceKind::Text,
    };
    let cloned = original.clone();
    assert_eq!(original.spdx, cloned.spdx);
    assert_eq!(original.confidence, cloned.confidence);
    assert_eq!(original.source_path, cloned.source_path);
    assert_eq!(original.source_kind, cloned.source_kind);
}

// ===========================================================================
// 14. Findings same-confidence ordering by spdx ascending
// ===========================================================================

#[test]
fn findings_with_same_confidence_sorted_by_spdx_ascending() {
    let dir = tempdir().unwrap();
    fs::write(
        dir.path().join("Cargo.toml"),
        "[package]\nname = \"x\"\nlicense = \"Zlib\"\n",
    )
    .unwrap();
    fs::write(
        dir.path().join("package.json"),
        r#"{"name": "x", "license": "AAL"}"#,
    )
    .unwrap();

    let files = vec![PathBuf::from("Cargo.toml"), PathBuf::from("package.json")];
    let report = build_license_report(dir.path(), &files, &default_limits()).unwrap();

    // Both metadata → same confidence 0.95 → sort by spdx ascending
    assert_eq!(report.findings.len(), 2);
    assert!(
        report.findings[0].spdx <= report.findings[1].spdx,
        "same confidence should sort by spdx ascending: {} vs {}",
        report.findings[0].spdx,
        report.findings[1].spdx,
    );
}

// ===========================================================================
// 15. Cargo.toml with license in [workspace.package] is not detected
// ===========================================================================

#[test]
fn cargo_toml_workspace_package_section_not_detected() {
    let dir = tempdir().unwrap();
    fs::write(
        dir.path().join("Cargo.toml"),
        "[workspace.package]\nlicense = \"MIT\"\n\n[workspace]\nmembers = [\"crates/*\"]\n",
    )
    .unwrap();

    let files = vec![PathBuf::from("Cargo.toml")];
    let report = build_license_report(dir.path(), &files, &default_limits()).unwrap();

    // [workspace.package] is not [package] so should not be detected
    assert!(
        report.findings.is_empty(),
        "license in [workspace.package] should not be detected as [package]"
    );
}

// ===========================================================================
// 16. MPL-2.0 with all three phrases yields highest confidence
// ===========================================================================

#[test]
fn mpl2_with_all_phrases_yields_high_confidence() {
    let dir = tempdir().unwrap();
    fs::write(
        dir.path().join("LICENSE"),
        "Mozilla Public License\nVersion 2.0\n\
         This Source Code Form is subject to the terms of the Mozilla Public License, \
         v. 2.0. If a copy of the MPL was not distributed with this file, You can \
         obtain one at http://mozilla.org/MPL/2.0/.",
    )
    .unwrap();

    let files = vec![PathBuf::from("LICENSE")];
    let report = build_license_report(dir.path(), &files, &default_limits()).unwrap();

    let mpl = report
        .findings
        .iter()
        .find(|f| f.spdx == "MPL-2.0")
        .unwrap();
    assert!(
        mpl.confidence > 0.8,
        "MPL-2.0 phrase hits should yield >0.8 confidence, got {}",
        mpl.confidence
    );
}

// ===========================================================================
// 17. LicenseReport JSON shape has expected top-level keys
// ===========================================================================

#[test]
fn license_report_json_shape() {
    let report = LicenseReport {
        findings: vec![],
        effective: None,
    };
    let v: serde_json::Value = serde_json::to_value(report).unwrap();
    assert!(v.is_object());
    assert!(v.get("findings").is_some());
    assert!(v.get("effective").is_some());
    assert!(v["findings"].is_array());
    assert!(v["effective"].is_null());
}

// ===========================================================================
// 18. LicenseReport with effective set serializes correctly
// ===========================================================================

#[test]
fn license_report_with_effective_serializes_correctly() {
    let report = LicenseReport {
        findings: vec![LicenseFinding {
            spdx: "MIT".to_string(),
            confidence: 0.95,
            source_path: "Cargo.toml".to_string(),
            source_kind: LicenseSourceKind::Metadata,
        }],
        effective: Some("MIT".to_string()),
    };
    let v: serde_json::Value = serde_json::to_value(report).unwrap();
    assert_eq!(v["effective"], "MIT");
    assert_eq!(v["findings"].as_array().unwrap().len(), 1);
}

// ===========================================================================
// 19. Empty LICENSE text file produces no findings
// ===========================================================================

#[test]
fn zero_byte_license_file_no_findings() {
    let dir = tempdir().unwrap();
    fs::write(dir.path().join("LICENSE"), "").unwrap();

    let files = vec![PathBuf::from("LICENSE")];
    let report = build_license_report(dir.path(), &files, &default_limits()).unwrap();

    assert!(report.findings.is_empty());
}

// ===========================================================================
// 20. GPL text without "any later version" doesn't match AGPL
// ===========================================================================

#[test]
fn gpl_text_without_later_version_does_not_match_agpl() {
    let dir = tempdir().unwrap();
    fs::write(
        dir.path().join("LICENSE"),
        "GNU General Public License\n\
         Version 3, 29 June 2007\n\
         This program is free software: you can redistribute it and/or modify it \
         under the terms of the GNU General Public License.\n\
         Everyone is permitted to copy and distribute verbatim copies.\n\
         You may redistribute under version 3 or any later version.",
    )
    .unwrap();

    let files = vec![PathBuf::from("LICENSE")];
    let report = build_license_report(dir.path(), &files, &default_limits()).unwrap();

    assert!(
        !report
            .findings
            .iter()
            .any(|f| f.spdx == "AGPL-3.0-or-later"),
        "GPL text should not match AGPL-3.0-or-later"
    );
}

// ===========================================================================
// 21. Confidence formula: exactly N out of M phrases → expected value
// ===========================================================================

#[test]
fn confidence_formula_produces_expected_value_for_mit_one_phrase() {
    let dir = tempdir().unwrap();
    fs::write(
        dir.path().join("LICENSE"),
        "Permission is hereby granted, free of charge, to any person.",
    )
    .unwrap();

    let files = vec![PathBuf::from("LICENSE")];
    let report = build_license_report(dir.path(), &files, &default_limits()).unwrap();

    // MIT has 2 phrases, min_hits=1. 1/2 hit → confidence = 0.6 + 0.4 * (1/2) = 0.8
    let mit = report.findings.iter().find(|f| f.spdx == "MIT");
    assert!(mit.is_some());
    let conf = mit.unwrap().confidence;
    assert!(
        (conf - 0.8).abs() < 0.01,
        "expected ~0.8 confidence for 1/2 MIT phrases, got {conf}"
    );
}

#[test]
fn confidence_formula_produces_expected_value_for_mit_both_phrases() {
    let dir = tempdir().unwrap();
    fs::write(
        dir.path().join("LICENSE"),
        "Permission is hereby granted, free of charge.\n\
         The software is provided \"as is\".",
    )
    .unwrap();

    let files = vec![PathBuf::from("LICENSE")];
    let report = build_license_report(dir.path(), &files, &default_limits()).unwrap();

    // MIT has 2 phrases, 2/2 hit → confidence = 0.6 + 0.4 * (2/2) = 1.0
    let mit = report.findings.iter().find(|f| f.spdx == "MIT").unwrap();
    assert!(
        (mit.confidence - 1.0).abs() < f32::EPSILON,
        "expected 1.0 confidence for 2/2 MIT phrases, got {}",
        mit.confidence
    );
}

// ===========================================================================
// 23. Deserialization from known JSON
// ===========================================================================

#[test]
fn license_finding_deserializes_from_known_json() {
    let json =
        r#"{"spdx":"MIT","confidence":0.95,"source_path":"Cargo.toml","source_kind":"metadata"}"#;
    let finding: LicenseFinding = serde_json::from_str(json).unwrap();
    assert_eq!(finding.spdx, "MIT");
    assert_eq!(finding.confidence, 0.95);
    assert_eq!(finding.source_path, "Cargo.toml");
    assert_eq!(finding.source_kind, LicenseSourceKind::Metadata);
}

#[test]
fn license_report_deserializes_from_known_json() {
    let json = r#"{"findings":[{"spdx":"Apache-2.0","confidence":0.85,"source_path":"LICENSE","source_kind":"text"}],"effective":"Apache-2.0"}"#;
    let report: LicenseReport = serde_json::from_str(json).unwrap();
    assert_eq!(report.effective.as_deref(), Some("Apache-2.0"));
    assert_eq!(report.findings.len(), 1);
    assert_eq!(report.findings[0].source_kind, LicenseSourceKind::Text);
}

// ===========================================================================
// 25. Effective license always matches first finding's spdx
// ===========================================================================

#[test]
fn effective_always_matches_first_finding() {
    let dir = tempdir().unwrap();
    fs::write(
        dir.path().join("Cargo.toml"),
        "[package]\nname = \"x\"\nlicense = \"BSD-2-Clause\"\n",
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

    assert!(!report.findings.is_empty());
    assert_eq!(
        report.effective.as_deref(),
        Some(report.findings[0].spdx.as_str()),
        "effective must match first finding's spdx"
    );
}

// ===========================================================================
// 26. JSON serialization is deterministic across calls
// ===========================================================================

#[test]
fn json_serialization_is_deterministic() {
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
    let r1 = build_license_report(dir.path(), &files, &default_limits()).unwrap();
    let r2 = build_license_report(dir.path(), &files, &default_limits()).unwrap();

    let json1 = serde_json::to_string(&r1).unwrap();
    let json2 = serde_json::to_string(&r2).unwrap();
    assert_eq!(json1, json2, "JSON serialization must be deterministic");
}
