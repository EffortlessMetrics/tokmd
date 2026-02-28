use proptest::prelude::*;
use std::fs;
use std::path::PathBuf;
use tempfile::tempdir;
use tokmd_analysis_license::build_license_report;
use tokmd_analysis_util::AnalysisLimits;

fn default_limits() -> AnalysisLimits {
    AnalysisLimits::default()
}

// ---------------------------------------------------------------------------
// Strategy: arbitrary SPDX-like identifiers
// ---------------------------------------------------------------------------

fn spdx_id_strategy() -> impl Strategy<Value = String> {
    prop::sample::select(vec![
        "MIT".to_string(),
        "Apache-2.0".to_string(),
        "GPL-3.0-only".to_string(),
        "BSD-2-Clause".to_string(),
        "ISC".to_string(),
        "LGPL-2.1-or-later".to_string(),
        "MPL-2.0".to_string(),
        "Unlicense".to_string(),
        "WTFPL".to_string(),
        "0BSD".to_string(),
    ])
}

// ---------------------------------------------------------------------------
// Property: Any SPDX id in Cargo.toml metadata is returned verbatim
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn metadata_spdx_round_trips(spdx in spdx_id_strategy()) {
        let dir = tempdir().unwrap();
        let cargo = dir.path().join("Cargo.toml");
        fs::write(
            &cargo,
            format!("[package]\nname = \"test\"\nlicense = \"{spdx}\"\n"),
        )
        .unwrap();

        let files = vec![PathBuf::from("Cargo.toml")];
        let report = build_license_report(dir.path(), &files, &default_limits()).unwrap();

        prop_assert!(!report.findings.is_empty(), "should find the license");
        prop_assert_eq!(&report.findings[0].spdx, &spdx);
    }
}

// ---------------------------------------------------------------------------
// Property: Empty file list always produces empty report
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn empty_file_list_produces_empty_report(_seed in 0u64..1000) {
        let dir = tempdir().unwrap();
        let report = build_license_report(dir.path(), &[], &default_limits()).unwrap();

        prop_assert!(report.findings.is_empty());
        prop_assert!(report.effective.is_none());
    }
}

// ---------------------------------------------------------------------------
// Property: Findings are always sorted by confidence descending
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn findings_always_sorted_descending(spdx in spdx_id_strategy()) {
        let dir = tempdir().unwrap();
        fs::write(
            dir.path().join("Cargo.toml"),
            format!("[package]\nname = \"test\"\nlicense = \"{spdx}\"\n"),
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
            prop_assert!(
                pair[0].confidence >= pair[1].confidence,
                "not sorted: {} >= {} failed",
                pair[0].confidence,
                pair[1].confidence,
            );
        }
    }
}

// ---------------------------------------------------------------------------
// Property: Effective license always matches first finding
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn effective_matches_first_finding(spdx in spdx_id_strategy()) {
        let dir = tempdir().unwrap();
        fs::write(
            dir.path().join("Cargo.toml"),
            format!("[package]\nname = \"test\"\nlicense = \"{spdx}\"\n"),
        )
        .unwrap();

        let files = vec![PathBuf::from("Cargo.toml")];
        let report = build_license_report(dir.path(), &files, &default_limits()).unwrap();

        let expected = report.findings.first().map(|f| f.spdx.clone());
        prop_assert_eq!(report.effective, expected);
    }
}

// ---------------------------------------------------------------------------
// Property: Confidence is always in [0.0, 1.0]
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn confidence_always_in_range(spdx in spdx_id_strategy()) {
        let dir = tempdir().unwrap();
        fs::write(
            dir.path().join("Cargo.toml"),
            format!("[package]\nname = \"test\"\nlicense = \"{spdx}\"\n"),
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
            prop_assert!(f.confidence >= 0.0 && f.confidence <= 1.0,
                "confidence out of range: {}", f.confidence);
        }
    }
}

// ---------------------------------------------------------------------------
// Property: Source paths never contain backslashes
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn source_paths_never_contain_backslash(spdx in spdx_id_strategy()) {
        let dir = tempdir().unwrap();
        let sub = dir.path().join("sub");
        fs::create_dir_all(&sub).unwrap();
        fs::write(
            sub.join("Cargo.toml"),
            format!("[package]\nname = \"test\"\nlicense = \"{spdx}\"\n"),
        )
        .unwrap();

        let files = vec![PathBuf::from("sub").join("Cargo.toml")];
        let report = build_license_report(dir.path(), &files, &default_limits()).unwrap();

        for f in &report.findings {
            prop_assert!(!f.source_path.contains('\\'),
                "path contains backslash: {}", f.source_path);
        }
    }
}

// ---------------------------------------------------------------------------
// Property: package.json license string round-trips
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn package_json_license_round_trips(spdx in spdx_id_strategy()) {
        let dir = tempdir().unwrap();
        fs::write(
            dir.path().join("package.json"),
            format!(r#"{{"name": "test", "license": "{spdx}"}}"#),
        )
        .unwrap();

        let files = vec![PathBuf::from("package.json")];
        let report = build_license_report(dir.path(), &files, &default_limits()).unwrap();

        prop_assert!(!report.findings.is_empty(), "should find the license");
        prop_assert_eq!(&report.findings[0].spdx, &spdx);
    }
}

// ---------------------------------------------------------------------------
// Property: pyproject.toml license round-trips
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn pyproject_license_round_trips(spdx in spdx_id_strategy()) {
        let dir = tempdir().unwrap();
        fs::write(
            dir.path().join("pyproject.toml"),
            format!("[project]\nname = \"test\"\nlicense = \"{spdx}\"\n"),
        )
        .unwrap();

        let files = vec![PathBuf::from("pyproject.toml")];
        let report = build_license_report(dir.path(), &files, &default_limits()).unwrap();

        prop_assert!(!report.findings.is_empty(), "should find the license");
        prop_assert_eq!(&report.findings[0].spdx, &spdx);
    }
}

// ---------------------------------------------------------------------------
// Property: Metadata confidence is always 0.95
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn metadata_confidence_is_fixed(spdx in spdx_id_strategy()) {
        let dir = tempdir().unwrap();
        fs::write(
            dir.path().join("Cargo.toml"),
            format!("[package]\nname = \"test\"\nlicense = \"{spdx}\"\n"),
        )
        .unwrap();

        let files = vec![PathBuf::from("Cargo.toml")];
        let report = build_license_report(dir.path(), &files, &default_limits()).unwrap();

        for f in &report.findings {
            if f.source_kind == tokmd_analysis_types::LicenseSourceKind::Metadata {
                let diff = (f.confidence - 0.95_f32).abs();
                prop_assert!(diff < f32::EPSILON, "metadata confidence should be 0.95, got {}", f.confidence);
            }
        }
    }
}
