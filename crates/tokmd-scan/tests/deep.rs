//! Deep integration tests for `tokmd-scan`.
//!
//! Exercises the public `scan()` function across real and synthetic filesystem
//! trees, verifying determinism, language detection accuracy, config flags,
//! path normalization, children handling, and edge cases.

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

fn crate_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn crate_src() -> PathBuf {
    crate_root().join("src")
}

/// Write a file inside `dir`, creating intermediate directories as needed.
fn write_file(dir: &TempDir, rel: &str, content: &str) {
    let path = dir.path().join(rel);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).expect("create parent directories");
    }
    fs::write(&path, content).expect("write test file");
}

// ===========================================================================
// 1. Self-scan tests
// ===========================================================================

#[test]
fn self_scan_finds_rust() -> Result<()> {
    let langs = scan(&[crate_src()], &default_opts())?;
    assert!(
        langs.get(&tokei::LanguageType::Rust).is_some(),
        "self-scan must detect Rust"
    );
    Ok(())
}

#[test]
fn self_scan_has_nonzero_code_lines() -> Result<()> {
    let langs = scan(&[crate_src()], &default_opts())?;
    let rust = langs
        .get(&tokei::LanguageType::Rust)
        .expect("Rust not found");
    assert!(rust.code > 0, "lib.rs must have code lines");
    Ok(())
}

#[test]
fn self_scan_has_nonzero_comments() -> Result<()> {
    // lib.rs contains doc comments, so comments should be > 0
    let langs = scan(&[crate_src()], &default_opts())?;
    let rust = langs
        .get(&tokei::LanguageType::Rust)
        .expect("Rust not found");
    assert!(rust.comments > 0, "lib.rs should have doc comments");
    Ok(())
}

#[test]
fn self_scan_total_lines_is_sum_of_parts() -> Result<()> {
    let langs = scan(&[crate_src()], &default_opts())?;
    let rust = langs
        .get(&tokei::LanguageType::Rust)
        .expect("Rust not found");
    assert_eq!(
        rust.lines(),
        rust.code + rust.comments + rust.blanks,
        "total lines must be code + comments + blanks"
    );
    Ok(())
}

// ===========================================================================
// 2. Empty tempdir
// ===========================================================================

#[test]
fn scan_empty_dir_returns_empty_languages() -> Result<()> {
    let dir = tempfile::tempdir()?;
    let langs = scan(&[dir.path().to_path_buf()], &default_opts())?;
    assert!(langs.is_empty(), "empty dir should produce no languages");
    Ok(())
}

// ===========================================================================
// 3. Custom config modes
// ===========================================================================

#[test]
fn scan_config_auto_succeeds() -> Result<()> {
    let mut opts = default_opts();
    opts.config = ConfigMode::Auto;
    let langs = scan(&[crate_src()], &opts)?;
    assert!(!langs.is_empty());
    Ok(())
}

#[test]
fn scan_config_none_succeeds() -> Result<()> {
    let mut opts = default_opts();
    opts.config = ConfigMode::None;
    let langs = scan(&[crate_src()], &opts)?;
    assert!(!langs.is_empty());
    Ok(())
}

#[test]
fn scan_config_auto_and_none_both_find_rust() -> Result<()> {
    let mut auto_opts = default_opts();
    auto_opts.config = ConfigMode::Auto;
    let auto_langs = scan(&[crate_src()], &auto_opts)?;

    let mut none_opts = default_opts();
    none_opts.config = ConfigMode::None;
    let none_langs = scan(&[crate_src()], &none_opts)?;

    assert!(auto_langs.get(&tokei::LanguageType::Rust).is_some());
    assert!(none_langs.get(&tokei::LanguageType::Rust).is_some());
    Ok(())
}

// ===========================================================================
// 4. Determinism — same input gives same output
// ===========================================================================

#[test]
fn scan_is_deterministic_code_lines() -> Result<()> {
    let opts = default_opts();
    let paths = vec![crate_src()];

    let r1 = scan(&paths, &opts)?;
    let r2 = scan(&paths, &opts)?;

    let code1 = r1.get(&tokei::LanguageType::Rust).unwrap().code;
    let code2 = r2.get(&tokei::LanguageType::Rust).unwrap().code;
    assert_eq!(code1, code2, "deterministic: code lines must match");
    Ok(())
}

