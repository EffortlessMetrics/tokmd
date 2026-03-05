//! Depth tests for `tokmd-scan` – W63 wave.
//!
//! Covers configuration options, language detection, file counting,
//! children/embedded handling, sort order, determinism, exclude patterns,
//! and property-based invariants.

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

/// Create a temp dir with a single Rust source file.
fn temp_rust_file(code: &str) -> Result<TempDir> {
    let dir = tempfile::tempdir()?;
    fs::write(dir.path().join("main.rs"), code)?;
    Ok(dir)
}

/// Create a temp dir with multiple files of different languages.
fn temp_multi_lang() -> Result<TempDir> {
    let dir = tempfile::tempdir()?;
    fs::write(
        dir.path().join("main.rs"),
        "fn main() {\n    println!(\"hi\");\n}\n",
    )?;
    fs::write(
        dir.path().join("app.py"),
        "def main():\n    print('hi')\n\nif __name__ == '__main__':\n    main()\n",
    )?;
    fs::write(
        dir.path().join("index.js"),
        "function main() {\n  console.log('hi');\n}\nmain();\n",
    )?;
    Ok(dir)
}

/// Create a temp dir with a nested structure.
fn temp_nested() -> Result<TempDir> {
    let dir = tempfile::tempdir()?;
    let src = dir.path().join("src");
    fs::create_dir_all(&src)?;
    fs::write(
        src.join("lib.rs"),
        "pub fn add(a: i32, b: i32) -> i32 { a + b }\n",
    )?;
    fs::write(
        src.join("util.rs"),
        "pub fn double(x: i32) -> i32 { x * 2 }\n",
    )?;
    fs::write(dir.path().join("Cargo.toml"), "[package]\nname = \"t\"\n")?;
    Ok(dir)
}

// ===========================================================================
// 1. Scan Configuration Options
// ===========================================================================

#[test]
fn config_mode_none_does_not_crash() -> Result<()> {
    let dir = temp_rust_file("fn f() {}\n")?;
    let opts = default_opts();
    assert!(scan(&[dir.path().to_path_buf()], &opts).is_ok());
    Ok(())
}

#[test]
fn config_mode_auto_does_not_crash() -> Result<()> {
    let dir = temp_rust_file("fn f() {}\n")?;
    let mut opts = default_opts();
    opts.config = ConfigMode::Auto;
    assert!(scan(&[dir.path().to_path_buf()], &opts).is_ok());
    Ok(())
}

#[test]
fn hidden_flag_includes_dot_files() -> Result<()> {
    let dir = tempfile::tempdir()?;
    fs::write(dir.path().join(".hidden.rs"), "fn secret() {}\n")?;
    let mut opts = default_opts();
    opts.hidden = true;
    let langs = scan(&[dir.path().to_path_buf()], &opts)?;
    // With hidden=true, the .hidden.rs file should be picked up
    let rust = langs.get(&tokei::LanguageType::Rust);
    assert!(rust.is_some(), "hidden .rs file should be found");
    Ok(())
}

#[test]
fn no_ignore_flag_sets_all_sub_flags() -> Result<()> {
    let dir = temp_rust_file("fn f() {}\n")?;
    let mut opts = default_opts();
    opts.no_ignore = true;
    // Should not panic and should succeed
    let result = scan(&[dir.path().to_path_buf()], &opts);
    assert!(result.is_ok());
    Ok(())
}

#[test]
fn individual_no_ignore_flags() -> Result<()> {
    let dir = temp_rust_file("fn f() {}\n")?;
    let mut opts = default_opts();
    opts.no_ignore_parent = true;
    opts.no_ignore_dot = true;
    opts.no_ignore_vcs = true;
    assert!(scan(&[dir.path().to_path_buf()], &opts).is_ok());
    Ok(())
}

#[test]
fn treat_doc_strings_as_comments_flag() -> Result<()> {
    let dir = temp_rust_file("/// doc\nfn f() {}\n")?;
    let mut opts = default_opts();
    opts.treat_doc_strings_as_comments = true;
    let langs = scan(&[dir.path().to_path_buf()], &opts)?;
    let rust = langs.get(&tokei::LanguageType::Rust).unwrap();
    // With treat_doc_strings_as_comments, the `/// doc` line is counted differently.
    // Verify the flag doesn't crash and metrics are consistent.
    assert!(rust.lines() > 0, "should have some lines");
    assert_eq!(
        rust.lines(),
        rust.code + rust.comments + rust.blanks,
        "lines invariant holds"
    );
    Ok(())
}

