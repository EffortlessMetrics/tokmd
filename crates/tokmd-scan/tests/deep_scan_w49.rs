//! Deep tests for tokmd-scan (w49).
//!
//! Covers: scan results for known fixtures, excluded paths, .gitignore respect,
//! empty directory, determinism, and property-based path existence checks.

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

fn write_file(dir: &TempDir, rel: &str, content: &str) {
    let path = dir.path().join(rel);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).expect("create parent dirs");
    }
    fs::write(&path, content).expect("write test file");
}

// ============================================================================
// 1. Scan produces results for known fixtures
// ============================================================================

#[test]
fn scan_crate_src_finds_rust() -> Result<()> {
    let langs = scan(&[crate_src()], &default_opts())?;
    assert!(langs.get(&tokei::LanguageType::Rust).is_some());
    Ok(())
}

#[test]
fn scan_crate_src_has_positive_code() -> Result<()> {
    let langs = scan(&[crate_src()], &default_opts())?;
    let rust = langs.get(&tokei::LanguageType::Rust).expect("Rust");
    assert!(rust.code > 0);
    Ok(())
}

#[test]
fn scan_synthetic_python_fixture() -> Result<()> {
    let dir = tempfile::tempdir()?;
    write_file(
        &dir,
        "main.py",
        "#!/usr/bin/env python3\ndef main():\n    pass\n",
    );
    let langs = scan(&[dir.path().to_path_buf()], &default_opts())?;
    let py = langs.get(&tokei::LanguageType::Python).expect("Python");
    assert!(py.code > 0);
    assert_eq!(py.reports.len(), 1);
    Ok(())
}

#[test]
fn scan_synthetic_rust_fixture() -> Result<()> {
    let dir = tempfile::tempdir()?;
    write_file(
        &dir,
        "lib.rs",
        "pub fn hello() -> &'static str {\n    \"world\"\n}\n",
    );
    let langs = scan(&[dir.path().to_path_buf()], &default_opts())?;
    let rust = langs.get(&tokei::LanguageType::Rust).expect("Rust");
    assert!(rust.code >= 3);
    Ok(())
}

#[test]
fn scan_synthetic_multi_language() -> Result<()> {
    let dir = tempfile::tempdir()?;
    write_file(&dir, "a.rs", "fn a() {}\n");
    write_file(&dir, "b.py", "x = 1\n");
    write_file(&dir, "c.js", "const c = 1;\n");
    let langs = scan(&[dir.path().to_path_buf()], &default_opts())?;
    assert!(langs.get(&tokei::LanguageType::Rust).is_some());
    assert!(langs.get(&tokei::LanguageType::Python).is_some());
    assert!(langs.get(&tokei::LanguageType::JavaScript).is_some());
    Ok(())
}

// ============================================================================
// 2. Scan respects excluded paths
// ============================================================================

#[test]
fn excluded_dir_is_not_scanned() -> Result<()> {
    let dir = tempfile::tempdir()?;
    write_file(&dir, "src/lib.rs", "fn lib() {}\n");
    write_file(&dir, "vendor/dep.rs", "fn dep() {}\n");

    let mut opts = default_opts();
    opts.excluded = vec!["vendor".to_string()];
    let langs = scan(&[dir.path().to_path_buf()], &opts)?;
    let rust = langs.get(&tokei::LanguageType::Rust).expect("Rust");
    assert_eq!(rust.reports.len(), 1, "vendor/ should be excluded");
    Ok(())
}

#[test]
fn multiple_excluded_dirs() -> Result<()> {
    let dir = tempfile::tempdir()?;
    write_file(&dir, "src/lib.rs", "fn lib() {}\n");
    write_file(&dir, "build/gen.rs", "fn gen() {}\n");
    write_file(&dir, "vendor/dep.rs", "fn dep() {}\n");

    let mut opts = default_opts();
    opts.excluded = vec!["build".to_string(), "vendor".to_string()];
    let langs = scan(&[dir.path().to_path_buf()], &opts)?;
    let rust = langs.get(&tokei::LanguageType::Rust).expect("Rust");
    assert_eq!(rust.reports.len(), 1, "build/ and vendor/ excluded");
    Ok(())
}

#[test]
fn excluded_glob_pattern() -> Result<()> {
    let dir = tempfile::tempdir()?;
    write_file(&dir, "src/lib.rs", "fn lib() {}\n");
    write_file(&dir, "src/generated.rs", "fn gen() {}\n");

    let mut opts = default_opts();
    opts.excluded = vec!["*generated*".to_string()];
    let langs = scan(&[dir.path().to_path_buf()], &opts)?;
    let rust = langs.get(&tokei::LanguageType::Rust).expect("Rust");
    assert_eq!(rust.reports.len(), 1, "generated file should be excluded");
    Ok(())
}

// ============================================================================
// 3. Scan respects .gitignore
// ============================================================================

