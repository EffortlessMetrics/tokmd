//! Behavioral tests for the scan adapter.
//!
//! Covers non-empty results, empty directories, language detection,
//! count consistency, and determinism.

use anyhow::Result;
use std::fs;
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

/// Path to the workspace root (parent of `crates/`).
fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|p| p.parent())
        .expect("workspace root")
        .to_path_buf()
}

// ========================
// Non-empty results
// ========================

#[test]
fn scanning_workspace_produces_non_empty_results() -> Result<()> {
    let langs = scan(&[workspace_root()], &default_opts())?;
    assert!(!langs.is_empty(), "workspace scan must find at least one language");
    Ok(())
}

#[test]
fn scanning_own_crate_finds_rust() -> Result<()> {
    let src = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src");
    let langs = scan(&[src], &default_opts())?;
    assert!(
        langs.get(&tokei::LanguageType::Rust).is_some(),
        "must detect Rust in own src/"
    );
    Ok(())
}

// ========================
// Empty directory
// ========================

#[test]
fn scanning_empty_directory_produces_empty_results() -> Result<()> {
    let tmp = tempfile::tempdir()?;
    let langs = scan(&[tmp.path().to_path_buf()], &default_opts())?;
    // Languages that have zero files are not present in the map
    let total_files: usize = langs.values().map(|l| l.reports.len()).sum();
    assert_eq!(total_files, 0, "empty dir must produce zero file reports");
    Ok(())
}

#[test]
fn scanning_dir_with_only_unknown_files_produces_no_known_code() -> Result<()> {
    let tmp = tempfile::tempdir()?;
    fs::write(tmp.path().join("data.xyz_unknown"), "some data")?;
    fs::write(tmp.path().join("notes.random_ext"), "notes")?;
    let langs = scan(&[tmp.path().to_path_buf()], &default_opts())?;
    let total_code: usize = langs.values().map(|l| l.code).sum();
    assert_eq!(total_code, 0, "unknown extensions should not count as code");
    Ok(())
}

// ========================
// Language detection accuracy
// ========================

#[test]
fn detects_rust_for_rs_extension() -> Result<()> {
    let tmp = tempfile::tempdir()?;
    fs::write(tmp.path().join("main.rs"), "fn main() {}\n")?;
    let langs = scan(&[tmp.path().to_path_buf()], &default_opts())?;
    assert!(langs.get(&tokei::LanguageType::Rust).is_some());
    Ok(())
}

#[test]
fn detects_python_for_py_extension() -> Result<()> {
    let tmp = tempfile::tempdir()?;
    fs::write(tmp.path().join("app.py"), "print('hello')\n")?;
    let langs = scan(&[tmp.path().to_path_buf()], &default_opts())?;
    assert!(langs.get(&tokei::LanguageType::Python).is_some());
    Ok(())
}

#[test]
fn detects_javascript_for_js_extension() -> Result<()> {
    let tmp = tempfile::tempdir()?;
    fs::write(tmp.path().join("index.js"), "console.log('hi');\n")?;
    let langs = scan(&[tmp.path().to_path_buf()], &default_opts())?;
    assert!(langs.get(&tokei::LanguageType::JavaScript).is_some());
    Ok(())
}

#[test]
fn detects_toml_for_toml_extension() -> Result<()> {
    let tmp = tempfile::tempdir()?;
    fs::write(tmp.path().join("config.toml"), "[section]\nkey = \"val\"\n")?;
    let langs = scan(&[tmp.path().to_path_buf()], &default_opts())?;
    assert!(langs.get(&tokei::LanguageType::Toml).is_some());
    Ok(())
}

#[test]
fn detects_json_for_json_extension() -> Result<()> {
    let tmp = tempfile::tempdir()?;
    fs::write(tmp.path().join("data.json"), "{\"a\": 1}\n")?;
    let langs = scan(&[tmp.path().to_path_buf()], &default_opts())?;
    assert!(langs.get(&tokei::LanguageType::Json).is_some());
    Ok(())
}

