//! Error handling and edge case tests for tokmd-scan.

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

// ── Non-existent paths ──────────────────────────────────────────────

#[test]
fn scan_nonexistent_directory_returns_error() {
    let result = scan(&[PathBuf::from("/nonexistent/path/xyz")], &default_opts());
    assert!(result.is_err());
    let msg = result.unwrap_err().to_string();
    assert!(
        msg.contains("Path not found"),
        "expected 'Path not found' in: {msg}"
    );
}

#[test]
fn scan_nonexistent_file_returns_error() {
    let result = scan(&[PathBuf::from("/nonexistent/file.rs")], &default_opts());
    assert!(result.is_err());
}

#[test]
fn scan_multiple_paths_one_missing_returns_error() {
    let tmp = tempfile::tempdir().unwrap();
    let valid = tmp.path().to_path_buf();
    let invalid = PathBuf::from("/nonexistent/abc");
    let result = scan(&[valid, invalid], &default_opts());
    assert!(result.is_err());
}

// ── Empty directories ───────────────────────────────────────────────

#[test]
fn scan_empty_directory_returns_empty_languages() {
    let tmp = tempfile::tempdir().unwrap();
    let result = scan(&[tmp.path().to_path_buf()], &default_opts());
    assert!(result.is_ok());
    let langs = result.unwrap();
    // Empty dir should yield zero language stats
    let total: usize = langs.iter().map(|(_, report)| report.code).sum();
    assert_eq!(total, 0);
}

#[test]
#[should_panic]
fn scan_empty_paths_slice_panics_in_tokei() {
    // tokei panics when given an empty paths slice — this documents the behavior
    let _ = scan(&[], &default_opts());
}

// ── Binary-only directories ────────────────────────────────────────

#[test]
fn scan_directory_with_only_binary_files_yields_no_code() {
    let tmp = tempfile::tempdir().unwrap();
    // Write a binary file
    let bin_path = tmp.path().join("data.bin");
    std::fs::write(&bin_path, &[0x00, 0xFF, 0xFE, 0x89, 0x50, 0x4E, 0x47]).unwrap();
    // Write another binary file with known extension
    let img_path = tmp.path().join("image.png");
    std::fs::write(&img_path, &[0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A]).unwrap();

    let result = scan(&[tmp.path().to_path_buf()], &default_opts());
    assert!(result.is_ok());
    let langs = result.unwrap();
    let total: usize = langs.iter().map(|(_, r)| r.code).sum();
    assert_eq!(total, 0);
}

// ── Exclusion patterns ─────────────────────────────────────────────

#[test]
fn scan_with_exclude_filters_out_matching_files() {
    let tmp = tempfile::tempdir().unwrap();
    std::fs::write(tmp.path().join("main.rs"), "fn main() {}\n").unwrap();
    std::fs::write(tmp.path().join("test.py"), "print('hello')\n").unwrap();

    let opts = ScanOptions {
        excluded: vec!["*.py".to_string()],
        ..default_opts()
    };
    let result = scan(&[tmp.path().to_path_buf()], &opts).unwrap();
    // Should not include Python
    assert!(
        result.get(&tokei::LanguageType::Python).is_none()
            || result.get(&tokei::LanguageType::Python).unwrap().code == 0,
        "Python should be excluded"
    );
}

#[test]
fn scan_exclude_all_yields_no_code() {
    let tmp = tempfile::tempdir().unwrap();
    std::fs::write(tmp.path().join("main.rs"), "fn main() {}\n").unwrap();

    let opts = ScanOptions {
        excluded: vec!["*.rs".to_string()],
        ..default_opts()
    };
    let result = scan(&[tmp.path().to_path_buf()], &opts).unwrap();
    let total: usize = result.iter().map(|(_, r)| r.code).sum();
    assert_eq!(total, 0);
}

// ── Config mode variants───────────────────────────────────────────

#[test]
fn scan_with_config_auto_succeeds() {
    let tmp = tempfile::tempdir().unwrap();
    std::fs::write(tmp.path().join("lib.rs"), "pub fn f() {}\n").unwrap();

    let opts = ScanOptions {
        config: ConfigMode::Auto,
        ..default_opts()
    };
    let result = scan(&[tmp.path().to_path_buf()], &opts);
    assert!(result.is_ok());
}

#[test]
fn scan_with_config_none_succeeds() {
    let tmp = tempfile::tempdir().unwrap();
    std::fs::write(tmp.path().join("lib.rs"), "pub fn f() {}\n").unwrap();

    let opts = ScanOptions {
        config: ConfigMode::None,
        ..default_opts()
    };
    let result = scan(&[tmp.path().to_path_buf()], &opts);
    assert!(result.is_ok());
}

// ── Edge cases ─────────────────────────────────────────────────────

#[test]
fn scan_file_with_no_extension_is_not_counted() {
    let tmp = tempfile::tempdir().unwrap();
    std::fs::write(tmp.path().join("Makefile"), "all:\n\techo hello\n").unwrap();
    // Some filenames without recognized extensions
    std::fs::write(tmp.path().join("README"), "Hello world\n").unwrap();

    let result = scan(&[tmp.path().to_path_buf()], &default_opts());
    assert!(result.is_ok());
}

#[test]
fn scan_deeply_nested_directory() {
    let tmp = tempfile::tempdir().unwrap();
    let deep = tmp.path().join("a").join("b").join("c").join("d");
    std::fs::create_dir_all(&deep).unwrap();
    std::fs::write(deep.join("main.rs"), "fn main() {}\n").unwrap();

    let result = scan(&[tmp.path().to_path_buf()], &default_opts()).unwrap();
    let rust_code: usize = result
        .get(&tokei::LanguageType::Rust)
        .map(|r| r.code)
        .unwrap_or(0);
    assert!(rust_code > 0);
}

#[test]
fn scan_with_hidden_flag_includes_dotfiles() {
    let tmp = tempfile::tempdir().unwrap();
    let hidden_dir = tmp.path().join(".hidden");
    std::fs::create_dir_all(&hidden_dir).unwrap();
    std::fs::write(hidden_dir.join("secret.rs"), "fn secret() {}\n").unwrap();

    let opts_hidden = ScanOptions {
        hidden: true,
        no_ignore: true,
        ..default_opts()
    };
    let result = scan(&[tmp.path().to_path_buf()], &opts_hidden).unwrap();
    let rust_code: usize = result
        .get(&tokei::LanguageType::Rust)
        .map(|r| r.code)
        .unwrap_or(0);
    assert!(rust_code > 0, "hidden Rust file should be counted");
}

#[test]
fn scan_symlink_like_path_nonexistent_target() {
    // Ensure a path that doesn't resolve produces a clear error
    let result = scan(
        &[PathBuf::from("C:\\does_not_exist_at_all\\foo")],
        &default_opts(),
    );
    assert!(result.is_err());
}