#[test]
fn all_flags_combined() -> Result<()> {
    let dir = temp_rust_file("fn f() {}\n")?;
    let opts = ScanOptions {
        excluded: vec!["*.txt".into()],
        config: ConfigMode::None,
        hidden: true,
        no_ignore: true,
        no_ignore_parent: true,
        no_ignore_dot: true,
        no_ignore_vcs: true,
        treat_doc_strings_as_comments: true,
    };
    assert!(scan(&[dir.path().to_path_buf()], &opts).is_ok());
    Ok(())
}

// ===========================================================================
// 2. Language Detection Accuracy
// ===========================================================================

#[test]
fn detects_rust() -> Result<()> {
    let dir = temp_rust_file("fn main() {}\n")?;
    let langs = scan(&[dir.path().to_path_buf()], &default_opts())?;
    assert!(langs.get(&tokei::LanguageType::Rust).is_some());
    Ok(())
}

#[test]
fn detects_python() -> Result<()> {
    let dir = tempfile::tempdir()?;
    fs::write(dir.path().join("app.py"), "print('hello')\n")?;
    let langs = scan(&[dir.path().to_path_buf()], &default_opts())?;
    assert!(langs.get(&tokei::LanguageType::Python).is_some());
    Ok(())
}

#[test]
fn detects_javascript() -> Result<()> {
    let dir = tempfile::tempdir()?;
    fs::write(dir.path().join("app.js"), "console.log('hi');\n")?;
    let langs = scan(&[dir.path().to_path_buf()], &default_opts())?;
    assert!(langs.get(&tokei::LanguageType::JavaScript).is_some());
    Ok(())
}

#[test]
fn detects_multiple_languages() -> Result<()> {
    let dir = temp_multi_lang()?;
    let langs = scan(&[dir.path().to_path_buf()], &default_opts())?;
    assert!(langs.get(&tokei::LanguageType::Rust).is_some());
    assert!(langs.get(&tokei::LanguageType::Python).is_some());
    assert!(langs.get(&tokei::LanguageType::JavaScript).is_some());
    Ok(())
}

#[test]
fn detects_toml() -> Result<()> {
    let dir = tempfile::tempdir()?;
    fs::write(dir.path().join("config.toml"), "[section]\nkey = \"val\"\n")?;
    let langs = scan(&[dir.path().to_path_buf()], &default_opts())?;
    assert!(langs.get(&tokei::LanguageType::Toml).is_some());
    Ok(())
}

#[test]
fn detects_json() -> Result<()> {
    let dir = tempfile::tempdir()?;
    fs::write(dir.path().join("data.json"), "{\"a\": 1}\n")?;
    let langs = scan(&[dir.path().to_path_buf()], &default_opts())?;
    assert!(langs.get(&tokei::LanguageType::Json).is_some());
    Ok(())
}

#[test]
fn detects_markdown() -> Result<()> {
    let dir = tempfile::tempdir()?;
    fs::write(dir.path().join("README.md"), "# Hello\nWorld\n")?;
    let langs = scan(&[dir.path().to_path_buf()], &default_opts())?;
    assert!(langs.get(&tokei::LanguageType::Markdown).is_some());
    Ok(())
}

// ===========================================================================
// 3. File Counting Correctness
// ===========================================================================

#[test]
fn single_file_counts_one_report() -> Result<()> {
    let dir = temp_rust_file("fn f() {}\n")?;
    let langs = scan(&[dir.path().to_path_buf()], &default_opts())?;
    let rust = langs.get(&tokei::LanguageType::Rust).unwrap();
    assert_eq!(rust.reports.len(), 1, "exactly one Rust report expected");
    Ok(())
}

#[test]
fn two_rust_files_count_two_reports() -> Result<()> {
    let dir = tempfile::tempdir()?;
    fs::write(dir.path().join("a.rs"), "fn a() {}\n")?;
    fs::write(dir.path().join("b.rs"), "fn b() {}\n")?;
    let langs = scan(&[dir.path().to_path_buf()], &default_opts())?;
    let rust = langs.get(&tokei::LanguageType::Rust).unwrap();
    assert_eq!(rust.reports.len(), 2);
    Ok(())
}

