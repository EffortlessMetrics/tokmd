//! Snapshot, determinism, and round-trip serialization tests for assets and dependency reports.

use std::path::{Path, PathBuf};

use tempfile::TempDir;
use tokmd_analysis_assets::{build_assets_report, build_dependency_report};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn write_file(dir: &Path, rel: &str, content: &[u8]) -> PathBuf {
    let full = dir.join(rel);
    if let Some(parent) = full.parent() {
        std::fs::create_dir_all(parent).unwrap();
    }
    std::fs::write(&full, content).unwrap();
    PathBuf::from(rel)
}

// ===========================================================================
// Insta snapshot tests – asset report
// ===========================================================================

#[test]
fn snapshot_asset_report_mixed_categories() {
    let tmp = TempDir::new().unwrap();
    let files = vec![
        write_file(tmp.path(), "logo.png", &[0u8; 128]),
        write_file(tmp.path(), "icon.svg", &[0u8; 64]),
        write_file(tmp.path(), "intro.mp4", &[0u8; 2048]),
        write_file(tmp.path(), "theme.mp3", &[0u8; 512]),
        write_file(tmp.path(), "bundle.zip", &[0u8; 1024]),
        write_file(tmp.path(), "app.exe", &[0u8; 256]),
        write_file(tmp.path(), "body.woff2", &[0u8; 96]),
    ];
    let report = build_assets_report(tmp.path(), &files).unwrap();
    insta::assert_json_snapshot!("asset_report_mixed_categories", report);
}

#[test]
fn snapshot_asset_report_empty() {
    let tmp = TempDir::new().unwrap();
    let report = build_assets_report(tmp.path(), &[]).unwrap();
    insta::assert_json_snapshot!("asset_report_empty", report);
}

#[test]
fn snapshot_dependency_report_cargo_lock() {
    let tmp = TempDir::new().unwrap();
    let content = "[[package]]\nname = \"serde\"\n\n[[package]]\nname = \"anyhow\"\n";
    let rel = write_file(tmp.path(), "Cargo.lock", content.as_bytes());
    let report = build_dependency_report(tmp.path(), &[rel]).unwrap();
    insta::assert_json_snapshot!("dependency_report_cargo_lock", report);
}

#[test]
fn snapshot_dependency_report_empty() {
    let tmp = TempDir::new().unwrap();
    let report = build_dependency_report(tmp.path(), &[]).unwrap();
    insta::assert_json_snapshot!("dependency_report_empty", report);
}

// ===========================================================================
// Determinism tests
// ===========================================================================

#[test]
fn given_same_input_when_asset_report_built_twice_then_json_is_identical() {
    let tmp = TempDir::new().unwrap();
    let files = vec![
        write_file(tmp.path(), "a.png", &[0u8; 100]),
        write_file(tmp.path(), "b.mp4", &[0u8; 200]),
        write_file(tmp.path(), "c.zip", &[0u8; 300]),
    ];

    let r1 = build_assets_report(tmp.path(), &files).unwrap();
    let r2 = build_assets_report(tmp.path(), &files).unwrap();

    let j1 = serde_json::to_string_pretty(&r1).unwrap();
    let j2 = serde_json::to_string_pretty(&r2).unwrap();
    assert_eq!(j1, j2, "asset report JSON must be deterministic");
}

#[test]
fn given_same_input_when_dependency_report_built_twice_then_json_is_identical() {
    let tmp = TempDir::new().unwrap();
    let cargo = write_file(
        tmp.path(),
        "Cargo.lock",
        b"[[package]]\nname = \"a\"\n\n[[package]]\nname = \"b\"\n",
    );

    let r1 = build_dependency_report(tmp.path(), &[cargo.clone()]).unwrap();
    let r2 = build_dependency_report(tmp.path(), &[cargo]).unwrap();

    let j1 = serde_json::to_string_pretty(&r1).unwrap();
    let j2 = serde_json::to_string_pretty(&r2).unwrap();
    assert_eq!(j1, j2, "dependency report JSON must be deterministic");
}

// ===========================================================================
// Round-trip serialization
// ===========================================================================

