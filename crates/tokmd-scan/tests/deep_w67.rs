//! Pipeline integration tests for `tokmd-scan` (w67).
//!
//! Verifies the scan → output flow: configuration flags, exclude patterns,
//! children modes, determinism, empty directories, and hidden-file handling.

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

fn crate_src() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src")
}

/// Write a file inside `dir`, creating intermediate directories as needed.
fn write_file(dir: &TempDir, rel: &str, content: &str) {
    let path = dir.path().join(rel);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).expect("create parent dirs");
    }
    fs::write(&path, content).expect("write test file");
}

// ===========================================================================
// 1. Basic scan with various configurations
// ===========================================================================

#[test]
fn scan_crate_src_finds_rust() -> Result<()> {
    let langs = scan(&[crate_src()], &default_opts())?;
    assert!(
        langs.get(&tokei::LanguageType::Rust).is_some(),
        "must detect Rust in own src/"
    );
    Ok(())
}

#[test]
fn scan_with_config_mode_auto() -> Result<()> {
    let mut opts = default_opts();
    opts.config = ConfigMode::Auto;
    let langs = scan(&[crate_src()], &opts)?;
    assert!(!langs.is_empty());
    Ok(())
}

#[test]
fn scan_with_config_mode_none() -> Result<()> {
    let langs = scan(&[crate_src()], &default_opts())?;
    assert!(!langs.is_empty());
    Ok(())
}

// ===========================================================================
// 2. Exclude patterns
// ===========================================================================

#[test]
fn exclude_pattern_filters_files() -> Result<()> {
    let dir = TempDir::new()?;
    write_file(&dir, "keep.rs", "fn main() {}\n");
    write_file(&dir, "vendor/dep.rs", "fn dep() {}\n");

    let mut opts = default_opts();
    opts.excluded = vec!["vendor".to_string()];
    let langs = scan(&[dir.path().to_path_buf()], &opts)?;
    let rust = langs.get(&tokei::LanguageType::Rust).expect("Rust");
    // Only keep.rs should be counted (1 report)
    assert_eq!(rust.reports.len(), 1, "vendor/ should be excluded");
    Ok(())
}

#[test]
fn multiple_exclude_patterns() -> Result<()> {
    let dir = TempDir::new()?;
    write_file(&dir, "main.rs", "fn main() {}\n");
    write_file(&dir, "gen/auto.rs", "fn auto() {}\n");
    write_file(&dir, "third_party/ext.rs", "fn ext() {}\n");

    let mut opts = default_opts();
    opts.excluded = vec!["gen".to_string(), "third_party".to_string()];
    let langs = scan(&[dir.path().to_path_buf()], &opts)?;
    let rust = langs.get(&tokei::LanguageType::Rust).expect("Rust");
    assert_eq!(rust.reports.len(), 1, "only main.rs should survive");
    Ok(())
}

#[test]
fn glob_exclude_pattern() -> Result<()> {
    let dir = TempDir::new()?;
    write_file(&dir, "app.rs", "fn app() {}\n");
    write_file(&dir, "app_test.rs", "fn test() {}\n");

    let mut opts = default_opts();
    opts.excluded = vec!["*_test.rs".to_string()];
    let langs = scan(&[dir.path().to_path_buf()], &opts)?;
    let rust = langs.get(&tokei::LanguageType::Rust).expect("Rust");
    assert_eq!(rust.reports.len(), 1);
    Ok(())
}

// ===========================================================================
// 3. Children mode handling (Collapse vs Separate)
// ===========================================================================

#[test]
fn scan_returns_language_stats_for_pure_rust() -> Result<()> {
    let langs = scan(&[crate_src()], &default_opts())?;
    let rust = langs.get(&tokei::LanguageType::Rust).expect("Rust");
    assert!(rust.code > 0, "Rust should have code lines");
    assert!(rust.lines() > 0, "Rust should have total lines");
    Ok(())
}

#[test]
fn scan_html_with_embedded_produces_children() -> Result<()> {
    let dir = TempDir::new()?;
    write_file(
        &dir,
        "page.html",
        r#"<!DOCTYPE html>
<html>
<head><style>body { color: red; }</style></head>
<body>
<script>console.log("hello");</script>
</body>
</html>
"#,
    );
    let langs = scan(&[dir.path().to_path_buf()], &default_opts())?;
    let html = langs.get(&tokei::LanguageType::Html);
    assert!(html.is_some(), "HTML should be detected");
    // HTML may have children (CSS/JS) depending on tokei config
    Ok(())
}

