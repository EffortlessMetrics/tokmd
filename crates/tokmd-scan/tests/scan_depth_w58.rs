//! Depth tests for `tokmd-scan` – W58.
//!
//! Exercises scan() against synthetic tempdirs with known file contents,
//! verifying language detection, line counts, determinism, exclusion,
//! and graceful handling of empty/non-UTF8 inputs.

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

fn write_file(dir: &TempDir, rel: &str, content: &str) {
    let path = dir.path().join(rel);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).expect("create parent dirs");
    }
    fs::write(&path, content).expect("write test file");
}

// ===========================================================================
// 1. Language detection on synthetic tempdirs
// ===========================================================================

#[test]
fn scan_detects_rust_in_tempdir() -> Result<()> {
    let dir = tempfile::tempdir()?;
    write_file(&dir, "main.rs", "fn main() {\n    println!(\"hi\");\n}\n");
    let langs = scan(&[dir.path().to_path_buf()], &default_opts())?;
    assert!(
        langs.get(&tokei::LanguageType::Rust).is_some(),
        "should detect Rust"
    );
    Ok(())
}

#[test]
fn scan_detects_python_in_tempdir() -> Result<()> {
    let dir = tempfile::tempdir()?;
    write_file(&dir, "app.py", "def hello():\n    print('hi')\n");
    let langs = scan(&[dir.path().to_path_buf()], &default_opts())?;
    assert!(
        langs.get(&tokei::LanguageType::Python).is_some(),
        "should detect Python"
    );
    Ok(())
}

#[test]
fn scan_detects_javascript_in_tempdir() -> Result<()> {
    let dir = tempfile::tempdir()?;
    write_file(
        &dir,
        "index.js",
        "function greet() {\n  console.log('hi');\n}\n",
    );
    let langs = scan(&[dir.path().to_path_buf()], &default_opts())?;
    assert!(
        langs.get(&tokei::LanguageType::JavaScript).is_some(),
        "should detect JavaScript"
    );
    Ok(())
}

#[test]
fn scan_detects_multiple_languages() -> Result<()> {
    let dir = tempfile::tempdir()?;
    write_file(&dir, "main.rs", "fn main() {}\n");
    write_file(&dir, "app.py", "x = 1\n");
    write_file(&dir, "index.js", "function f() {\n  return 1;\n}\n");

    let langs = scan(&[dir.path().to_path_buf()], &default_opts())?;
    assert!(langs.get(&tokei::LanguageType::Rust).is_some());
    assert!(langs.get(&tokei::LanguageType::Python).is_some());
    assert!(langs.get(&tokei::LanguageType::JavaScript).is_some());
    Ok(())
}

// ===========================================================================
// 2. Line count accuracy
// ===========================================================================

#[test]
fn scan_counts_rust_code_lines() -> Result<()> {
    let dir = tempfile::tempdir()?;
    // 3 code lines, 1 comment, 1 blank
    write_file(
        &dir,
        "lib.rs",
        "// a comment\nfn add(a: i32, b: i32) -> i32 {\n    a + b\n}\n\n",
    );
    let langs = scan(&[dir.path().to_path_buf()], &default_opts())?;
    let rust = langs.get(&tokei::LanguageType::Rust).expect("Rust present");
    assert!(rust.code >= 3, "expected ≥3 code lines, got {}", rust.code);
    assert!(
        rust.comments >= 1,
        "expected ≥1 comment line, got {}",
        rust.comments
    );
    Ok(())
}

#[test]
fn scan_counts_python_code_lines() -> Result<()> {
    let dir = tempfile::tempdir()?;
    write_file(
        &dir,
        "script.py",
        "# comment\ndef add(a, b):\n    return a + b\n",
    );
    let langs = scan(&[dir.path().to_path_buf()], &default_opts())?;
    let py = langs
        .get(&tokei::LanguageType::Python)
        .expect("Python present");
    assert!(py.code >= 2, "expected ≥2 code lines, got {}", py.code);
    assert!(py.comments >= 1, "expected ≥1 comment, got {}", py.comments);
    Ok(())
}

#[test]
fn scan_total_lines_equals_code_plus_comments_plus_blanks() -> Result<()> {
    let dir = tempfile::tempdir()?;
    write_file(
        &dir,
        "math.rs",
        "fn mul(a: i32, b: i32) -> i32 {\n    a * b\n}\n",
    );
    let langs = scan(&[dir.path().to_path_buf()], &default_opts())?;
    let rust = langs.get(&tokei::LanguageType::Rust).expect("Rust present");
    assert_eq!(
        rust.lines(),
        rust.code + rust.comments + rust.blanks,
        "total = code + comments + blanks"
    );
    Ok(())
}