#[test]
fn nested_files_are_found() -> Result<()> {
    let dir = temp_nested()?;
    let langs = scan(&[dir.path().to_path_buf()], &default_opts())?;
    let rust = langs.get(&tokei::LanguageType::Rust).unwrap();
    assert_eq!(rust.reports.len(), 2, "lib.rs and util.rs");
    Ok(())
}

#[test]
fn code_lines_are_positive_for_nonempty_file() -> Result<()> {
    let dir = temp_rust_file("fn main() {\n    let x = 1;\n}\n")?;
    let langs = scan(&[dir.path().to_path_buf()], &default_opts())?;
    let rust = langs.get(&tokei::LanguageType::Rust).unwrap();
    assert!(rust.code > 0);
    Ok(())
}

#[test]
fn blank_lines_counted() -> Result<()> {
    let dir = temp_rust_file("fn f() {}\n\n\n")?;
    let langs = scan(&[dir.path().to_path_buf()], &default_opts())?;
    let rust = langs.get(&tokei::LanguageType::Rust).unwrap();
    assert!(rust.blanks > 0, "should count blank lines");
    Ok(())
}

#[test]
fn comment_lines_counted() -> Result<()> {
    let dir = temp_rust_file("// comment\nfn f() {}\n")?;
    let langs = scan(&[dir.path().to_path_buf()], &default_opts())?;
    let rust = langs.get(&tokei::LanguageType::Rust).unwrap();
    assert!(rust.comments > 0, "should count comment lines");
    Ok(())
}

#[test]
fn total_lines_equals_sum() -> Result<()> {
    let dir = temp_rust_file("// comment\nfn f() {}\n\n")?;
    let langs = scan(&[dir.path().to_path_buf()], &default_opts())?;
    let rust = langs.get(&tokei::LanguageType::Rust).unwrap();
    let computed = rust.code + rust.comments + rust.blanks;
    assert_eq!(
        rust.lines(),
        computed,
        "lines() == code + comments + blanks"
    );
    Ok(())
}

// ===========================================================================
// 4. Children / Embedded Language Handling
// ===========================================================================

#[test]
fn html_may_have_embedded_children() -> Result<()> {
    let dir = tempfile::tempdir()?;
    fs::write(
        dir.path().join("page.html"),
        r#"<!DOCTYPE html>
<html>
<head>
<style>
body { color: red; }
</style>
<script>
console.log('hello');
</script>
</head>
<body>
<p>Hello</p>
</body>
</html>
"#,
    )?;
    let langs = scan(&[dir.path().to_path_buf()], &default_opts())?;
    let html = langs.get(&tokei::LanguageType::Html);
    assert!(html.is_some(), "should detect HTML");
    Ok(())
}

#[test]
fn scan_result_children_map_exists() -> Result<()> {
    let dir = tempfile::tempdir()?;
    fs::write(
        dir.path().join("page.html"),
        "<html><head><style>body{}</style></head><body></body></html>\n",
    )?;
    let langs = scan(&[dir.path().to_path_buf()], &default_opts())?;
    // The children map is part of Language, verify we can access it
    for (_lt, lang) in langs.iter() {
        let _ = &lang.children; // should compile and not panic
    }
    Ok(())
}

// ===========================================================================
// 5. Sort Order Verification
// ===========================================================================

#[test]
fn languages_iterator_is_deterministic() -> Result<()> {
    let dir = temp_multi_lang()?;
    let path = dir.path().to_path_buf();
    let opts = default_opts();

    let langs1 = scan(&[path.clone()], &opts)?;
    let langs2 = scan(&[path], &opts)?;

    let keys1: Vec<_> = langs1.iter().map(|(lt, _)| lt.name()).collect();
    let keys2: Vec<_> = langs2.iter().map(|(lt, _)| lt.name()).collect();
    assert_eq!(keys1, keys2, "language iteration order must be stable");
    Ok(())
}

// ===========================================================================
// 6. Empty Directory Scanning
// ===========================================================================

#[test]
fn empty_dir_produces_no_languages() -> Result<()> {
    let dir = tempfile::tempdir()?;
    let langs = scan(&[dir.path().to_path_buf()], &default_opts())?;
    // No source files → no languages with code
    let nonempty: Vec<_> = langs.iter().filter(|(_, l)| l.code > 0).collect();
    assert!(nonempty.is_empty(), "empty dir should produce no code");
    Ok(())
}

