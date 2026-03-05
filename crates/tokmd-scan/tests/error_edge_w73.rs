//! Error handling and edge case tests for tokmd-scan (W73).
//!
//! Tests non-existent directories, empty directories, hidden-only directories,
//! conflicting options, and boundary conditions in the scan function.

use std::fs;
use std::io::Write;
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
// Non-existent directory
// =============================================================================

#[test]
fn scan_nonexistent_directory_error_message_includes_path() {
    let dir = tempfile::tempdir().unwrap();
    let missing = dir.path().join("does_not_exist_w73");
    let result = scan(std::slice::from_ref(&missing), &default_options());
    assert!(result.is_err());
    let msg = result.unwrap_err().to_string();
    assert!(
        msg.contains("Path not found"),
        "expected 'Path not found', got: {msg}"
    );
    assert!(
        msg.contains("does_not_exist_w73"),
        "error should contain path fragment: {msg}"
    );
}

// =============================================================================
// Empty directory
// =============================================================================

#[test]
fn scan_empty_directory_returns_empty_languages() {
    let dir = tempfile::tempdir().unwrap();
    let result = scan(&[dir.path().to_path_buf()], &default_options());
    assert!(result.is_ok());
    let langs = result.unwrap();
    let total_code: usize = langs.values().map(|lang| lang.code).sum();
    assert_eq!(total_code, 0, "empty dir should yield zero code lines");
}

// =============================================================================
// Directory with only hidden files
// =============================================================================

#[test]
fn scan_hidden_only_dir_without_hidden_flag_returns_no_code() {
    let dir = tempfile::tempdir().unwrap();
    let hidden_file = dir.path().join(".hidden_source.rs");
    let mut f = fs::File::create(&hidden_file).unwrap();
    writeln!(f, "fn main() {{ println!(\"hello\"); }}").unwrap();

    let opts = default_options(); // hidden = false
    let result = scan(&[dir.path().to_path_buf()], &opts).unwrap();
    let total_code: usize = result.values().map(|lang| lang.code).sum();
    assert_eq!(
        total_code, 0,
        "hidden files should not be counted without --hidden"
    );
}

#[test]
fn scan_hidden_only_dir_with_hidden_flag_finds_code() {
    let dir = tempfile::tempdir().unwrap();
    let hidden_file = dir.path().join(".hidden_source.rs");
    let mut f = fs::File::create(&hidden_file).unwrap();
    writeln!(f, "fn main() {{ println!(\"hello\"); }}").unwrap();

    let mut opts = default_options();
    opts.hidden = true;
    let result = scan(&[dir.path().to_path_buf()], &opts).unwrap();
    let total_code: usize = result.values().map(|lang| lang.code).sum();
    assert!(
        total_code > 0,
        "hidden files should be counted with --hidden"
    );
}

// =============================================================================
// Conflicting / combined options
// =============================================================================

#[test]
fn scan_with_all_no_ignore_flags_does_not_panic() {
    let dir = tempfile::tempdir().unwrap();
    let src = dir.path().join("hello.rs");
    fs::write(&src, "fn main() {}").unwrap();

    let opts = ScanOptions {
        excluded: vec![],
        config: ConfigMode::None,
        hidden: true,
        no_ignore: true,
        no_ignore_parent: true,
        no_ignore_dot: true,
        no_ignore_vcs: true,
        treat_doc_strings_as_comments: true,
    };
    let result = scan(&[dir.path().to_path_buf()], &opts);
    assert!(result.is_ok());
}

#[test]
fn scan_with_exclude_pattern_filters_matching_files() {
    let dir = tempfile::tempdir().unwrap();
    let src = dir.path().join("keep.rs");
    fs::write(&src, "fn keep() {}").unwrap();
    let excluded = dir.path().join("skip.rs");
    fs::write(&excluded, "fn skip() {}").unwrap();

    let opts = ScanOptions {
        excluded: vec!["skip.rs".to_string()],
        config: ConfigMode::None,
        ..default_options()
    };
    let result = scan(&[dir.path().to_path_buf()], &opts).unwrap();
    if let Some(rust) = result.get(&tokei::LanguageType::Rust) {
        assert!(
            rust.reports.len() <= 1,
            "excluded file should be filtered out"
        );
    }
}

// =============================================================================
// Multiple paths — mixed valid / invalid
// =============================================================================

#[test]
fn scan_multiple_paths_first_invalid_returns_error() {
    let dir = tempfile::tempdir().unwrap();
    let valid = dir.path().to_path_buf();
    let invalid = dir.path().join("nope_w73");
    let result = scan(&[invalid, valid], &default_options());
    assert!(result.is_err(), "should fail when any path is invalid");
}

// =============================================================================
// Empty paths slice
// =============================================================================

#[test]
fn scan_empty_paths_slice_panics_in_tokei() {
    // tokei panics on empty paths — document this known limitation
    let result = std::panic::catch_unwind(|| scan(&[], &default_options()));
    assert!(result.is_err(), "tokei panics when given zero paths");
}

// =============================================================================
// ConfigMode variations
// =============================================================================

#[test]
fn scan_config_mode_none_succeeds_on_valid_dir() {
    let dir = tempfile::tempdir().unwrap();
    fs::write(dir.path().join("lib.rs"), "pub fn x() {}").unwrap();
    let mut opts = default_options();
    opts.config = ConfigMode::None;
    let result = scan(&[dir.path().to_path_buf()], &opts);
    assert!(result.is_ok());
}
