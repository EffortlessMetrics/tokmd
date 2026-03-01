//! BDD-style scenarios for `tokmd-scan`.
//!
//! Each scenario exercises the public `scan()` function with a real (or
//! temporary) filesystem tree so that configuration flags, error paths, and
//! result-processing invariants are covered end-to-end.

use std::fs;
use std::path::PathBuf;

use anyhow::Result;
use tempfile::TempDir;
use tokmd_scan::scan;
use tokmd_settings::ScanOptions;
use tokmd_types::ConfigMode;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

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

/// The crate's own `src/` directory – always contains at least `lib.rs`.
fn crate_src() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src")
}

/// Write a file inside `dir`, creating intermediate directories as needed.
fn write_file(dir: &TempDir, rel: &str, content: &str) {
    let path = dir.path().join(rel);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    fs::write(&path, content).unwrap();
}

// ===========================================================================
// Scenario group: basic scan behaviour
// ===========================================================================

#[test]
fn given_rust_source_when_scanned_then_rust_language_detected() -> Result<()> {
    let langs = scan(&[crate_src()], &default_opts())?;

    assert!(
        langs.get(&tokei::LanguageType::Rust).is_some(),
        "expected Rust language in scan results"
    );
    Ok(())
}

#[test]
fn given_rust_source_when_scanned_then_code_lines_positive() -> Result<()> {
    let langs = scan(&[crate_src()], &default_opts())?;
    let rust = langs.get(&tokei::LanguageType::Rust).unwrap();

    assert!(rust.code > 0, "expected positive code-line count");
    assert!(rust.lines() > 0, "expected positive total-line count");
    Ok(())
}

#[test]
fn given_empty_directory_when_scanned_then_result_is_empty() -> Result<()> {
    let tmp = TempDir::new()?;
    let langs = scan(&[tmp.path().to_path_buf()], &default_opts())?;

    assert!(langs.is_empty(), "empty dir should yield zero languages");
    Ok(())
}

#[test]
fn given_temp_rust_file_when_scanned_then_detected() -> Result<()> {
    let tmp = TempDir::new()?;
    write_file(&tmp, "main.rs", "fn main() {}\n");

    let langs = scan(&[tmp.path().to_path_buf()], &default_opts())?;

    assert!(langs.get(&tokei::LanguageType::Rust).is_some());
    let rust = langs.get(&tokei::LanguageType::Rust).unwrap();
    assert!(rust.code > 0);
    Ok(())
}

// ===========================================================================
// Scenario group: multiple paths
// ===========================================================================

#[test]
fn given_two_temp_dirs_when_scanned_then_results_merged() -> Result<()> {
    let a = TempDir::new()?;
    let b = TempDir::new()?;
    write_file(&a, "a.rs", "fn a() {}\n");
    write_file(&b, "b.rs", "fn b() {}\nfn c() {}\n");

    let langs = scan(
        &[a.path().to_path_buf(), b.path().to_path_buf()],
        &default_opts(),
    )?;

    let rust = langs.get(&tokei::LanguageType::Rust).unwrap();
    // Two source files ⇒ at least 2 code lines (one from each dir).
    assert!(rust.code >= 2);
    Ok(())
}

// ===========================================================================
// Scenario group: error handling
// ===========================================================================

#[test]
fn given_nonexistent_path_when_scanned_then_error_returned() {
    let tmp = TempDir::new().unwrap();
    let bad = tmp.path().join("nope");

    let err = scan(&[bad], &default_opts()).unwrap_err();
    assert!(
        err.to_string().contains("Path not found"),
        "error should mention 'Path not found', got: {err}"
    );
}

#[test]
fn given_mixed_valid_and_invalid_paths_when_scanned_then_error_returned() {
    let bad = PathBuf::from("__this_path_does_not_exist__");

    let err = scan(&[crate_src(), bad], &default_opts()).unwrap_err();
    assert!(err.to_string().contains("Path not found"));
}

// ===========================================================================
// Scenario group: exclusion patterns
// ===========================================================================

#[test]
fn given_exclusion_matching_all_files_when_scanned_then_result_empty() -> Result<()> {
    let tmp = TempDir::new()?;
    write_file(&tmp, "hello.rs", "fn hello() {}\n");

    let mut opts = default_opts();
    // Exclude everything under the temp dir via a catch-all glob.
    opts.excluded = vec!["*.rs".to_string()];

    let langs = scan(&[tmp.path().to_path_buf()], &opts)?;

    // The Rust file should have been excluded.
    assert!(
        langs.get(&tokei::LanguageType::Rust).is_none()
            || langs
                .get(&tokei::LanguageType::Rust)
                .is_none_or(|r| r.code == 0),
        "excluded .rs files should not appear"
    );
    Ok(())
}

#[test]
fn given_exclusion_not_matching_when_scanned_then_files_still_counted() -> Result<()> {
    let tmp = TempDir::new()?;
    write_file(&tmp, "main.rs", "fn main() {}\n");

    let mut opts = default_opts();
    opts.excluded = vec!["*.py".to_string()]; // won't match .rs

    let langs = scan(&[tmp.path().to_path_buf()], &opts)?;
    assert!(langs.get(&tokei::LanguageType::Rust).is_some());
    Ok(())
}