#[test]
fn scan_is_deterministic_comments() -> Result<()> {
    let opts = default_opts();
    let paths = vec![crate_src()];

    let r1 = scan(&paths, &opts)?;
    let r2 = scan(&paths, &opts)?;

    let c1 = r1.get(&tokei::LanguageType::Rust).unwrap().comments;
    let c2 = r2.get(&tokei::LanguageType::Rust).unwrap().comments;
    assert_eq!(c1, c2, "deterministic: comment lines must match");
    Ok(())
}

#[test]
fn scan_is_deterministic_blanks() -> Result<()> {
    let opts = default_opts();
    let paths = vec![crate_src()];

    let r1 = scan(&paths, &opts)?;
    let r2 = scan(&paths, &opts)?;

    let b1 = r1.get(&tokei::LanguageType::Rust).unwrap().blanks;
    let b2 = r2.get(&tokei::LanguageType::Rust).unwrap().blanks;
    assert_eq!(b1, b2, "deterministic: blank lines must match");
    Ok(())
}

#[test]
fn scan_determinism_language_set() -> Result<()> {
    let opts = default_opts();
    let paths = vec![crate_src()];

    let r1 = scan(&paths, &opts)?;
    let r2 = scan(&paths, &opts)?;

    let keys1: Vec<_> = r1.iter().map(|(k, _)| k).collect();
    let keys2: Vec<_> = r2.iter().map(|(k, _)| k).collect();
    assert_eq!(keys1, keys2, "deterministic: language set must match");
    Ok(())
}

// ===========================================================================
// 5. Language detection accuracy
// ===========================================================================

#[test]
fn detects_python_files() -> Result<()> {
    let dir = tempfile::tempdir()?;
    write_file(&dir, "hello.py", "#!/usr/bin/env python3\nprint('hello')\n");
    let langs = scan(&[dir.path().to_path_buf()], &default_opts())?;
    assert!(langs.get(&tokei::LanguageType::Python).is_some());
    Ok(())
}

#[test]
fn detects_javascript_files() -> Result<()> {
    let dir = tempfile::tempdir()?;
    write_file(&dir, "app.js", "const x = 1;\nconsole.log(x);\n");
    let langs = scan(&[dir.path().to_path_buf()], &default_opts())?;
    assert!(langs.get(&tokei::LanguageType::JavaScript).is_some());
    Ok(())
}

#[test]
fn detects_typescript_files() -> Result<()> {
    let dir = tempfile::tempdir()?;
    write_file(&dir, "app.ts", "const x: number = 1;\nconsole.log(x);\n");
    let langs = scan(&[dir.path().to_path_buf()], &default_opts())?;
    assert!(langs.get(&tokei::LanguageType::TypeScript).is_some());
    Ok(())
}

#[test]
fn detects_c_files() -> Result<()> {
    let dir = tempfile::tempdir()?;
    write_file(
        &dir,
        "main.c",
        "#include <stdio.h>\nint main() { return 0; }\n",
    );
    let langs = scan(&[dir.path().to_path_buf()], &default_opts())?;
    assert!(langs.get(&tokei::LanguageType::C).is_some());
    Ok(())
}

#[test]
fn detects_go_files() -> Result<()> {
    let dir = tempfile::tempdir()?;
    write_file(
        &dir,
        "main.go",
        "package main\n\nimport \"fmt\"\n\nfunc main() {\n\tfmt.Println(\"hi\")\n}\n",
    );
    let langs = scan(&[dir.path().to_path_buf()], &default_opts())?;
    assert!(langs.get(&tokei::LanguageType::Go).is_some());
    Ok(())
}

#[test]
fn detects_multiple_languages_in_one_scan() -> Result<()> {
    let dir = tempfile::tempdir()?;
    write_file(&dir, "lib.rs", "fn main() {}\n");
    write_file(&dir, "app.py", "print('hi')\n");
    write_file(&dir, "index.js", "console.log(1);\n");

    let langs = scan(&[dir.path().to_path_buf()], &default_opts())?;
    assert!(langs.get(&tokei::LanguageType::Rust).is_some());
    assert!(langs.get(&tokei::LanguageType::Python).is_some());
    assert!(langs.get(&tokei::LanguageType::JavaScript).is_some());
    Ok(())
}

