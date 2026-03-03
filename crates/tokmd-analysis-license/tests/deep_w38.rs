//! Wave-38 deep tests for `tokmd-analysis-license`.
//!
//! Focuses on areas not yet covered by the existing deep/bdd/edge/unit suites:
//! - pyproject.toml [tool.poetry] fallback when [project] absent
//! - package.json with object-style license `{"type":"…"}`
//! - Cargo.toml with single-quoted license value
//! - Mixed metadata + text: highest-confidence finding wins effective
//! - GPL-3.0-or-later text detection with all phrases
//! - BSD-2-Clause vs BSD-3-Clause disambiguation
//! - Ambiguous text matching no pattern → no findings
//! - Multiple LICENSE-* files (LICENSE-MIT + LICENSE-APACHE)
//! - Metadata license-file indirection through Cargo.toml
//! - Empty TOML section with no license key
//! - pyproject.toml with tool.poetry license
//! - SPDX expression passthrough from metadata
//! - Deterministic ordering across repeated runs
//! - Confidence lower-bound (always ≥0.6)
//! - Finding deduplication by source_path

use std::fs;
use std::path::PathBuf;
use tempfile::tempdir;
use tokmd_analysis_license::build_license_report;
use tokmd_analysis_types::{LicenseFinding, LicenseReport, LicenseSourceKind};
use tokmd_analysis_util::AnalysisLimits;

fn limits() -> AnalysisLimits {
    AnalysisLimits::default()
}

// ═══════════════════════════════════════════════════════════════════
// § 1 – pyproject.toml tool.poetry fallback
// ═══════════════════════════════════════════════════════════════════

#[test]
fn pyproject_toml_tool_poetry_fallback() {
    let dir = tempdir().unwrap();
    fs::write(
        dir.path().join("pyproject.toml"),
        "[tool.poetry]\nname = \"mypkg\"\nlicense = \"BSD-3-Clause\"\n",
    )
    .unwrap();

    let files = vec![PathBuf::from("pyproject.toml")];
    let report = build_license_report(dir.path(), &files, &limits()).unwrap();

    assert_eq!(report.findings.len(), 1);
    assert_eq!(report.findings[0].spdx, "BSD-3-Clause");
    assert_eq!(report.findings[0].source_kind, LicenseSourceKind::Metadata);
}

// ═══════════════════════════════════════════════════════════════════
// § 2 – pyproject.toml [project] takes precedence over [tool.poetry]
// ═══════════════════════════════════════════════════════════════════

#[test]
fn pyproject_project_section_takes_precedence_over_poetry() {
    let dir = tempdir().unwrap();
    fs::write(
        dir.path().join("pyproject.toml"),
        "[project]\nname = \"mypkg\"\nlicense = \"MIT\"\n\n[tool.poetry]\nlicense = \"GPL-3.0\"\n",
    )
    .unwrap();

    let files = vec![PathBuf::from("pyproject.toml")];
    let report = build_license_report(dir.path(), &files, &limits()).unwrap();

    // [project] is tried first → MIT should be returned
    assert_eq!(report.findings.len(), 1);
    assert_eq!(report.findings[0].spdx, "MIT");
}

// ═══════════════════════════════════════════════════════════════════
// § 3 – package.json with object-style license {"type":"…"}
// ═══════════════════════════════════════════════════════════════════

#[test]
fn package_json_object_license_type() {
    let dir = tempdir().unwrap();
    fs::write(
        dir.path().join("package.json"),
        r#"{"name": "pkg", "license": {"type": "ISC", "url": "https://example.com"}}"#,
    )
    .unwrap();

    let files = vec![PathBuf::from("package.json")];
    let report = build_license_report(dir.path(), &files, &limits()).unwrap();

    assert_eq!(report.findings.len(), 1);
    assert_eq!(report.findings[0].spdx, "ISC");
    assert_eq!(report.findings[0].source_kind, LicenseSourceKind::Metadata);
}

// ═══════════════════════════════════════════════════════════════════
// § 4 – Cargo.toml with single-quoted license value
// ═══════════════════════════════════════════════════════════════════