#[test]
fn given_subdirectory_exclusion_when_scanned_then_subdir_skipped() -> Result<()> {
    let tmp = TempDir::new()?;
    write_file(&tmp, "root.rs", "fn root() {}\n");
    write_file(&tmp, "vendor/dep.rs", "fn dep() {}\n");

    let mut opts = default_opts();
    opts.excluded = vec!["vendor".to_string()];

    let langs = scan(&[tmp.path().to_path_buf()], &opts)?;
    let rust = langs.get(&tokei::LanguageType::Rust).unwrap();

    // Only root.rs should be counted (1 code line).
    assert_eq!(rust.code, 1, "vendor/ should have been excluded");
    Ok(())
}

// ===========================================================================
// Scenario group: ConfigMode
// ===========================================================================

#[test]
fn given_config_mode_none_when_scanned_then_succeeds() -> Result<()> {
    let mut opts = default_opts();
    opts.config = ConfigMode::None;

    let langs = scan(&[crate_src()], &opts)?;
    assert!(!langs.is_empty());
    Ok(())
}

#[test]
fn given_config_mode_auto_when_scanned_then_succeeds() -> Result<()> {
    let mut opts = default_opts();
    opts.config = ConfigMode::Auto;

    let langs = scan(&[crate_src()], &opts)?;
    assert!(!langs.is_empty());
    Ok(())
}

// ===========================================================================
// Scenario group: boolean flag effects
// ===========================================================================

#[test]
fn given_hidden_flag_when_scanned_then_hidden_files_included() -> Result<()> {
    let tmp = TempDir::new()?;
    write_file(&tmp, ".hidden.rs", "fn secret() {}\n");
    write_file(&tmp, "visible.rs", "fn public() {}\n");

    let mut opts = default_opts();
    opts.hidden = true;
    opts.no_ignore = true; // ensure ignores don't interfere

    let langs = scan(&[tmp.path().to_path_buf()], &opts)?;
    let rust = langs.get(&tokei::LanguageType::Rust).unwrap();

    // Both hidden and visible files should be counted.
    assert!(rust.code >= 2, "hidden file should be counted");
    Ok(())
}

#[test]
fn given_treat_doc_strings_flag_when_scanned_then_doc_comments_counted_as_comments() -> Result<()> {
    let tmp = TempDir::new()?;
    let src = "\
/// This is a doc comment.
fn documented() {}
";
    write_file(&tmp, "lib.rs", src);

    // Scan without the flag.
    let without = scan(&[tmp.path().to_path_buf()], &default_opts())?;
    let rust_without = without.get(&tokei::LanguageType::Rust).unwrap();

    // Scan with the flag.
    let mut opts = default_opts();
    opts.treat_doc_strings_as_comments = true;
    let with = scan(&[tmp.path().to_path_buf()], &opts)?;
    let rust_with = with.get(&tokei::LanguageType::Rust).unwrap();

    // With the flag, the doc comment line should move from "code" (or doc)
    // to "comments", so the comment count should be >= without-flag count.
    assert!(
        rust_with.comments >= rust_without.comments,
        "comments should be >= when treating doc strings as comments"
    );
    Ok(())
}

// ===========================================================================
// Scenario group: determinism / idempotency
// ===========================================================================

#[test]
fn given_same_input_when_scanned_twice_then_results_identical() -> Result<()> {
    let opts = default_opts();
    let paths = vec![crate_src()];

    let r1 = scan(&paths, &opts)?;
    let r2 = scan(&paths, &opts)?;

    let rust1 = r1.get(&tokei::LanguageType::Rust).unwrap();
    let rust2 = r2.get(&tokei::LanguageType::Rust).unwrap();

    assert_eq!(rust1.code, rust2.code, "code lines must be identical");
    assert_eq!(
        rust1.comments, rust2.comments,
        "comment lines must be identical"
    );
    assert_eq!(rust1.blanks, rust2.blanks, "blank lines must be identical");
    Ok(())
}

// ===========================================================================
// Scenario group: multi-language detection
// ===========================================================================

#[test]
fn given_multiple_languages_when_scanned_then_all_detected() -> Result<()> {
    let tmp = TempDir::new()?;
    write_file(&tmp, "app.rs", "fn main() {}\n");
    write_file(&tmp, "util.py", "def util():\n    pass\n");
    write_file(&tmp, "index.js", "function f() {}\n");

    let langs = scan(&[tmp.path().to_path_buf()], &default_opts())?;

    assert!(
        langs.get(&tokei::LanguageType::Rust).is_some(),
        "Rust should be detected"
    );
    assert!(
        langs.get(&tokei::LanguageType::Python).is_some(),
        "Python should be detected"
    );
    assert!(
        langs.get(&tokei::LanguageType::JavaScript).is_some(),
        "JavaScript should be detected"
    );
    Ok(())
}

#[test]
fn given_nested_structure_when_scanned_then_all_files_found() -> Result<()> {
    let tmp = TempDir::new()?;
    write_file(&tmp, "a/b/c/deep.rs", "fn deep() {}\n");
    write_file(&tmp, "top.rs", "fn top() {}\n");

    let langs = scan(&[tmp.path().to_path_buf()], &default_opts())?;
    let rust = langs.get(&tokei::LanguageType::Rust).unwrap();

    // Both files should be counted.
    assert!(rust.code >= 2);
    Ok(())
}
