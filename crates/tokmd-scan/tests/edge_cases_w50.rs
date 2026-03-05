//! Edge-case and boundary-condition tests for tokmd-scan.

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

// ---------------------------------------------------------------------------
// Non-existent path
// ---------------------------------------------------------------------------

#[test]
fn scan_nonexistent_path_errors_not_panics() {
    let opts = default_opts();
    let result = scan(&[PathBuf::from("/this/path/does/not/exist")], &opts);
    assert!(result.is_err());
    let msg = result.unwrap_err().to_string();
    assert!(msg.contains("Path not found"), "unexpected error: {msg}");
}

// ---------------------------------------------------------------------------
// Empty directory
// ---------------------------------------------------------------------------

#[test]
fn scan_empty_directory_succeeds() {
    let dir = tempfile::tempdir().unwrap();
    let opts = default_opts();
    let result = scan(&[dir.path().to_path_buf()], &opts);
    assert!(result.is_ok());
    let langs = result.unwrap();
    // An empty directory should produce an empty languages map
    assert!(langs.is_empty());
}

// ---------------------------------------------------------------------------
// Path with spaces
// ---------------------------------------------------------------------------

#[test]
fn scan_path_with_spaces() {
    let dir = tempfile::tempdir().unwrap();
    let spaced = dir.path().join("my project");
    std::fs::create_dir_all(&spaced).unwrap();
    std::fs::write(spaced.join("main.rs"), "fn main() {}\n").unwrap();

    let opts = default_opts();
    let result = scan(&[spaced], &opts);
    assert!(result.is_ok());
    let langs = result.unwrap();
    assert!(!langs.is_empty(), "should find at least one file");
}

// ---------------------------------------------------------------------------
// Very deep directory nesting
// ---------------------------------------------------------------------------

#[test]
fn scan_deep_directory_nesting() {
    let dir = tempfile::tempdir().unwrap();
    let mut deep = dir.path().to_path_buf();
    for i in 0..15 {
        deep = deep.join(format!("level_{i}"));
    }
    std::fs::create_dir_all(&deep).unwrap();
    std::fs::write(deep.join("deep.rs"), "fn deep() {}\n").unwrap();

    let opts = default_opts();
    let result = scan(&[dir.path().to_path_buf()], &opts);
    assert!(result.is_ok());
    let langs = result.unwrap();
    assert!(!langs.is_empty(), "should find the deeply nested file");
}

// ---------------------------------------------------------------------------
// Multiple paths, one valid and one invalid
// ---------------------------------------------------------------------------

#[test]
fn scan_mixed_valid_invalid_paths_errors() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("lib.rs"), "// comment\n").unwrap();
    let opts = default_opts();
    let result = scan(
        &[dir.path().to_path_buf(), PathBuf::from("/nonexistent/fake")],
        &opts,
    );
    assert!(result.is_err());
}

// ---------------------------------------------------------------------------
// Excluded patterns filter results
// ---------------------------------------------------------------------------

#[test]
fn scan_excluded_patterns_applied() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("keep.rs"), "fn keep() {}\n").unwrap();
    std::fs::write(dir.path().join("skip.py"), "print('skip')\n").unwrap();

    let mut opts = default_opts();
    opts.excluded = vec!["*.py".to_string()];
    let result = scan(&[dir.path().to_path_buf()], &opts);
    assert!(result.is_ok());
    let langs = result.unwrap();
    assert!(
        langs.get(&tokei::LanguageType::Python).is_none(),
        "python files should be excluded"
    );
}
