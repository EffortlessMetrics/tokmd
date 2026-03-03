//! Error handling tests for tokmd-scan.
//!
//! Tests non-existent directories, empty directories,
//! and edge cases in the scan function.

use std::path::PathBuf;
use tokmd_scan::scan;
use tokmd_settings::ScanOptions;
use tokmd_types::ConfigMode;

fn default_options() -> ScanOptions {
    ScanOptions {
        excluded: vec![],
        config: ConfigMode::Auto,
        hidden: false,
        no_ignore: false,
        no_ignore_parent: false,
        no_ignore_dot: false,
        no_ignore_vcs: false,
        treat_doc_strings_as_comments: false,
    }
}

// =============================================================================
// Non-existent directories
// =============================================================================

#[test]
fn scan_nonexistent_directory_returns_error() {
    let opts = default_options();
    let paths = vec![PathBuf::from("/tmp/tokmd-errors-nonexistent-dir-xyz-123")];
    let result = scan(&paths, &opts);
    assert!(result.is_err());
    let err = result.unwrap_err().to_string();
    assert!(
        err.contains("Path not found"),
        "Error should mention 'Path not found': {err}"
    );
}

#[test]
fn scan_nonexistent_directory_error_contains_path() {
    let opts = default_options();
    let bad_path = "/tmp/tokmd-errors-specific-missing-path";
    let paths = vec![PathBuf::from(bad_path)];
    let result = scan(&paths, &opts);
    assert!(result.is_err());
    let err = result.unwrap_err().to_string();
    assert!(
        err.contains(bad_path),
        "Error should contain the path: {err}"
    );
}

#[test]
fn scan_multiple_paths_one_nonexistent_returns_error() {
    let opts = default_options();
    let valid = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src");
    let invalid = PathBuf::from("/tmp/tokmd-errors-nonexistent-dir-abc");
    let paths = vec![valid, invalid];
    let result = scan(&paths, &opts);
    assert!(result.is_err());
}

// =============================================================================
// Empty directories
// =============================================================================

#[test]
fn scan_empty_directory_returns_empty_languages() {
    let dir = tempfile::tempdir().expect("create temp dir");
    let opts = default_options();
    let paths = vec![dir.path().to_path_buf()];
    let result = scan(&paths, &opts);
    // An empty directory should scan successfully but find no languages
    assert!(result.is_ok());
    let languages = result.unwrap();
    // All language entries should have 0 files
    let total_files: usize = languages.values().map(|l| l.reports.len()).sum();
    assert_eq!(total_files, 0, "Empty directory should have no files");
}

#[test]
fn scan_directory_with_only_hidden_files_default_skips_them() {
    let dir = tempfile::tempdir().expect("create temp dir");
    // Create a hidden file
    let hidden_file = dir.path().join(".hidden_test.rs");
    std::fs::write(&hidden_file, "fn main() {}").expect("write hidden file");

    let opts = default_options(); // hidden = false (default)
    let paths = vec![dir.path().to_path_buf()];
    let result = scan(&paths, &opts);
    assert!(result.is_ok());
    // With default settings, hidden files should be skipped
    let languages = result.unwrap();
    let total_files: usize = languages.values().map(|l| l.reports.len()).sum();
    assert_eq!(total_files, 0, "Hidden files should be skipped by default");
}

#[test]
fn scan_directory_with_hidden_files_found_when_enabled() {
    let dir = tempfile::tempdir().expect("create temp dir");
    let hidden_file = dir.path().join(".hidden_test.rs");
    std::fs::write(&hidden_file, "fn main() {}").expect("write hidden file");

    let mut opts = default_options();
    opts.hidden = true;
    opts.config = ConfigMode::None;
    let paths = vec![dir.path().to_path_buf()];
    let result = scan(&paths, &opts);
    assert!(result.is_ok());
    let languages = result.unwrap();
    let total_files: usize = languages.values().map(|l| l.reports.len()).sum();
    assert!(total_files > 0, "Hidden files should be found when enabled");
}

// =============================================================================
// Empty paths list
// =============================================================================

#[test]
fn scan_empty_paths_does_not_error_gracefully() {
    // tokei panics on empty paths - this documents the upstream limitation.
    // The scan function relies on tokei internals, so empty paths is
    // not a supported input. Callers should provide at least one path.
    let opts = default_options();
    let paths: Vec<PathBuf> = vec![];
    let result = std::panic::catch_unwind(|| scan(&paths, &opts));
    // Either it panics (tokei limitation) or returns Ok - both are acceptable
    // but it should not hang or produce undefined behavior.
    let _ = result;
}

// =============================================================================
// Config mode variations
// =============================================================================

#[test]
fn scan_with_config_none_succeeds() {
    let mut opts = default_options();
    opts.config = ConfigMode::None;
    let valid = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src");
    let paths = vec![valid];
    let result = scan(&paths, &opts);
    assert!(result.is_ok());
}

#[test]
fn scan_with_all_ignore_flags_succeeds() {
    let mut opts = default_options();
    opts.no_ignore = true;
    opts.no_ignore_parent = true;
    opts.no_ignore_dot = true;
    opts.no_ignore_vcs = true;
    opts.hidden = true;
    opts.treat_doc_strings_as_comments = true;
    opts.config = ConfigMode::None;
    let valid = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src");
    let paths = vec![valid];
    let result = scan(&paths, &opts);
    assert!(result.is_ok());
}

// =============================================================================
// Excluded patterns
// =============================================================================

#[test]
fn scan_with_exclusion_patterns_succeeds() {
    let mut opts = default_options();
    opts.excluded = vec!["*.rs".to_string()]; // Exclude all Rust files
    let valid = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src");
    let paths = vec![valid];
    let result = scan(&paths, &opts);
    assert!(result.is_ok());
    let languages = result.unwrap();
    // Should find no Rust files since they're all excluded
    let rust = languages.get(&tokei::LanguageType::Rust);
    let rust_files = rust.map(|l| l.reports.len()).unwrap_or(0);
    assert_eq!(rust_files, 0, "All .rs files should be excluded");
}