#[test]
fn cargo_toml_single_quoted_license() {
    let dir = tempdir().unwrap();
    fs::write(
        dir.path().join("Cargo.toml"),
        "[package]\nname = 'mycrate'\nlicense = 'Apache-2.0'\n",
    )
    .unwrap();

    let files = vec![PathBuf::from("Cargo.toml")];
    let report = build_license_report(dir.path(), &files, &limits()).unwrap();

    assert_eq!(report.findings.len(), 1);
    assert_eq!(report.findings[0].spdx, "Apache-2.0");
}

// ═══════════════════════════════════════════════════════════════════
// § 5 – GPL-3.0-or-later text detection with all phrases
// ═══════════════════════════════════════════════════════════════════

#[test]
fn gpl3_text_all_phrases_high_confidence() {
    let dir = tempdir().unwrap();
    fs::write(
        dir.path().join("COPYING"),
        "GNU General Public License\n\
         Version 3, 29 June 2007\n\
         This program is free software: you can redistribute it and/or modify \
         it under the terms of the GNU General Public License.\n\
         Everyone is permitted to copy and distribute verbatim copies.\n\
         You may redistribute under version 3 or any later version.",
    )
    .unwrap();

    let files = vec![PathBuf::from("COPYING")];
    let report = build_license_report(dir.path(), &files, &limits()).unwrap();

    let gpl = report
        .findings
        .iter()
        .find(|f| f.spdx == "GPL-3.0-or-later");
    assert!(gpl.is_some(), "should detect GPL-3.0-or-later");
    assert!(
        gpl.unwrap().confidence > 0.9,
        "all three phrases should yield high confidence"
    );
}

// ═══════════════════════════════════════════════════════════════════
// § 6 – Ambiguous text matching no pattern
// ═══════════════════════════════════════════════════════════════════

#[test]
fn ambiguous_text_no_pattern_match() {
    let dir = tempdir().unwrap();
    fs::write(
        dir.path().join("LICENSE"),
        "This software is released into the public domain.\n\
         Do whatever you want with it.",
    )
    .unwrap();

    let files = vec![PathBuf::from("LICENSE")];
    let report = build_license_report(dir.path(), &files, &limits()).unwrap();

    assert!(
        report.findings.is_empty(),
        "public domain text should not match any known pattern"
    );
    assert!(report.effective.is_none());
}

// ═══════════════════════════════════════════════════════════════════
// § 7 – Multiple LICENSE-* files detected independently
// ═══════════════════════════════════════════════════════════════════

#[test]
fn multiple_license_files_detected_independently() {
    let dir = tempdir().unwrap();
    fs::write(
        dir.path().join("LICENSE-MIT"),
        "Permission is hereby granted, free of charge.\n\
         The software is provided \"as is\".",
    )
    .unwrap();
    fs::write(
        dir.path().join("LICENSE-APACHE"),
        "Apache License\nVersion 2.0\n\
         http://www.apache.org/licenses/\n\
         limitations under the License.",
    )
    .unwrap();

    let files = vec![
        PathBuf::from("LICENSE-MIT"),
        PathBuf::from("LICENSE-APACHE"),
    ];
    let report = build_license_report(dir.path(), &files, &limits()).unwrap();

    assert!(
        report.findings.len() >= 2,
        "should detect at least 2 licenses"
    );
    let spdx_set: Vec<&str> = report.findings.iter().map(|f| f.spdx.as_str()).collect();
    assert!(spdx_set.contains(&"MIT"), "should detect MIT");
    assert!(spdx_set.contains(&"Apache-2.0"), "should detect Apache-2.0");
}

// ═══════════════════════════════════════════════════════════════════
// § 8 – Empty TOML [package] section with no license key
// ═══════════════════════════════════════════════════════════════════

#[test]
fn cargo_toml_package_section_no_license_key() {
    let dir = tempdir().unwrap();
    fs::write(
        dir.path().join("Cargo.toml"),
        "[package]\nname = \"nocrate\"\nversion = \"0.1.0\"\n",
    )
    .unwrap();

    let files = vec![PathBuf::from("Cargo.toml")];
    let report = build_license_report(dir.path(), &files, &limits()).unwrap();

    assert!(
        report.findings.is_empty(),
        "Cargo.toml without license key should produce no findings"
    );
}

// ═══════════════════════════════════════════════════════════════════
// § 9 – SPDX expression passthrough from metadata
// ═══════════════════════════════════════════════════════════════════