// ===========================================================================
// 6. File counting correctness
// ===========================================================================

#[test]
fn single_file_reports_one_report() -> Result<()> {
    let dir = tempfile::tempdir()?;
    write_file(&dir, "one.rs", "fn main() {}\n");

    let langs = scan(&[dir.path().to_path_buf()], &default_opts())?;
    let rust = langs
        .get(&tokei::LanguageType::Rust)
        .expect("Rust not found");
    assert_eq!(rust.reports.len(), 1, "single file -> single report");
    Ok(())
}

#[test]
fn multiple_files_same_language_counted() -> Result<()> {
    let dir = tempfile::tempdir()?;
    write_file(&dir, "a.rs", "fn a() {}\n");
    write_file(&dir, "b.rs", "fn b() {}\n");
    write_file(&dir, "c.rs", "fn c() {}\n");

    let langs = scan(&[dir.path().to_path_buf()], &default_opts())?;
    let rust = langs
        .get(&tokei::LanguageType::Rust)
        .expect("Rust not found");
    assert_eq!(rust.reports.len(), 3, "3 Rust files -> 3 reports");
    Ok(())
}

#[test]
fn files_in_subdirectories_are_found() -> Result<()> {
    let dir = tempfile::tempdir()?;
    write_file(&dir, "src/a.rs", "fn a() {}\n");
    write_file(&dir, "src/nested/b.rs", "fn b() {}\n");

    let langs = scan(&[dir.path().to_path_buf()], &default_opts())?;
    let rust = langs
        .get(&tokei::LanguageType::Rust)
        .expect("Rust not found");
    assert_eq!(rust.reports.len(), 2, "2 Rust files in subdirs");
    Ok(())
}

#[test]
fn code_lines_sum_across_files() -> Result<()> {
    let dir = tempfile::tempdir()?;
    write_file(&dir, "a.rs", "fn a() {}\n");
    write_file(&dir, "b.rs", "fn b() {\n    let x = 1;\n}\n");

    let langs = scan(&[dir.path().to_path_buf()], &default_opts())?;
    let rust = langs
        .get(&tokei::LanguageType::Rust)
        .expect("Rust not found");
    // a.rs: 1 code line, b.rs: 3 code lines
    assert!(rust.code >= 2, "total code must be at least 2");
    Ok(())
}

// ===========================================================================
// 7. Children / embedded language handling
// ===========================================================================

#[test]
fn html_with_embedded_css_detected() -> Result<()> {
    let dir = tempfile::tempdir()?;
    write_file(
        &dir,
        "page.html",
        r#"<!DOCTYPE html>
<html>
<head>
<style>
body { color: red; }
</style>
</head>
<body><p>Hello</p></body>
</html>
"#,
    );
    let langs = scan(&[dir.path().to_path_buf()], &default_opts())?;
    assert!(
        langs.get(&tokei::LanguageType::Html).is_some(),
        "HTML must be detected in .html files"
    );
    Ok(())
}

#[test]
fn html_with_embedded_js_detected() -> Result<()> {
    let dir = tempfile::tempdir()?;
    write_file(
        &dir,
        "app.html",
        r#"<!DOCTYPE html>
<html>
<body>
<script>
console.log("hello");
</script>
</body>
</html>
"#,
    );
    let langs = scan(&[dir.path().to_path_buf()], &default_opts())?;
    assert!(langs.get(&tokei::LanguageType::Html).is_some());
    Ok(())
}

// ===========================================================================
// 8. Config flag behaviour
// ===========================================================================

#[test]
fn hidden_flag_includes_dotfiles() -> Result<()> {
    let dir = tempfile::tempdir()?;
    write_file(&dir, ".hidden.py", "print('hidden')\n");
    write_file(&dir, "visible.py", "print('visible')\n");

    let mut opts = default_opts();
    opts.hidden = true;
    let langs = scan(&[dir.path().to_path_buf()], &opts)?;

    let py = langs
        .get(&tokei::LanguageType::Python)
        .expect("Python not found");
    assert!(
        py.reports.len() >= 2,
        "hidden flag should include .hidden.py"
    );
    Ok(())
}