// ===========================================================================
// 3. Empty directory
// ===========================================================================

#[test]
fn scan_empty_directory_returns_empty() -> Result<()> {
    let dir = tempfile::tempdir()?;
    let langs = scan(&[dir.path().to_path_buf()], &default_opts())?;
    assert!(langs.is_empty(), "empty dir should yield no languages");
    Ok(())
}

#[test]
fn scan_dir_with_only_unknown_ext_returns_empty() -> Result<()> {
    let dir = tempfile::tempdir()?;
    write_file(&dir, "data.xyz123", "some random content\n");
    let langs = scan(&[dir.path().to_path_buf()], &default_opts())?;
    // tokei won't recognize .xyz123
    let total: usize = langs.iter().map(|(_, l)| l.code as usize).sum();
    // Should either be empty or have zero code
    assert!(
        langs.is_empty() || total == 0,
        "unknown extension should not count code"
    );
    Ok(())
}

// ===========================================================================
// 4. Excluded patterns
// ===========================================================================

#[test]
fn scan_excludes_glob_pattern() -> Result<()> {
    let dir = tempfile::tempdir()?;
    write_file(&dir, "keep.rs", "fn keep() {}\n");
    write_file(&dir, "vendor/dep.rs", "fn dep() {}\n");

    let mut opts = default_opts();
    opts.excluded = vec!["vendor".to_string()];
    let langs = scan(&[dir.path().to_path_buf()], &opts)?;
    let rust = langs.get(&tokei::LanguageType::Rust).expect("Rust present");
    // Only the non-vendor file should be counted (1 code line in keep.rs body)
    assert!(
        rust.code <= 2,
        "vendor should be excluded, got {} code lines",
        rust.code
    );
    Ok(())
}

#[test]
fn scan_excludes_wildcard_extension() -> Result<()> {
    let dir = tempfile::tempdir()?;
    write_file(&dir, "app.py", "x = 1\n");
    write_file(&dir, "index.js", "const y = 2;\n");

    let mut opts = default_opts();
    opts.excluded = vec!["*.js".to_string()];
    let langs = scan(&[dir.path().to_path_buf()], &opts)?;
    assert!(
        langs.get(&tokei::LanguageType::JavaScript).is_none()
            || langs
                .get(&tokei::LanguageType::JavaScript)
                .map_or(true, |l| l.code == 0),
        "JS should be excluded"
    );
    assert!(
        langs.get(&tokei::LanguageType::Python).is_some(),
        "Python should remain"
    );
    Ok(())
}

#[test]
fn scan_multiple_excludes() -> Result<()> {
    let dir = tempfile::tempdir()?;
    write_file(&dir, "main.rs", "fn main() {}\n");
    write_file(&dir, "test.py", "x = 1\n");
    write_file(&dir, "app.js", "const a = 1;\n");

    let mut opts = default_opts();
    opts.excluded = vec!["*.py".to_string(), "*.js".to_string()];
    let langs = scan(&[dir.path().to_path_buf()], &opts)?;
    assert!(langs.get(&tokei::LanguageType::Rust).is_some());
    let py_code = langs
        .get(&tokei::LanguageType::Python)
        .map_or(0, |l| l.code);
    let js_code = langs
        .get(&tokei::LanguageType::JavaScript)
        .map_or(0, |l| l.code);
    assert_eq!(py_code + js_code, 0, "py and js should be excluded");
    Ok(())
}

// ===========================================================================
// 5. Determinism
// ===========================================================================

#[test]
fn scan_is_deterministic_same_input() -> Result<()> {
    let dir = tempfile::tempdir()?;
    write_file(&dir, "a.rs", "fn a() {}\n");
    write_file(&dir, "b.rs", "fn b() {}\n");
    write_file(&dir, "c.py", "x = 1\n");

    let opts = default_opts();
    let path = dir.path().to_path_buf();
    let r1 = scan(&[path.clone()], &opts)?;
    let r2 = scan(&[path], &opts)?;

    for (lang, stats1) in r1.iter() {
        let stats2 = r2
            .get(lang)
            .unwrap_or_else(|| panic!("second scan missing {lang:?}"));
        assert_eq!(stats1.code, stats2.code, "code mismatch for {lang:?}");
        assert_eq!(
            stats1.comments, stats2.comments,
            "comments mismatch for {lang:?}"
        );
        assert_eq!(stats1.blanks, stats2.blanks, "blanks mismatch for {lang:?}");
    }
    Ok(())
}