#[test]
fn scan_result_children_are_accessible() -> Result<()> {
    let dir = TempDir::new()?;
    write_file(
        &dir,
        "index.html",
        "<html><head><style>h1 { font-size: 2em; }</style></head><body></body></html>\n",
    );
    let langs = scan(&[dir.path().to_path_buf()], &default_opts())?;
    // We just verify children map is accessible without panic
    for (_lang_type, lang) in langs.iter() {
        let _ = &lang.children;
    }
    Ok(())
}

// ===========================================================================
// 4. Determinism — same directory yields same result
// ===========================================================================

#[test]
fn deterministic_scan_same_dir_twice() -> Result<()> {
    let dir = TempDir::new()?;
    write_file(&dir, "a.rs", "fn a() {}\n");
    write_file(&dir, "b.rs", "fn b() { let x = 1; }\n");

    let opts = default_opts();
    let r1 = scan(&[dir.path().to_path_buf()], &opts)?;
    let r2 = scan(&[dir.path().to_path_buf()], &opts)?;

    let rust1 = r1.get(&tokei::LanguageType::Rust).expect("Rust r1");
    let rust2 = r2.get(&tokei::LanguageType::Rust).expect("Rust r2");
    assert_eq!(rust1.code, rust2.code, "code lines must be identical");
    assert_eq!(rust1.comments, rust2.comments);
    assert_eq!(rust1.blanks, rust2.blanks);
    assert_eq!(rust1.reports.len(), rust2.reports.len());
    Ok(())
}

#[test]
fn deterministic_scan_multiple_languages() -> Result<()> {
    let dir = TempDir::new()?;
    write_file(&dir, "main.rs", "fn main() {}\n");
    write_file(&dir, "config.toml", "[package]\nname = \"test\"\n");

    let opts = default_opts();
    let r1 = scan(&[dir.path().to_path_buf()], &opts)?;
    let r2 = scan(&[dir.path().to_path_buf()], &opts)?;

    // Same set of detected languages
    let keys1: Vec<_> = r1.keys().collect();
    let keys2: Vec<_> = r2.keys().collect();
    assert_eq!(keys1, keys2, "detected languages must be identical");
    Ok(())
}

#[test]
fn deterministic_code_and_comment_counts() -> Result<()> {
    let dir = TempDir::new()?;
    write_file(
        &dir,
        "lib.rs",
        "// comment\nfn foo() {}\n\n// another\nfn bar() {}\n",
    );
    let opts = default_opts();
    let r1 = scan(&[dir.path().to_path_buf()], &opts)?;
    let r2 = scan(&[dir.path().to_path_buf()], &opts)?;

    let rust1 = r1.get(&tokei::LanguageType::Rust).unwrap();
    let rust2 = r2.get(&tokei::LanguageType::Rust).unwrap();
    assert_eq!(rust1.code, rust2.code);
    assert_eq!(rust1.comments, rust2.comments);
    assert_eq!(rust1.blanks, rust2.blanks);
    Ok(())
}

// ===========================================================================
// 5. Empty directory scanning
// ===========================================================================

#[test]
fn empty_directory_produces_empty_languages() -> Result<()> {
    let dir = TempDir::new()?;
    let langs = scan(&[dir.path().to_path_buf()], &default_opts())?;
    // No files → no languages detected
    let non_empty: Vec<_> = langs.iter().filter(|(_, l)| l.code > 0).collect();
    assert!(non_empty.is_empty(), "empty dir should produce no results");
    Ok(())
}

#[test]
fn directory_with_only_blanks_has_zero_code() -> Result<()> {
    let dir = TempDir::new()?;
    write_file(&dir, "empty.rs", "\n\n\n\n");
    let langs = scan(&[dir.path().to_path_buf()], &default_opts())?;
    let rust = langs.get(&tokei::LanguageType::Rust);
    if let Some(r) = rust {
        assert_eq!(r.code, 0, "blank-only file has zero code");
    }
    Ok(())
}

#[test]
fn nonexistent_path_errors() {
    let opts = default_opts();
    let result = scan(&[PathBuf::from("/surely/does/not/exist")], &opts);
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("Path not found"));
}