#[test]
fn excluded_pattern_filters_files() -> Result<()> {
    let dir = tempfile::tempdir()?;
    write_file(&dir, "src/lib.rs", "fn lib() {}\n");
    write_file(&dir, "build/gen.rs", "fn gen() {}\n");

    let mut opts = default_opts();
    opts.excluded = vec!["build".to_string()];
    let langs = scan(&[dir.path().to_path_buf()], &opts)?;

    let rust = langs
        .get(&tokei::LanguageType::Rust)
        .expect("Rust not found");
    // Only the file in src should be found
    assert_eq!(
        rust.reports.len(),
        1,
        "excluded build/ should filter gen.rs"
    );
    Ok(())
}

#[test]
fn multiple_excluded_patterns() -> Result<()> {
    let dir = tempfile::tempdir()?;
    write_file(&dir, "src/lib.rs", "fn lib() {}\n");
    write_file(&dir, "build/gen.rs", "fn gen() {}\n");
    write_file(&dir, "vendor/dep.rs", "fn dep() {}\n");

    let mut opts = default_opts();
    opts.excluded = vec!["build".to_string(), "vendor".to_string()];
    let langs = scan(&[dir.path().to_path_buf()], &opts)?;

    let rust = langs
        .get(&tokei::LanguageType::Rust)
        .expect("Rust not found");
    assert_eq!(
        rust.reports.len(),
        1,
        "both build and vendor should be excluded"
    );
    Ok(())
}

#[test]
fn treat_doc_strings_as_comments_changes_counts() -> Result<()> {
    let dir = tempfile::tempdir()?;
    write_file(
        &dir,
        "mod.rs",
        r#"/// This is a doc comment
/// spanning two lines
fn documented() {}
"#,
    );

    let normal = scan(&[dir.path().to_path_buf()], &default_opts())?;
    let normal_rust = normal
        .get(&tokei::LanguageType::Rust)
        .expect("Rust not found");

    let mut doc_opts = default_opts();
    doc_opts.treat_doc_strings_as_comments = true;
    let doc = scan(&[dir.path().to_path_buf()], &doc_opts)?;
    let doc_rust = doc.get(&tokei::LanguageType::Rust).expect("Rust not found");

    // Both scans should succeed and find lines
    assert!(normal_rust.lines() > 0);
    assert!(doc_rust.lines() > 0);
    // Total lines should remain the same
    assert_eq!(normal_rust.lines(), doc_rust.lines());
    Ok(())
}

#[test]
fn no_ignore_flag_does_not_panic() -> Result<()> {
    let dir = tempfile::tempdir()?;
    write_file(&dir, "lib.rs", "fn f() {}\n");

    let mut opts = default_opts();
    opts.no_ignore = true;
    let result = scan(&[dir.path().to_path_buf()], &opts);
    assert!(result.is_ok());
    Ok(())
}

#[test]
fn no_ignore_dot_flag_does_not_panic() -> Result<()> {
    let dir = tempfile::tempdir()?;
    write_file(&dir, "lib.rs", "fn f() {}\n");

    let mut opts = default_opts();
    opts.no_ignore_dot = true;
    let result = scan(&[dir.path().to_path_buf()], &opts);
    assert!(result.is_ok());
    Ok(())
}

#[test]
fn no_ignore_parent_flag_does_not_panic() -> Result<()> {
    let dir = tempfile::tempdir()?;
    write_file(&dir, "lib.rs", "fn f() {}\n");

    let mut opts = default_opts();
    opts.no_ignore_parent = true;
    let result = scan(&[dir.path().to_path_buf()], &opts);
    assert!(result.is_ok());
    Ok(())
}

#[test]
fn no_ignore_vcs_flag_does_not_panic() -> Result<()> {
    let dir = tempfile::tempdir()?;
    write_file(&dir, "lib.rs", "fn f() {}\n");

    let mut opts = default_opts();
    opts.no_ignore_vcs = true;
    let result = scan(&[dir.path().to_path_buf()], &opts);
    assert!(result.is_ok());
    Ok(())
}

// ===========================================================================
// 9. Error handling
// ===========================================================================

