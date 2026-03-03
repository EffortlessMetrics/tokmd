//! Error boundary tests for tokmd-scan.
//!
//! Tests non-existent directories, empty directories, excluded patterns,
//! multiple paths, and edge cases in scan options.

use anyhow::Result;
use std::path::PathBuf;
use tokmd_scan::scan;
use tokmd_settings::ScanOptions;
use tokmd_types::ConfigMode;

fn default_opts() -> ScanOptions {
    ScanOptions {
        excluded: vec![],
        config: ConfigMode::None,
        hidden: false,
        no_ignore: false,
        no_ignore_parent: false,
        no_ignore_dot: false,
        no_ignore_vcs: false,
        treat_doc_strings_as_comments: false,
    }
}

// ── Non-existent directory ───────────────────────────────────────────

#[test]
fn scan_nonexistent_directory_returns_error() {
    let paths = vec![PathBuf::from(
        "/tmp/tokmd-scan-nonexistent-dir-abc123xyz789",
    )];
    let result = scan(&paths, &default_opts());
    assert!(result.is_err());
    let msg = result.unwrap_err().to_string();
    assert!(msg.contains("Path not found"), "got: {msg}");
}

#[test]
fn scan_nonexistent_among_valid_paths_returns_error() {
    let paths = vec![
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src"),
        PathBuf::from("/tmp/tokmd-scan-nonexistent-12345"),
    ];
    let result = scan(&paths, &default_opts());
    assert!(result.is_err(), "should fail on first nonexistent path");
}

// ── Empty directory ──────────────────────────────────────────────────

#[test]
fn scan_empty_directory_returns_empty_languages() -> Result<()> {
    let dir = tempfile::tempdir()?;
    let paths = vec![dir.path().to_path_buf()];
    let result = scan(&paths, &default_opts())?;
    assert!(
        result.is_empty(),
        "empty directory should produce no languages"
    );
    Ok(())
}

// ── All files excluded ───────────────────────────────────────────────

#[test]
fn scan_with_all_files_excluded_returns_empty() -> Result<()> {
    let opts = ScanOptions {
        excluded: vec!["*".to_string()],
        ..default_opts()
    };
    let paths = vec![PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src")];
    let result = scan(&paths, &opts)?;
    assert!(result.is_empty(), "excluding * should produce no languages");
    Ok(())
}

#[test]
fn scan_excluding_rs_files_has_no_rust() -> Result<()> {
    let opts = ScanOptions {
        excluded: vec!["*.rs".to_string()],
        ..default_opts()
    };
    let paths = vec![PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src")];
    let result = scan(&paths, &opts)?;
    assert!(
        result.get(&tokei::LanguageType::Rust).is_none(),
        "excluding *.rs should remove Rust from results"
    );
    Ok(())
}

// ── Empty paths slice ────────────────────────────────────────────────

#[test]
fn scan_with_empty_paths_slice_panics_in_tokei() {
    // tokei panics when given an empty path slice (unwrap on None).
    // This documents the upstream behavior.
    let paths: Vec<PathBuf> = vec![];
    let result = std::panic::catch_unwind(|| scan(&paths, &default_opts()));
    assert!(result.is_err(), "empty paths should cause a tokei panic");
}

// ── Path to a file (not directory) ───────────────────────────────────

#[test]
fn scan_single_file_path_succeeds() -> Result<()> {
    let file = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("src")
        .join("lib.rs");
    if file.exists() {
        let paths = vec![file];
        let result = scan(&paths, &default_opts())?;
        assert!(
            result.get(&tokei::LanguageType::Rust).is_some(),
            "scanning a .rs file should find Rust"
        );
    }
    Ok(())
}

// ── Config modes ─────────────────────────────────────────────────────

#[test]
fn scan_with_config_auto_does_not_panic() -> Result<()> {
    let opts = ScanOptions {
        config: ConfigMode::Auto,
        ..default_opts()
    };
    let paths = vec![PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src")];
    let result = scan(&paths, &opts);
    assert!(result.is_ok(), "config=auto should not panic");
    Ok(())
}

// ── Multiple exclusion patterns ──────────────────────────────────────

#[test]
fn scan_with_multiple_exclude_patterns() -> Result<()> {
    let opts = ScanOptions {
        excluded: vec![
            "target".to_string(),
            "*.min.js".to_string(),
            "node_modules".to_string(),
        ],
        ..default_opts()
    };
    let paths = vec![PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src")];
    let result = scan(&paths, &opts);
    assert!(result.is_ok(), "multiple excludes should not cause errors");
    Ok(())
}