#[test]
fn detects_markdown_for_md_extension() -> Result<()> {
    let tmp = tempfile::tempdir()?;
    fs::write(tmp.path().join("readme.md"), "# Title\n\nBody text.\n")?;
    let langs = scan(&[tmp.path().to_path_buf()], &default_opts())?;
    assert!(langs.get(&tokei::LanguageType::Markdown).is_some());
    Ok(())
}

// ========================
// Count consistency
// ========================

#[test]
fn lines_gte_code_plus_comments_plus_blanks() -> Result<()> {
    let src = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src");
    let langs = scan(&[src], &default_opts())?;
    for (lang_type, lang) in &langs {
        let total = lang.code + lang.comments + lang.blanks;
        assert!(
            lang.lines() >= total,
            "{:?}: lines ({}) < code+comments+blanks ({})",
            lang_type,
            lang.lines(),
            total
        );
    }
    Ok(())
}

#[test]
fn all_counts_are_non_negative_for_known_source() -> Result<()> {
    let tmp = tempfile::tempdir()?;
    fs::write(
        tmp.path().join("sample.rs"),
        "// comment\nfn main() {\n    println!(\"hi\");\n}\n\n",
    )?;
    let langs = scan(&[tmp.path().to_path_buf()], &default_opts())?;
    let rust = langs
        .get(&tokei::LanguageType::Rust)
        .expect("should find Rust");
    // usize is always >= 0, but verify non-zero for meaningful source
    assert!(rust.code > 0, "code lines must be positive");
    assert!(rust.lines() > 0, "total lines must be positive");
    Ok(())
}

#[test]
fn comment_and_blank_counts_for_known_input() -> Result<()> {
    let tmp = tempfile::tempdir()?;
    // 1 comment, 3 code lines, 1 blank
    fs::write(
        tmp.path().join("demo.rs"),
        "// a comment\nfn main() {\n    let x = 1;\n}\n\n",
    )?;
    let langs = scan(&[tmp.path().to_path_buf()], &default_opts())?;
    let rust = langs
        .get(&tokei::LanguageType::Rust)
        .expect("should find Rust");
    assert!(rust.comments >= 1, "should have at least 1 comment line");
    assert!(rust.blanks >= 1, "should have at least 1 blank line");
    assert!(rust.code >= 3, "should have at least 3 code lines");
    Ok(())
}

// ========================
// Determinism
// ========================

#[test]
fn two_scans_of_same_directory_produce_identical_results() -> Result<()> {
    let src = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src");
    let opts = default_opts();

    let run1 = scan(&[src.clone()], &opts)?;
    let run2 = scan(&[src], &opts)?;

    // Same set of languages detected
    let keys1: Vec<_> = run1.keys().collect();
    let keys2: Vec<_> = run2.keys().collect();
    assert_eq!(keys1, keys2, "detected languages must be identical");

    // Same counts for every language
    for lang_type in run1.keys() {
        let a = &run1[lang_type];
        let b = &run2[lang_type];
        assert_eq!(a.code, b.code, "{:?} code mismatch", lang_type);
        assert_eq!(a.comments, b.comments, "{:?} comments mismatch", lang_type);
        assert_eq!(a.blanks, b.blanks, "{:?} blanks mismatch", lang_type);
        assert_eq!(
            a.reports.len(),
            b.reports.len(),
            "{:?} file count mismatch",
            lang_type
        );
    }
    Ok(())
}

#[test]
fn determinism_with_synthetic_files() -> Result<()> {
    let tmp = tempfile::tempdir()?;
    fs::write(tmp.path().join("a.py"), "x = 1\ny = 2\n")?;
    fs::write(tmp.path().join("b.rs"), "fn f() {}\n")?;
    let opts = default_opts();

    let run1 = scan(&[tmp.path().to_path_buf()], &opts)?;
    let run2 = scan(&[tmp.path().to_path_buf()], &opts)?;

    for lang_type in run1.keys() {
        let a = &run1[lang_type];
        let b = &run2[lang_type];
        assert_eq!(a.code, b.code, "{:?} code mismatch on synthetic", lang_type);
        assert_eq!(a.blanks, b.blanks, "{:?} blanks mismatch on synthetic", lang_type);
    }
    Ok(())
}