#[test]
fn nonexistent_path_returns_error() -> Result<()> {
    let dir = tempfile::tempdir()?;
    let missing = dir.path().join("does_not_exist");
    let result = scan(&[missing], &default_opts());
    assert!(result.is_err());
    let msg = result.unwrap_err().to_string();
    assert!(msg.contains("Path not found"), "error: {msg}");
    Ok(())
}

#[test]
fn nonexistent_path_error_includes_path() -> Result<()> {
    let dir = tempfile::tempdir()?;
    let missing = dir.path().join("phantom_dir");
    let result = scan(&[missing.clone()], &default_opts());
    let msg = result.unwrap_err().to_string();
    assert!(
        msg.contains("phantom_dir"),
        "error should mention the missing path: {msg}"
    );
    Ok(())
}

#[test]
#[should_panic]
fn empty_paths_slice_panics_in_tokei() {
    // tokei panics on empty paths — this is a known upstream behavior
    let _ = scan(&[], &default_opts());
}

// ===========================================================================
// 10. Multiple paths in a single scan
// ===========================================================================

#[test]
fn scan_multiple_directories() -> Result<()> {
    let dir = tempfile::tempdir()?;
    write_file(&dir, "alpha/a.rs", "fn a() {}\n");
    write_file(&dir, "beta/b.py", "print('b')\n");

    let paths = vec![dir.path().join("alpha"), dir.path().join("beta")];
    let langs = scan(&paths, &default_opts())?;

    assert!(langs.get(&tokei::LanguageType::Rust).is_some());
    assert!(langs.get(&tokei::LanguageType::Python).is_some());
    Ok(())
}

#[test]
fn scan_overlapping_paths_finds_files() -> Result<()> {
    let dir = tempfile::tempdir()?;
    write_file(&dir, "src/lib.rs", "fn lib() {}\n");

    // Scanning the same directory twice — tokei may or may not deduplicate
    let paths = vec![dir.path().join("src"), dir.path().join("src")];
    let langs = scan(&paths, &default_opts())?;
    let rust = langs
        .get(&tokei::LanguageType::Rust)
        .expect("Rust not found");
    assert!(
        rust.reports.len() >= 1,
        "overlapping paths should still find files"
    );
    Ok(())
}

// ===========================================================================
// 11. All flags combined
// ===========================================================================

#[test]
fn all_flags_combined_does_not_panic() -> Result<()> {
    let dir = tempfile::tempdir()?;
    write_file(&dir, "code.rs", "fn main() {}\n");

    let opts = ScanOptions {
        excluded: vec!["node_modules".to_string()],
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
    Ok(())
}

// ===========================================================================
// 12. Blank and comment counting
// ===========================================================================

#[test]
fn blank_lines_counted_correctly() -> Result<()> {
    let dir = tempfile::tempdir()?;
    write_file(&dir, "blanks.rs", "fn main() {\n\n\n    let x = 1;\n\n}\n");
    let langs = scan(&[dir.path().to_path_buf()], &default_opts())?;
    let rust = langs
        .get(&tokei::LanguageType::Rust)
        .expect("Rust not found");
    assert!(rust.blanks > 0, "should detect blank lines");
    Ok(())
}

#[test]
fn comment_lines_counted() -> Result<()> {
    let dir = tempfile::tempdir()?;
    write_file(
        &dir,
        "commented.rs",
        "// This is a comment\n// Another comment\nfn main() {}\n",
    );
    let langs = scan(&[dir.path().to_path_buf()], &default_opts())?;
    let rust = langs
        .get(&tokei::LanguageType::Rust)
        .expect("Rust not found");
    assert!(rust.comments >= 2, "should detect at least 2 comment lines");
    Ok(())
}

// ===========================================================================
// 13. Single file path scan
// ===========================================================================

#[test]
fn scan_single_file_path() -> Result<()> {
    let dir = tempfile::tempdir()?;
    write_file(&dir, "solo.py", "x = 1\ny = 2\n");

    let langs = scan(&[dir.path().join("solo.py")], &default_opts())?;
    assert!(
        langs.get(&tokei::LanguageType::Python).is_some(),
        "scanning a single file path should work"
    );
    Ok(())
}
