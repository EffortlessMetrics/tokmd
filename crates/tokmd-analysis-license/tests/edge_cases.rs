//! Edge-case BDD tests for license detection.

use std::fs;
use std::path::PathBuf;
use tempfile::tempdir;
use tokmd_analysis_license::build_license_report;
use tokmd_analysis_types::LicenseSourceKind;
use tokmd_analysis_util::AnalysisLimits;

fn default_limits() -> AnalysisLimits {
    AnalysisLimits::default()
}

// ── Scenario: pyproject.toml with both [project] and [tool.poetry] ──

#[test]
fn given_pyproject_with_both_sections_when_scanned_then_project_takes_precedence() {
    let dir = tempdir().unwrap();
    fs::write(
        dir.path().join("pyproject.toml"),
        "[project]\nname = \"x\"\nlicense = \"MIT\"\n\n[tool.poetry]\nname = \"x\"\nlicense = \"Apache-2.0\"\n",
    )
    .unwrap();

    let files = vec![PathBuf::from("pyproject.toml")];
    let report = build_license_report(dir.path(), &files, &default_limits()).unwrap();

    // [project] section should be found first
    assert!(!report.findings.is_empty());
    assert_eq!(report.findings[0].spdx, "MIT");
}

// ── Scenario: Cargo.toml license-file pointing to missing file ──────

#[test]
fn given_cargo_toml_license_file_to_missing_file_when_scanned_then_returns_error() {
    let dir = tempdir().unwrap();
    fs::write(
        dir.path().join("Cargo.toml"),
        "[package]\nname = \"x\"\nlicense-file = \"NONEXISTENT\"\n",
    )
    .unwrap();

    let files = vec![PathBuf::from("Cargo.toml")];
    // Missing referenced file propagates as an error
    let result = build_license_report(dir.path(), &files, &default_limits());
    assert!(
        result.is_err(),
        "missing license-file should propagate error"
    );
}

// ── Scenario: invalid JSON in package.json ──────────────────────────

#[test]
fn given_invalid_json_in_package_json_when_scanned_then_no_panic() {
    let dir = tempdir().unwrap();
    fs::write(dir.path().join("package.json"), "{ not valid json !!!").unwrap();

    let files = vec![PathBuf::from("package.json")];
    let report = build_license_report(dir.path(), &files, &default_limits()).unwrap();
    assert!(report.findings.is_empty());
}

// ── Scenario: license text with heavy whitespace ────────────────────

#[test]
fn given_mit_text_with_extra_whitespace_when_scanned_then_still_detected() {
    let dir = tempdir().unwrap();
    fs::write(
        dir.path().join("LICENSE"),
        "\n\n   Permission is hereby granted, free of charge   \n\n   The software is provided \"as is\"   \n\n",
    )
    .unwrap();

    let files = vec![PathBuf::from("LICENSE")];
    let report = build_license_report(dir.path(), &files, &default_limits()).unwrap();
    assert!(report.findings.iter().any(|f| f.spdx == "MIT"));
}

// ── Scenario: BSD-2-Clause vs BSD-3-Clause disambiguation ───────────

#[test]
fn given_bsd2_text_without_third_clause_when_scanned_then_matches_bsd() {
    let dir = tempdir().unwrap();
    // BSD-2-Clause: has redistribution phrase + "as is" but NOT "neither the name of"
    fs::write(
        dir.path().join("LICENSE"),
        "Redistribution and use in source and binary forms, with or without modification, \
         are permitted.\n\
         This software is provided by the copyright holders and contributors \"as is\".",
    )
    .unwrap();

    let files = vec![PathBuf::from("LICENSE")];
    let report = build_license_report(dir.path(), &files, &default_limits()).unwrap();
    assert!(!report.findings.is_empty());
    // Should match BSD-2-Clause (has the specific "as is" phrase)
    assert!(report.findings.iter().any(|f| f.spdx.starts_with("BSD")));
}

// ── Scenario: Cargo.toml in a subdirectory ──────────────────────────

#[test]
fn given_cargo_toml_in_subdirectory_when_scanned_then_detected_with_forward_slash_path() {
    let dir = tempdir().unwrap();
    let sub = dir.path().join("crates").join("my-crate");
    fs::create_dir_all(&sub).unwrap();
    fs::write(
        sub.join("Cargo.toml"),
        "[package]\nname = \"my-crate\"\nlicense = \"MIT\"\n",
    )
    .unwrap();

    let files = vec![PathBuf::from("crates").join("my-crate").join("Cargo.toml")];
    let report = build_license_report(dir.path(), &files, &default_limits()).unwrap();

    assert_eq!(report.findings.len(), 1);
    assert_eq!(report.findings[0].spdx, "MIT");
    assert!(
        report.findings[0].source_path.contains('/'),
        "should use forward slashes"
    );
    assert!(
        !report.findings[0].source_path.contains('\\'),
        "should not contain backslashes"
    );
}

// ── Scenario: empty SPDX identifier in metadata ─────────────────────

#[test]
fn given_cargo_toml_with_empty_license_string_when_scanned_then_no_finding() {
    let dir = tempdir().unwrap();
    fs::write(
        dir.path().join("Cargo.toml"),
        "[package]\nname = \"x\"\nlicense = \"\"\n",
    )
    .unwrap();

    let files = vec![PathBuf::from("Cargo.toml")];
    let report = build_license_report(dir.path(), &files, &default_limits()).unwrap();
    // Empty string should not produce a finding
    assert!(report.findings.is_empty());
}

// ── Scenario: only non-license files provided ───────────────────────

#[test]
fn given_only_source_files_when_scanned_then_no_findings() {
    let dir = tempdir().unwrap();
    fs::write(dir.path().join("main.rs"), "fn main() {}").unwrap();
    fs::write(dir.path().join("lib.rs"), "pub fn hello() {}").unwrap();

    let files = vec![PathBuf::from("main.rs"), PathBuf::from("lib.rs")];
    let report = build_license_report(dir.path(), &files, &default_limits()).unwrap();
    assert!(report.findings.is_empty());
    assert!(report.effective.is_none());
}

// ── Scenario: multiple LICENSE files with different licenses ─────────

#[test]
fn given_multiple_license_text_files_when_scanned_then_all_detected() {
    let dir = tempdir().unwrap();
    fs::write(
        dir.path().join("LICENSE-MIT"),
        "Permission is hereby granted, free of charge.\nThe software is provided \"as is\".",
    )
    .unwrap();
    fs::write(
        dir.path().join("LICENSE-APACHE"),
        "Apache License\nVersion 2.0\nhttp://www.apache.org/licenses/\nlimitations under the License.",
    )
    .unwrap();

    let files = vec![
        PathBuf::from("LICENSE-MIT"),
        PathBuf::from("LICENSE-APACHE"),
    ];
    let report = build_license_report(dir.path(), &files, &default_limits()).unwrap();

    assert!(report.findings.len() >= 2);
    let spdx_ids: Vec<&str> = report.findings.iter().map(|f| f.spdx.as_str()).collect();
    assert!(spdx_ids.contains(&"MIT"));
    assert!(spdx_ids.contains(&"Apache-2.0"));
}

// ── Scenario: effective license comes from highest confidence ────────

#[test]
fn given_metadata_and_text_when_scanned_then_effective_is_metadata() {
    let dir = tempdir().unwrap();
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

    // Metadata has 0.95 confidence, text is lower
    assert_eq!(report.effective.as_deref(), Some("MIT"));
    assert_eq!(report.findings[0].source_kind, LicenseSourceKind::Metadata);
}