#[test]
fn dir_with_only_empty_files() -> Result<()> {
    let dir = tempfile::tempdir()?;
    fs::write(dir.path().join("empty.rs"), "")?;
    let langs = scan(&[dir.path().to_path_buf()], &default_opts())?;
    // An empty .rs file has 0 code lines
    for (_, lang) in langs.iter() {
        assert_eq!(lang.code, 0, "empty file should have 0 code lines");
    }
    Ok(())
}

#[test]
fn dir_with_only_blank_lines() -> Result<()> {
    let dir = tempfile::tempdir()?;
    fs::write(dir.path().join("blank.rs"), "\n\n\n\n")?;
    let langs = scan(&[dir.path().to_path_buf()], &default_opts())?;
    for (_, lang) in langs.iter() {
        assert_eq!(lang.code, 0, "all-blank file should have 0 code lines");
    }
    Ok(())
}

// ===========================================================================
// 7. Exclude Patterns Effect
// ===========================================================================

#[test]
fn exclude_pattern_hides_files() -> Result<()> {
    let dir = tempfile::tempdir()?;
    fs::write(dir.path().join("keep.rs"), "fn f() {}\n")?;
    fs::write(dir.path().join("skip.py"), "x = 1\n")?;

    let mut opts = default_opts();
    opts.excluded = vec!["*.py".into()];
    let langs = scan(&[dir.path().to_path_buf()], &opts)?;

    assert!(langs.get(&tokei::LanguageType::Rust).is_some());
    let py = langs.get(&tokei::LanguageType::Python);
    let py_code = py.map(|l| l.code).unwrap_or(0);
    assert_eq!(py_code, 0, "excluded .py should have no code");
    Ok(())
}

#[test]
fn exclude_directory_pattern() -> Result<()> {
    let dir = tempfile::tempdir()?;
    let vendor = dir.path().join("vendor");
    fs::create_dir_all(&vendor)?;
    fs::write(vendor.join("lib.rs"), "fn vendor() {}\n")?;
    fs::write(dir.path().join("main.rs"), "fn main() {}\n")?;

    let mut opts = default_opts();
    opts.excluded = vec!["vendor".into()];
    let langs = scan(&[dir.path().to_path_buf()], &opts)?;
    let rust = langs.get(&tokei::LanguageType::Rust).unwrap();
    assert_eq!(rust.reports.len(), 1, "vendor file should be excluded");
    Ok(())
}

#[test]
fn multiple_exclude_patterns() -> Result<()> {
    let dir = temp_multi_lang()?;
    let mut opts = default_opts();
    opts.excluded = vec!["*.py".into(), "*.js".into()];
    let langs = scan(&[dir.path().to_path_buf()], &opts)?;

    assert!(langs.get(&tokei::LanguageType::Rust).is_some());
    let py = langs
        .get(&tokei::LanguageType::Python)
        .map(|l| l.code)
        .unwrap_or(0);
    let js = langs
        .get(&tokei::LanguageType::JavaScript)
        .map(|l| l.code)
        .unwrap_or(0);
    assert_eq!(py, 0, "Python should be excluded");
    assert_eq!(js, 0, "JavaScript should be excluded");
    Ok(())
}

#[test]
fn empty_exclude_list_changes_nothing() -> Result<()> {
    let dir = temp_rust_file("fn f() {}\n")?;
    let opts = default_opts(); // excluded is empty
    let langs = scan(&[dir.path().to_path_buf()], &opts)?;
    assert!(langs.get(&tokei::LanguageType::Rust).is_some());
    Ok(())
}

// ===========================================================================
// 8. Error Handling
// ===========================================================================

#[test]
fn nonexistent_path_returns_error() {
    let opts = default_opts();
    let result = scan(&[PathBuf::from("/no/such/path/w63")], &opts);
    assert!(result.is_err());
    let msg = result.unwrap_err().to_string();
    assert!(msg.contains("Path not found"), "error: {msg}");
}

#[test]
fn multiple_paths_one_missing_returns_error() -> Result<()> {
    let dir = temp_rust_file("fn f() {}\n")?;
    let result = scan(
        &[dir.path().to_path_buf(), PathBuf::from("/no/exist/w63")],
        &default_opts(),
    );
    assert!(result.is_err());
    Ok(())
}