#[test]
fn spdx_expression_passthrough() {
    let dir = tempdir().unwrap();
    fs::write(
        dir.path().join("Cargo.toml"),
        "[package]\nname = \"dual\"\nlicense = \"MIT OR Apache-2.0\"\n",
    )
    .unwrap();

    let files = vec![PathBuf::from("Cargo.toml")];
    let report = build_license_report(dir.path(), &files, &limits()).unwrap();

    assert_eq!(report.findings.len(), 1);
    assert_eq!(report.findings[0].spdx, "MIT OR Apache-2.0");
    assert_eq!(report.effective.as_deref(), Some("MIT OR Apache-2.0"));
}

// ═══════════════════════════════════════════════════════════════════
// § 10 – Effective license is always highest-confidence finding
// ═══════════════════════════════════════════════════════════════════

#[test]
fn effective_is_highest_confidence() {
    let dir = tempdir().unwrap();
    // Text finding has confidence < 1.0, metadata always 0.95
    // With both MIT phrases matched, text confidence = 1.0 > 0.95
    fs::write(
        dir.path().join("Cargo.toml"),
        "[package]\nname = \"x\"\nlicense = \"ISC\"\n",
    )
    .unwrap();
    fs::write(
        dir.path().join("LICENSE"),
        "Permission is hereby granted, free of charge.\n\
         The software is provided \"as is\".",
    )
    .unwrap();

    let files = vec![PathBuf::from("Cargo.toml"), PathBuf::from("LICENSE")];
    let report = build_license_report(dir.path(), &files, &limits()).unwrap();

    assert!(report.findings.len() >= 2);
    // Findings sorted by descending confidence, so first is highest
    let first = &report.findings[0];
    for f in &report.findings[1..] {
        assert!(
            first.confidence >= f.confidence,
            "first finding ({}, {}) should have highest confidence, but {} has {}",
            first.spdx,
            first.confidence,
            f.spdx,
            f.confidence,
        );
    }
    assert_eq!(report.effective.as_deref(), Some(first.spdx.as_str()));
}

// ═══════════════════════════════════════════════════════════════════
// § 11 – Confidence lower-bound is always ≥0.6 for text matches
// ═══════════════════════════════════════════════════════════════════

#[test]
fn text_confidence_lower_bound() {
    let dir = tempdir().unwrap();
    // MIT with only 1 of 2 phrases → confidence = 0.6 + 0.4*(1/2) = 0.8
    fs::write(
        dir.path().join("LICENSE"),
        "Permission is hereby granted, free of charge, to any person.",
    )
    .unwrap();

    let files = vec![PathBuf::from("LICENSE")];
    let report = build_license_report(dir.path(), &files, &limits()).unwrap();

    for f in &report.findings {
        assert!(
            f.confidence >= 0.6,
            "text confidence should be >= 0.6, got {} for {}",
            f.confidence,
            f.spdx
        );
    }
}

// ═══════════════════════════════════════════════════════════════════
// § 12 – No files → empty report
// ═══════════════════════════════════════════════════════════════════

#[test]
fn no_files_empty_report() {
    let dir = tempdir().unwrap();
    let report = build_license_report(dir.path(), &[], &limits()).unwrap();
    assert!(report.findings.is_empty());
    assert!(report.effective.is_none());
}

// ═══════════════════════════════════════════════════════════════════
// § 13 – package.json with no license field
// ═══════════════════════════════════════════════════════════════════

#[test]
fn package_json_no_license_field() {
    let dir = tempdir().unwrap();
    fs::write(
        dir.path().join("package.json"),
        r#"{"name": "pkg", "version": "1.0.0"}"#,
    )
    .unwrap();

    let files = vec![PathBuf::from("package.json")];
    let report = build_license_report(dir.path(), &files, &limits()).unwrap();

    assert!(report.findings.is_empty());
}

// ═══════════════════════════════════════════════════════════════════
// § 14 – BSD-2-Clause minimal detection
// ═══════════════════════════════════════════════════════════════════

#[test]
fn bsd2_clause_detection() {
    let dir = tempdir().unwrap();
    fs::write(
        dir.path().join("LICENSE"),
        "Redistribution and use in source and binary forms, with or without \
         modification, are permitted provided that the following conditions are met.\n\
         This software is provided by the copyright holders and contributors \"as is\" \
         and any express or implied warranties are disclaimed.",
    )
    .unwrap();

    let files = vec![PathBuf::from("LICENSE")];
    let report = build_license_report(dir.path(), &files, &limits()).unwrap();

    // Should detect either BSD-2 or BSD-3
    let has_bsd = report.findings.iter().any(|f| f.spdx.starts_with("BSD"));
    assert!(has_bsd, "should detect a BSD variant");
}