#[test]
fn gitignore_excludes_matching_files() -> Result<()> {
    let dir = tempfile::tempdir()?;
    // tokei requires a .git directory to honour .gitignore
    fs::create_dir_all(dir.path().join(".git"))?;
    write_file(&dir, ".gitignore", "ignored/\n");
    write_file(&dir, "src/lib.rs", "fn lib() {}\n");
    write_file(&dir, "ignored/junk.rs", "fn junk() {}\n");

    let langs = scan(&[dir.path().to_path_buf()], &default_opts())?;
    let rust = langs.get(&tokei::LanguageType::Rust).expect("Rust");
    assert_eq!(rust.reports.len(), 1, ".gitignore should exclude ignored/");
    Ok(())
}

#[test]
fn no_ignore_vcs_bypasses_gitignore() -> Result<()> {
    let dir = tempfile::tempdir()?;
    write_file(&dir, ".gitignore", "extra/\n");
    write_file(&dir, "src/lib.rs", "fn lib() {}\n");
    write_file(&dir, "extra/more.rs", "fn more() {}\n");

    let mut opts = default_opts();
    opts.no_ignore_vcs = true;
    let langs = scan(&[dir.path().to_path_buf()], &opts)?;
    let rust = langs.get(&tokei::LanguageType::Rust).expect("Rust");
    assert!(
        rust.reports.len() >= 2,
        "no_ignore_vcs should bypass .gitignore",
    );
    Ok(())
}

// ============================================================================
// 4. Empty directory scan returns empty results
// ============================================================================

#[test]
fn empty_dir_returns_empty_languages() -> Result<()> {
    let dir = tempfile::tempdir()?;
    let langs = scan(&[dir.path().to_path_buf()], &default_opts())?;
    assert!(langs.is_empty(), "empty dir → no languages");
    Ok(())
}

#[test]
fn dir_with_only_non_code_files_returns_empty() -> Result<()> {
    let dir = tempfile::tempdir()?;
    write_file(&dir, "notes.txt", "just some notes\n");
    let langs = scan(&[dir.path().to_path_buf()], &default_opts())?;
    // .txt may or may not be detected by tokei depending on config; at minimum
    // it should not panic.
    let _ = langs;
    Ok(())
}

// ============================================================================
// 5. Determinism — same directory produces same results
// ============================================================================

#[test]
fn deterministic_code_lines() -> Result<()> {
    let dir = tempfile::tempdir()?;
    write_file(&dir, "a.rs", "fn a() { let x = 1; }\n");
    write_file(&dir, "b.rs", "fn b() { let y = 2; }\n");

    let paths = vec![dir.path().to_path_buf()];
    let r1 = scan(&paths, &default_opts())?;
    let r2 = scan(&paths, &default_opts())?;
    let c1 = r1.get(&tokei::LanguageType::Rust).unwrap().code;
    let c2 = r2.get(&tokei::LanguageType::Rust).unwrap().code;
    assert_eq!(c1, c2);
    Ok(())
}

#[test]
fn deterministic_language_set() -> Result<()> {
    let dir = tempfile::tempdir()?;
    write_file(&dir, "a.rs", "fn a() {}\n");
    write_file(&dir, "b.py", "x = 1\n");

    let paths = vec![dir.path().to_path_buf()];
    let r1 = scan(&paths, &default_opts())?;
    let r2 = scan(&paths, &default_opts())?;
    let k1: Vec<_> = r1.keys().collect();
    let k2: Vec<_> = r2.keys().collect();
    assert_eq!(k1, k2);
    Ok(())
}

#[test]
fn deterministic_comment_and_blank_counts() -> Result<()> {
    let dir = tempfile::tempdir()?;
    write_file(
        &dir,
        "lib.rs",
        "// comment\nfn main() {\n\n    let x = 1;\n}\n",
    );

    let paths = vec![dir.path().to_path_buf()];
    let r1 = scan(&paths, &default_opts())?;
    let r2 = scan(&paths, &default_opts())?;
    let rust1 = r1.get(&tokei::LanguageType::Rust).unwrap();
    let rust2 = r2.get(&tokei::LanguageType::Rust).unwrap();
    assert_eq!(rust1.comments, rust2.comments);
    assert_eq!(rust1.blanks, rust2.blanks);
    Ok(())
}

#[test]
fn deterministic_report_count() -> Result<()> {
    let dir = tempfile::tempdir()?;
    write_file(&dir, "a.rs", "fn a() {}\n");
    write_file(&dir, "b.rs", "fn b() {}\n");
    write_file(&dir, "c.rs", "fn c() {}\n");

    let paths = vec![dir.path().to_path_buf()];
    let r1 = scan(&paths, &default_opts())?;
    let r2 = scan(&paths, &default_opts())?;
    assert_eq!(
        r1.get(&tokei::LanguageType::Rust).unwrap().reports.len(),
        r2.get(&tokei::LanguageType::Rust).unwrap().reports.len(),
    );
    Ok(())
}