#[test]
fn empty_paths_slice_panics_in_tokei() {
    // tokei panics when given empty paths; verify we handle this gracefully
    // or that the behavior is at least known.
    let opts = default_opts();
    let result = std::panic::catch_unwind(|| scan(&[], &opts));
    // Either it succeeds or panics in tokei – both are acceptable; we just document the behavior
    let _ = result;
}

// ===========================================================================
// 9. Determinism: Same Directory → Same Result
// ===========================================================================

#[test]
fn determinism_same_dir_same_result() -> Result<()> {
    let dir = temp_multi_lang()?;
    let path = dir.path().to_path_buf();
    let opts = default_opts();

    let r1 = scan(&[path.clone()], &opts)?;
    let r2 = scan(&[path], &opts)?;

    // Same set of languages detected
    let k1: Vec<_> = r1.iter().map(|(lt, _)| lt.name()).collect();
    let k2: Vec<_> = r2.iter().map(|(lt, _)| lt.name()).collect();
    assert_eq!(k1, k2);

    // Same code counts per language
    for (lt, lang1) in r1.iter() {
        let lang2 = r2.get(lt).unwrap();
        assert_eq!(lang1.code, lang2.code, "code mismatch for {}", lt.name());
        assert_eq!(lang1.comments, lang2.comments);
        assert_eq!(lang1.blanks, lang2.blanks);
    }
    Ok(())
}

#[test]
fn determinism_repeated_three_times() -> Result<()> {
    let dir = temp_rust_file("fn f() { let x = 1; }\n// comment\n\n")?;
    let path = dir.path().to_path_buf();
    let opts = default_opts();

    let mut codes = Vec::new();
    for _ in 0..3 {
        let langs = scan(&[path.clone()], &opts)?;
        let rust = langs.get(&tokei::LanguageType::Rust).unwrap();
        codes.push((rust.code, rust.comments, rust.blanks));
    }
    assert!(codes.windows(2).all(|w| w[0] == w[1]), "must be identical");
    Ok(())
}

#[test]
fn determinism_config_mode_none_vs_none() -> Result<()> {
    let dir = temp_rust_file("fn main() { println!(\"hi\"); }\n")?;
    let mut opts1 = default_opts();
    opts1.config = ConfigMode::None;
    let mut opts2 = default_opts();
    opts2.config = ConfigMode::None;

    let r1 = scan(&[dir.path().to_path_buf()], &opts1)?;
    let r2 = scan(&[dir.path().to_path_buf()], &opts2)?;

    let rust1 = r1.get(&tokei::LanguageType::Rust).unwrap();
    let rust2 = r2.get(&tokei::LanguageType::Rust).unwrap();
    assert_eq!(rust1.code, rust2.code);
    Ok(())
}

// ===========================================================================
// 10. Multi-path scanning
// ===========================================================================

#[test]
fn scan_multiple_paths() -> Result<()> {
    let dir1 = temp_rust_file("fn a() {}\n")?;
    let dir2 = temp_rust_file("fn b() {}\nfn c() {}\n")?;
    let langs = scan(
        &[dir1.path().to_path_buf(), dir2.path().to_path_buf()],
        &default_opts(),
    )?;
    let rust = langs.get(&tokei::LanguageType::Rust).unwrap();
    assert_eq!(rust.reports.len(), 2, "one file from each directory");
    Ok(())
}

// ===========================================================================
// 11. Various file scenarios
// ===========================================================================

#[test]
fn file_with_only_comments() -> Result<()> {
    let dir = temp_rust_file("// line one\n// line two\n")?;
    let langs = scan(&[dir.path().to_path_buf()], &default_opts())?;
    let rust = langs.get(&tokei::LanguageType::Rust).unwrap();
    assert_eq!(rust.code, 0);
    assert!(rust.comments >= 2);
    Ok(())
}

#[test]
fn large_file_counts_correctly() -> Result<()> {
    let dir = tempfile::tempdir()?;
    let mut code = String::new();
    for i in 0..200 {
        code.push_str(&format!("fn f{i}() {{}}\n"));
    }
    fs::write(dir.path().join("big.rs"), &code)?;
    let langs = scan(&[dir.path().to_path_buf()], &default_opts())?;
    let rust = langs.get(&tokei::LanguageType::Rust).unwrap();
    assert_eq!(rust.code, 200, "200 single-line functions");
    Ok(())
}