#[test]
fn asset_report_round_trip_serialization() {
    let tmp = TempDir::new().unwrap();
    let files = vec![
        write_file(tmp.path(), "photo.jpg", &[0u8; 500]),
        write_file(tmp.path(), "clip.avi", &[0u8; 1000]),
        write_file(tmp.path(), "font.ttf", &[0u8; 200]),
    ];
    let report = build_assets_report(tmp.path(), &files).unwrap();

    let json = serde_json::to_string(&report).unwrap();
    let deserialized: tokmd_analysis_types::AssetReport = serde_json::from_str(&json).unwrap();
    let json2 = serde_json::to_string(&deserialized).unwrap();
    assert_eq!(json, json2, "round-trip serialization must be stable");
}

#[test]
fn dependency_report_round_trip_serialization() {
    let tmp = TempDir::new().unwrap();
    let rel = write_file(
        tmp.path(),
        "Cargo.lock",
        b"[[package]]\nname = \"serde\"\n\n[[package]]\nname = \"tokei\"\n",
    );
    let report = build_dependency_report(tmp.path(), &[rel]).unwrap();

    let json = serde_json::to_string(&report).unwrap();
    let deserialized: tokmd_analysis_types::DependencyReport = serde_json::from_str(&json).unwrap();
    let json2 = serde_json::to_string(&deserialized).unwrap();
    assert_eq!(json, json2, "round-trip serialization must be stable");
}

// ===========================================================================
// Additional edge cases
// ===========================================================================

#[test]
fn given_config_file_extensions_when_assets_report_built_then_not_classified() {
    let tmp = TempDir::new().unwrap();
    let files = vec![
        write_file(tmp.path(), "config.yaml", b"key: value"),
        write_file(tmp.path(), "settings.json", b"{}"),
        write_file(tmp.path(), "app.toml", b"[section]"),
        write_file(tmp.path(), "env.ini", b"KEY=val"),
        write_file(tmp.path(), "config.xml", b"<root/>"),
    ];
    let report = build_assets_report(tmp.path(), &files).unwrap();

    assert_eq!(
        report.total_files, 0,
        "config files should not be classified as assets"
    );
}

#[test]
fn given_empty_directory_when_assets_report_built_then_no_assets() {
    let tmp = TempDir::new().unwrap();
    let report = build_assets_report(tmp.path(), &[]).unwrap();

    assert_eq!(report.total_files, 0);
    assert_eq!(report.total_bytes, 0);
    assert!(report.categories.is_empty());
    assert!(report.top_files.is_empty());
}

#[test]
fn given_duplicate_extensions_when_assets_report_built_then_counted_correctly() {
    let tmp = TempDir::new().unwrap();
    let files = vec![
        write_file(tmp.path(), "a.png", &[0u8; 50]),
        write_file(tmp.path(), "b.png", &[0u8; 75]),
        write_file(tmp.path(), "c.png", &[0u8; 100]),
    ];
    let report = build_assets_report(tmp.path(), &files).unwrap();

    assert_eq!(report.total_files, 3);
    assert_eq!(report.total_bytes, 225);
    assert_eq!(report.categories.len(), 1);
    assert_eq!(report.categories[0].files, 3);
    assert_eq!(report.categories[0].extensions, vec!["png"]);
}

#[test]
fn given_valid_asset_report_when_serialized_then_non_empty_json() {
    let tmp = TempDir::new().unwrap();
    let files = vec![write_file(tmp.path(), "logo.png", &[0u8; 64])];
    let report = build_assets_report(tmp.path(), &files).unwrap();

    let json = serde_json::to_string(&report).unwrap();
    assert!(!json.is_empty());
    assert!(json.contains("total_files"));
    assert!(json.contains("categories"));
    assert!(json.contains("image"));
}

#[test]
fn given_valid_dependency_report_when_serialized_then_non_empty_json() {
    let tmp = TempDir::new().unwrap();
    let rel = write_file(tmp.path(), "Cargo.lock", b"[[package]]\nname = \"a\"\n");
    let report = build_dependency_report(tmp.path(), &[rel]).unwrap();

    let json = serde_json::to_string(&report).unwrap();
    assert!(!json.is_empty());
    assert!(json.contains("total"));
    assert!(json.contains("lockfiles"));
    assert!(json.contains("cargo"));
}