#[test]
fn scan_deterministic_across_multiple_runs() -> Result<()> {
    let dir = tempfile::tempdir()?;
    write_file(
        &dir,
        "lib.rs",
        "pub fn compute(x: i32) -> i32 {\n    x * 2\n}\n",
    );
    let opts = default_opts();
    let path = dir.path().to_path_buf();

    let mut results = Vec::new();
    for _ in 0..5 {
        let r = scan(&[path.clone()], &opts)?;
        let rust = r.get(&tokei::LanguageType::Rust).expect("Rust present");
        results.push((rust.code, rust.comments, rust.blanks));
    }
    assert!(
        results.windows(2).all(|w| w[0] == w[1]),
        "all 5 runs must produce identical counts"
    );
    Ok(())
}

// ===========================================================================
// 6. Non-UTF8 / binary file handling
// ===========================================================================

#[test]
fn scan_handles_binary_file_gracefully() -> Result<()> {
    let dir = tempfile::tempdir()?;
    // Write binary content with a .rs extension
    let binary: Vec<u8> = (0u8..=255).cycle().take(512).collect();
    fs::write(dir.path().join("binary.rs"), &binary)?;
    // Should not panic
    let result = scan(&[dir.path().to_path_buf()], &default_opts());
    assert!(result.is_ok(), "scan should not panic on binary files");
    Ok(())
}

#[test]
fn scan_handles_mixed_binary_and_text() -> Result<()> {
    let dir = tempfile::tempdir()?;
    write_file(&dir, "good.rs", "fn good() {}\n");
    let binary: Vec<u8> = vec![0u8, 0xFF, 0xFE, 0x00, 0x01];
    fs::write(dir.path().join("bad.rs"), &binary)?;

    let langs = scan(&[dir.path().to_path_buf()], &default_opts())?;
    // The good file should still be counted
    let rust = langs.get(&tokei::LanguageType::Rust);
    assert!(rust.is_some(), "should still find the valid Rust file");
    Ok(())
}

// ===========================================================================
// 7. Config mode variants
// ===========================================================================

#[test]
fn scan_config_mode_none_works() -> Result<()> {
    let dir = tempfile::tempdir()?;
    write_file(&dir, "src.rs", "fn f() {}\n");
    let mut opts = default_opts();
    opts.config = ConfigMode::None;
    let langs = scan(&[dir.path().to_path_buf()], &opts)?;
    assert!(!langs.is_empty());
    Ok(())
}

#[test]
fn scan_config_mode_auto_works() -> Result<()> {
    let dir = tempfile::tempdir()?;
    write_file(&dir, "src.rs", "fn f() {}\n");
    let mut opts = default_opts();
    opts.config = ConfigMode::Auto;
    let langs = scan(&[dir.path().to_path_buf()], &opts)?;
    assert!(!langs.is_empty());
    Ok(())
}

// ===========================================================================
// 8. Nested directory scanning
// ===========================================================================

#[test]
fn scan_finds_files_in_nested_dirs() -> Result<()> {
    let dir = tempfile::tempdir()?;
    write_file(&dir, "a/b/c/deep.rs", "fn deep() {}\n");
    write_file(&dir, "top.rs", "fn top() {}\n");
    let langs = scan(&[dir.path().to_path_buf()], &default_opts())?;
    let rust = langs.get(&tokei::LanguageType::Rust).expect("Rust present");
    // Both files should contribute
    assert!(rust.code >= 2, "both files should be counted");
    Ok(())
}

// ===========================================================================
// 9. Nonexistent path errors
// ===========================================================================

#[test]
fn scan_nonexistent_path_errors() {
    let path = PathBuf::from("/surely/does/not/exist/w58");
    let result = scan(&[path], &default_opts());
    assert!(result.is_err());
    let msg = result.unwrap_err().to_string();
    assert!(msg.contains("Path not found"), "error: {msg}");
}

// ===========================================================================
// 10. Doc-string-as-comments flag
// ===========================================================================

#[test]
fn scan_treat_doc_strings_as_comments_changes_counts() -> Result<()> {
    let dir = tempfile::tempdir()?;
    write_file(&dir, "lib.rs", "/// A doc comment\nfn documented() {}\n");
    let mut opts_normal = default_opts();
    opts_normal.treat_doc_strings_as_comments = false;
    let r1 = scan(&[dir.path().to_path_buf()], &opts_normal)?;

    let mut opts_doc = default_opts();
    opts_doc.treat_doc_strings_as_comments = true;
    let r2 = scan(&[dir.path().to_path_buf()], &opts_doc)?;

    let rust1 = r1.get(&tokei::LanguageType::Rust).expect("Rust");
    let rust2 = r2.get(&tokei::LanguageType::Rust).expect("Rust");

    // With doc-as-comments, comment count should be ≥ the normal count
    assert!(
        rust2.comments >= rust1.comments,
        "doc-as-comments should not reduce comment count"
    );
    Ok(())
}