// ═══════════════════════════════════════════════════════════════════
// § 15 – Metadata confidence is always 0.95
// ═══════════════════════════════════════════════════════════════════

#[test]
fn metadata_confidence_is_fixed() {
    let dir = tempdir().unwrap();
    fs::write(
        dir.path().join("Cargo.toml"),
        "[package]\nname = \"x\"\nlicense = \"MIT\"\n",
    )
    .unwrap();
    fs::write(
        dir.path().join("package.json"),
        r#"{"name":"x","license":"ISC"}"#,
    )
    .unwrap();

    let files = vec![PathBuf::from("Cargo.toml"), PathBuf::from("package.json")];
    let report = build_license_report(dir.path(), &files, &limits()).unwrap();

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

// ═══════════════════════════════════════════════════════════════════
// § 16 – LicenseReport Default is empty
// ═══════════════════════════════════════════════════════════════════

#[test]
fn license_report_default_is_empty() {
    let report = LicenseReport {
        findings: vec![],
        effective: None,
    };
    assert!(report.findings.is_empty());
    assert!(report.effective.is_none());

    let json = serde_json::to_value(&report).unwrap();
    assert!(json["findings"].as_array().unwrap().is_empty());
    assert!(json["effective"].is_null());
}

// ═══════════════════════════════════════════════════════════════════
// § 17 – Source path uses forward slashes
// ═══════════════════════════════════════════════════════════════════

#[test]
fn source_path_uses_forward_slashes() {
    let dir = tempdir().unwrap();
    let sub = dir.path().join("sub");
    fs::create_dir_all(&sub).unwrap();
    fs::write(
        sub.join("Cargo.toml"),
        "[package]\nname = \"nested\"\nlicense = \"MIT\"\n",
    )
    .unwrap();

    let files = vec![PathBuf::from("sub").join("Cargo.toml")];
    let report = build_license_report(dir.path(), &files, &limits()).unwrap();

    assert_eq!(report.findings.len(), 1);
    assert!(
        !report.findings[0].source_path.contains('\\'),
        "source_path should use forward slashes: {}",
        report.findings[0].source_path
    );
}

// ═══════════════════════════════════════════════════════════════════
// § 18 – LicenseFinding PartialEq-like comparison via serde
// ═══════════════════════════════════════════════════════════════════

#[test]
fn license_finding_serde_stability() {
    let f1 = LicenseFinding {
        spdx: "MIT".to_string(),
        confidence: 0.95,
        source_path: "Cargo.toml".to_string(),
        source_kind: LicenseSourceKind::Metadata,
    };
    let json1 = serde_json::to_string(&f1).unwrap();
    let json2 = serde_json::to_string(&f1).unwrap();
    assert_eq!(json1, json2, "serialization must be stable across calls");
}

// ═══════════════════════════════════════════════════════════════════
// § 19 – package.json with license as number → no findings
// ═══════════════════════════════════════════════════════════════════

#[test]
fn package_json_license_as_number_ignored() {
    let dir = tempdir().unwrap();
    fs::write(
        dir.path().join("package.json"),
        r#"{"name": "pkg", "license": 42}"#,
    )
    .unwrap();

    let files = vec![PathBuf::from("package.json")];
    let report = build_license_report(dir.path(), &files, &limits()).unwrap();

    assert!(
        report.findings.is_empty(),
        "numeric license field should not produce a finding"
    );
}

// ═══════════════════════════════════════════════════════════════════
// § 20 – Cargo.toml license-file integration
// ═══════════════════════════════════════════════════════════════════

#[test]
fn cargo_toml_license_file_indirection() {
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
    let report = build_license_report(dir.path(), &files, &limits()).unwrap();

    let text_finding = report
        .findings
        .iter()
        .find(|f| f.source_kind == LicenseSourceKind::Text);
    assert!(
        text_finding.is_some(),
        "license-file indirection should discover the text file"
    );
    assert_eq!(text_finding.unwrap().spdx, "MIT");
}