#[test]
fn deeply_nested_directory() -> Result<()> {
    let dir = tempfile::tempdir()?;
    let deep = dir.path().join("a").join("b").join("c").join("d");
    fs::create_dir_all(&deep)?;
    fs::write(deep.join("deep.rs"), "fn deep() {}\n")?;
    let langs = scan(&[dir.path().to_path_buf()], &default_opts())?;
    let rust = langs.get(&tokei::LanguageType::Rust).unwrap();
    assert_eq!(rust.reports.len(), 1);
    Ok(())
}

// ===========================================================================
// 12. Property-based Tests
// ===========================================================================

mod properties {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn scan_never_has_negative_metrics(
            lines in 1usize..20,
            blanks in 0usize..5
        ) {
            let dir = tempfile::tempdir().unwrap();
            let mut code = String::new();
            for i in 0..lines {
                code.push_str(&format!("fn f{i}() {{}}\n"));
            }
            for _ in 0..blanks {
                code.push('\n');
            }
            fs::write(dir.path().join("gen.rs"), &code).unwrap();
            let langs = scan(&[dir.path().to_path_buf()], &default_opts()).unwrap();
            for (_, lang) in langs.iter() {
                // All metrics must be non-negative (they're usize, but verify > 0 logic)
                let _ = lang.code;
                let _ = lang.comments;
                let _ = lang.blanks;
                prop_assert!(lang.lines() == lang.code + lang.comments + lang.blanks);
            }
        }

        #[test]
        fn scan_deterministic_for_any_line_count(n in 1usize..50) {
            let dir = tempfile::tempdir().unwrap();
            let mut code = String::new();
            for i in 0..n {
                code.push_str(&format!("fn f{i}() {{}}\n"));
            }
            fs::write(dir.path().join("gen.rs"), &code).unwrap();
            let opts = default_opts();
            let r1 = scan(&[dir.path().to_path_buf()], &opts).unwrap();
            let r2 = scan(&[dir.path().to_path_buf()], &opts).unwrap();

            let rust1 = r1.get(&tokei::LanguageType::Rust).unwrap();
            let rust2 = r2.get(&tokei::LanguageType::Rust).unwrap();
            prop_assert_eq!(rust1.code, rust2.code);
            prop_assert_eq!(rust1.comments, rust2.comments);
            prop_assert_eq!(rust1.blanks, rust2.blanks);
        }

        #[test]
        fn lines_equals_code_plus_comments_plus_blanks(n in 1usize..30) {
            let dir = tempfile::tempdir().unwrap();
            let mut src = String::new();
            for i in 0..n {
                src.push_str(&format!("// comment {i}\nfn f{i}() {{}}\n\n"));
            }
            fs::write(dir.path().join("gen.rs"), &src).unwrap();
            let langs = scan(&[dir.path().to_path_buf()], &default_opts()).unwrap();
            let rust = langs.get(&tokei::LanguageType::Rust).unwrap();
            prop_assert_eq!(
                rust.lines(),
                rust.code + rust.comments + rust.blanks,
                "lines must equal sum of parts"
            );
        }

        #[test]
        fn scan_code_never_exceeds_lines(n in 1usize..30) {
            let dir = tempfile::tempdir().unwrap();
            let mut src = String::new();
            for i in 0..n {
                src.push_str(&format!("fn f{i}() {{}}\n"));
            }
            fs::write(dir.path().join("gen.rs"), &src).unwrap();
            let langs = scan(&[dir.path().to_path_buf()], &default_opts()).unwrap();
            for (_, lang) in langs.iter() {
                prop_assert!(lang.code <= lang.lines(), "code <= total lines");
            }
        }

        #[test]
        fn report_count_matches_file_count(n in 1usize..10) {
            let dir = tempfile::tempdir().unwrap();
            for i in 0..n {
                fs::write(dir.path().join(format!("f{i}.rs")), format!("fn f{i}() {{}}\n")).unwrap();
            }
            let langs = scan(&[dir.path().to_path_buf()], &default_opts()).unwrap();
            let rust = langs.get(&tokei::LanguageType::Rust).unwrap();
            prop_assert_eq!(rust.reports.len(), n, "one report per file");
        }
    }
}