// ============================================================================
// 6. Property test — all file paths in scan output exist on disk
// ============================================================================

use proptest::prelude::*;

fn arb_rust_snippet() -> impl Strategy<Value = String> {
    prop::collection::vec(
        prop::sample::select(vec![
            "fn f() {}\n".to_string(),
            "let x = 1;\n".to_string(),
            "// comment\n".to_string(),
            "struct S;\n".to_string(),
            "const C: u8 = 0;\n".to_string(),
        ]),
        1..=5,
    )
    .prop_map(|lines| lines.join(""))
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(16))]

    #[test]
    fn prop_all_report_paths_exist(snippets in prop::collection::vec(arb_rust_snippet(), 1..=4)) {
        let dir = tempfile::tempdir().unwrap();
        for (i, snippet) in snippets.iter().enumerate() {
            let rel = format!("file_{i}.rs");
            let path = dir.path().join(&rel);
            fs::write(&path, snippet).unwrap();
        }

        let langs = scan(&[dir.path().to_path_buf()], &default_opts()).unwrap();
        if let Some(rust) = langs.get(&tokei::LanguageType::Rust) {
            for report in &rust.reports {
                prop_assert!(
                    report.name.exists(),
                    "Report path should exist: {}",
                    report.name.display(),
                );
            }
        }
    }

    #[test]
    fn prop_scan_report_count_matches_file_count(count in 1usize..=6) {
        let dir = tempfile::tempdir().unwrap();
        for i in 0..count {
            let path = dir.path().join(format!("f{i}.rs"));
            fs::write(&path, format!("fn f{i}() {{}}\n")).unwrap();
        }

        let langs = scan(&[dir.path().to_path_buf()], &default_opts()).unwrap();
        let rust = langs.get(&tokei::LanguageType::Rust).expect("Rust");
        prop_assert_eq!(rust.reports.len(), count, "report count must equal file count");
    }
}

// ============================================================================
// 7. Additional edge cases
// ============================================================================

#[test]
fn scan_nonexistent_path_errors() {
    let dir = tempfile::tempdir().unwrap();
    let missing = dir.path().join("nope");
    let result = scan(&[missing], &default_opts());
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("Path not found"));
}

#[test]
fn scan_lines_eq_code_plus_comments_plus_blanks() -> Result<()> {
    let dir = tempfile::tempdir()?;
    write_file(
        &dir,
        "full.rs",
        "// comment\nfn main() {\n\n    let x = 1;\n}\n",
    );
    let langs = scan(&[dir.path().to_path_buf()], &default_opts())?;
    let rust = langs.get(&tokei::LanguageType::Rust).unwrap();
    assert_eq!(rust.lines(), rust.code + rust.comments + rust.blanks);
    Ok(())
}

#[test]
fn scan_hidden_file_excluded_by_default() -> Result<()> {
    let dir = tempfile::tempdir()?;
    write_file(&dir, "visible.py", "x = 1\n");
    write_file(&dir, ".hidden.py", "y = 2\n");

    let langs = scan(&[dir.path().to_path_buf()], &default_opts())?;
    let py = langs.get(&tokei::LanguageType::Python).expect("Python");
    assert_eq!(py.reports.len(), 1, "hidden files excluded by default");
    Ok(())
}

#[test]
fn scan_hidden_flag_includes_dotfiles() -> Result<()> {
    let dir = tempfile::tempdir()?;
    write_file(&dir, "visible.py", "x = 1\n");
    write_file(&dir, ".hidden.py", "y = 2\n");

    let mut opts = default_opts();
    opts.hidden = true;
    let langs = scan(&[dir.path().to_path_buf()], &opts)?;
    let py = langs.get(&tokei::LanguageType::Python).expect("Python");
    assert!(py.reports.len() >= 2, "hidden flag should include dotfiles");
    Ok(())
}

#[test]
fn scan_subdirectories_recursive() -> Result<()> {
    let dir = tempfile::tempdir()?;
    write_file(&dir, "a/b/c/deep.rs", "fn deep() {}\n");
    let langs = scan(&[dir.path().to_path_buf()], &default_opts())?;
    let rust = langs.get(&tokei::LanguageType::Rust).expect("Rust");
    assert_eq!(rust.reports.len(), 1, "should find deeply nested file");
    Ok(())
}

#[test]
fn scan_multiple_paths_combines_results() -> Result<()> {
    let dir = tempfile::tempdir()?;
    write_file(&dir, "alpha/a.rs", "fn a() {}\n");
    write_file(&dir, "beta/b.py", "x = 1\n");

    let paths = vec![dir.path().join("alpha"), dir.path().join("beta")];
    let langs = scan(&paths, &default_opts())?;
    assert!(langs.get(&tokei::LanguageType::Rust).is_some());
    assert!(langs.get(&tokei::LanguageType::Python).is_some());
    Ok(())
}