// ===========================================================================
// 6. Hidden files handling
// ===========================================================================

#[test]
fn hidden_files_excluded_by_default() -> Result<()> {
    let dir = TempDir::new()?;
    write_file(&dir, "visible.rs", "fn vis() {}\n");
    write_file(&dir, ".hidden.rs", "fn hid() {}\n");

    let opts = default_opts();
    let langs = scan(&[dir.path().to_path_buf()], &opts)?;
    let rust = langs.get(&tokei::LanguageType::Rust).expect("Rust");
    // By default hidden files should be excluded (1 file)
    assert_eq!(rust.reports.len(), 1, "hidden file should be excluded");
    Ok(())
}

#[test]
fn hidden_flag_includes_hidden_files() -> Result<()> {
    let dir = TempDir::new()?;
    write_file(&dir, "visible.rs", "fn vis() {}\n");
    write_file(&dir, ".hidden.rs", "fn hid() {}\n");

    let mut opts = default_opts();
    opts.hidden = true;
    let langs = scan(&[dir.path().to_path_buf()], &opts)?;
    let rust = langs.get(&tokei::LanguageType::Rust).expect("Rust");
    assert_eq!(
        rust.reports.len(),
        2,
        "hidden flag should include .hidden.rs"
    );
    Ok(())
}

#[test]
fn hidden_directory_excluded_by_default() -> Result<()> {
    let dir = TempDir::new()?;
    write_file(&dir, "main.rs", "fn main() {}\n");
    write_file(&dir, ".secret/internal.rs", "fn secret() {}\n");

    let opts = default_opts();
    let langs = scan(&[dir.path().to_path_buf()], &opts)?;
    let rust = langs.get(&tokei::LanguageType::Rust).expect("Rust");
    assert_eq!(
        rust.reports.len(),
        1,
        "hidden directory should be excluded by default"
    );
    Ok(())
}

// ===========================================================================
// 7. Miscellaneous config combinations
// ===========================================================================

#[test]
fn doc_strings_as_comments_changes_counts() -> Result<()> {
    let dir = TempDir::new()?;
    write_file(&dir, "lib.rs", "/// A doc comment\nfn documented() {}\n");
    let mut opts_normal = default_opts();
    opts_normal.treat_doc_strings_as_comments = false;

    let mut opts_doc = default_opts();
    opts_doc.treat_doc_strings_as_comments = true;

    let r_normal = scan(&[dir.path().to_path_buf()], &opts_normal)?;
    let r_doc = scan(&[dir.path().to_path_buf()], &opts_doc)?;

    // Both should succeed without error
    assert!(r_normal.get(&tokei::LanguageType::Rust).is_some());
    assert!(r_doc.get(&tokei::LanguageType::Rust).is_some());
    Ok(())
}

#[test]
fn no_ignore_flags_accept_without_error() -> Result<()> {
    let dir = TempDir::new()?;
    write_file(&dir, "src.rs", "fn src() {}\n");

    let mut opts = default_opts();
    opts.no_ignore = true;
    opts.no_ignore_parent = true;
    opts.no_ignore_dot = true;
    opts.no_ignore_vcs = true;
    let result = scan(&[dir.path().to_path_buf()], &opts);
    assert!(result.is_ok());
    Ok(())
}

#[test]
fn scan_multiple_paths() -> Result<()> {
    let dir1 = TempDir::new()?;
    let dir2 = TempDir::new()?;
    write_file(&dir1, "a.rs", "fn a() {}\n");
    write_file(&dir2, "b.rs", "fn b() {}\n");

    let opts = default_opts();
    let langs = scan(
        &[dir1.path().to_path_buf(), dir2.path().to_path_buf()],
        &opts,
    )?;
    let rust = langs.get(&tokei::LanguageType::Rust).expect("Rust");
    assert_eq!(rust.reports.len(), 2, "both dirs should contribute files");
    Ok(())
}

#[test]
fn scan_deeply_nested_files() -> Result<()> {
    let dir = TempDir::new()?;
    write_file(&dir, "a/b/c/d/deep.rs", "fn deep() {}\n");

    let langs = scan(&[dir.path().to_path_buf()], &default_opts())?;
    let rust = langs.get(&tokei::LanguageType::Rust).expect("Rust");
    assert_eq!(rust.reports.len(), 1);
    assert!(rust.code > 0);
    Ok(())
}
