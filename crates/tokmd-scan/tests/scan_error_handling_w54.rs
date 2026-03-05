//! Comprehensive error handling and edge case tests for tokmd-scan.

use std::path::PathBuf;
use tokmd_scan::scan;
use tokmd_settings::ScanOptions;
use tokmd_types::ConfigMode;

fn default_opts() -> ScanOptions {
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

// ── Non-existent / missing paths ──────────────────────────────────────

#[test]
fn scan_nonexistent_directory_returns_error() {
    let opts = default_opts();
    let paths = vec![PathBuf::from("/definitely/does/not/exist/w54")];
    let result = scan(&paths, &opts);
    assert!(result.is_err());
    let msg = result.unwrap_err().to_string();
    assert!(msg.contains("Path not found"), "got: {msg}");
}

#[test]
fn scan_nonexistent_file_returns_error() {
    let opts = default_opts();
    let dir = tempfile::tempdir().unwrap();
    let fake_file = dir.path().join("nonexistent.rs");
    let result = scan(&[fake_file], &opts);
    assert!(result.is_err());
}

#[test]
fn scan_multiple_paths_first_missing() {
    let opts = default_opts();
    let dir = tempfile::tempdir().unwrap();
    let missing = dir.path().join("nope");
    let existing = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src");
    // First path is missing → should bail before scanning the second
    let result = scan(&[missing, existing], &opts);
    assert!(result.is_err());
}

#[test]
fn scan_multiple_paths_second_missing() {
    let opts = default_opts();
    let dir = tempfile::tempdir().unwrap();
    let existing = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src");
    let missing = dir.path().join("nope");
    let result = scan(&[existing, missing], &opts);
    assert!(result.is_err());
}

// ── Empty directory ───────────────────────────────────────────────────

#[test]
fn scan_empty_directory_returns_empty_languages() {
    let opts = default_opts();
    let dir = tempfile::tempdir().unwrap();
    let result = scan(&[dir.path().to_path_buf()], &opts).unwrap();
    assert!(result.is_empty(), "empty dir should yield no languages");
}

#[test]
fn scan_empty_directory_with_config_none_returns_empty() {
    let opts = ScanOptions {
        config: ConfigMode::None,
        ..default_opts()
    };
    let dir = tempfile::tempdir().unwrap();
    let result = scan(&[dir.path().to_path_buf()], &opts).unwrap();
    assert!(
        result.is_empty(),
        "empty dir with ConfigMode::None → no languages"
    );
}

// ── Directory with only ignored / unrecognized files ──────────────────

#[test]
fn scan_directory_with_only_ignored_files() {
    let opts = default_opts();
    let dir = tempfile::tempdir().unwrap();
    // Create files that tokei won't recognize as source code
    std::fs::write(dir.path().join(".hidden"), "hidden content").unwrap();
    std::fs::write(dir.path().join("data.bin"), [0u8; 64]).unwrap();
    let result = scan(&[dir.path().to_path_buf()], &opts).unwrap();
    assert!(
        result.is_empty(),
        "unrecognized files should yield empty languages"
    );
}

#[test]
fn scan_directory_with_only_gitignore() {
    let opts = default_opts();
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join(".gitignore"), "*.rs\n").unwrap();
    let result = scan(&[dir.path().to_path_buf()], &opts).unwrap();
    assert!(result.is_empty());
}

// ── Config combinations ──────────────────────────────────────────────

#[test]
fn scan_config_mode_none_with_hidden() {
    let opts = ScanOptions {
        config: ConfigMode::None,
        hidden: true,
        ..default_opts()
    };
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src");
    let result = scan(&[path], &opts);
    assert!(result.is_ok());
    assert!(!result.unwrap().is_empty());
}

#[test]
fn scan_config_mode_auto_with_all_no_ignore() {
    let opts = ScanOptions {
        config: ConfigMode::Auto,
        no_ignore: true,
        no_ignore_parent: true,
        no_ignore_dot: true,
        no_ignore_vcs: true,
        ..default_opts()
    };
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src");
    let result = scan(&[path], &opts);
    assert!(result.is_ok());
}

#[test]
fn scan_with_exclusion_pattern_matching_all_files() {
    let opts = ScanOptions {
        excluded: vec!["*.rs".to_string()],
        ..default_opts()
    };
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("main.rs"), "fn main() {}").unwrap();
    let result = scan(&[dir.path().to_path_buf()], &opts).unwrap();
    // All .rs files excluded → empty
    assert!(result.is_empty());
}

#[test]
fn scan_with_doc_strings_as_comments_flag() {
    let opts = ScanOptions {
        treat_doc_strings_as_comments: true,
        config: ConfigMode::None,
        ..default_opts()
    };
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src");
    let result = scan(&[path], &opts).unwrap();
    let rust = result.get(&tokei::LanguageType::Rust).unwrap();
    // With doc-strings-as-comments, comment count should be > 0
    // (this crate has doc comments)
    assert!(rust.comments > 0 || rust.code > 0);
}

// ── Minimal input verification ────────────────────────────────────────

#[test]
fn scan_single_rust_file_directory() {
    let opts = default_opts();
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("lib.rs"), "pub fn hello() {}\n").unwrap();
    let result = scan(&[dir.path().to_path_buf()], &opts).unwrap();
    let rust = result.get(&tokei::LanguageType::Rust);
    assert!(rust.is_some(), "should detect Rust");
    assert!(rust.unwrap().code > 0);
}

#[test]
fn scan_reports_correct_code_lines() {
    let opts = ScanOptions {
        config: ConfigMode::None,
        ..default_opts()
    };
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(
        dir.path().join("main.rs"),
        "// comment\nfn main() {\n    println!(\"hi\");\n}\n",
    )
    .unwrap();
    let result = scan(&[dir.path().to_path_buf()], &opts).unwrap();
    let rust = result.get(&tokei::LanguageType::Rust).unwrap();
    assert_eq!(rust.code, 3, "3 code lines (fn, println, closing brace)");
    assert_eq!(rust.comments, 1, "1 comment line");
}

#[test]
fn scan_error_message_includes_path() {
    let opts = default_opts();
    let bad_path = PathBuf::from("w54_this_path_does_not_exist_at_all");
    let err = scan(std::slice::from_ref(&bad_path), &opts).unwrap_err();
    assert!(
        err.to_string()
            .contains("w54_this_path_does_not_exist_at_all"),
        "error should mention the path: {}",
        err
    );
}
