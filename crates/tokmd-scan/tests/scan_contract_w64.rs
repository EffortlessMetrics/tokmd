//! Contract tests for `tokmd-scan` crate — W64 batch.
//!
//! Covers scan execution, language detection, line counting accuracy,
//! config flag mapping, embedded-language handling, and edge cases.

use std::fs;
use std::path::PathBuf;

use anyhow::Result;
use tempfile::TempDir;
use tokmd_scan::scan;
use tokmd_settings::ScanOptions;
use tokmd_types::ConfigMode;

// ===========================================================================
// Helpers
// ===========================================================================

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
    fs::write(&path, content).expect("write file");
}

// ===========================================================================
// 1  Basic scan execution
// ===========================================================================

#[test]
fn scan_own_source_detects_rust() -> Result<()> {
    let langs = scan(&[crate_src()], &default_opts())?;
    assert!(langs.get(&tokei::LanguageType::Rust).is_some());
    Ok(())
}

#[test]
fn scan_own_source_has_positive_code_lines() -> Result<()> {
    let langs = scan(&[crate_src()], &default_opts())?;
    let rust = langs.get(&tokei::LanguageType::Rust).unwrap();
    assert!(rust.code > 0);
    assert!(rust.lines() > 0);
    Ok(())
}

#[test]
fn scan_empty_directory_returns_empty() -> Result<()> {
    let dir = TempDir::new()?;
    let langs = scan(&[dir.path().to_path_buf()], &default_opts())?;
    assert!(langs.is_empty());
    Ok(())
}

#[test]
fn scan_nonexistent_path_returns_error() {
    let dir = TempDir::new().unwrap();
    let bad = dir.path().join("nonexistent");
    let result = scan(&[bad], &default_opts());
    assert!(result.is_err());
    let msg = result.unwrap_err().to_string();
    assert!(msg.contains("Path not found"));
}

// ===========================================================================
// 2  Language detection
// ===========================================================================

#[test]
fn scan_detects_python() -> Result<()> {
    let dir = TempDir::new()?;
    write_file(&dir, "main.py", "def hello():\n    print('hi')\n");
    let langs = scan(&[dir.path().to_path_buf()], &default_opts())?;
    assert!(langs.get(&tokei::LanguageType::Python).is_some());
    Ok(())
}

#[test]
fn scan_detects_javascript() -> Result<()> {
    let dir = TempDir::new()?;
    write_file(&dir, "app.js", "function main() {\n  return 42;\n}\n");
    let langs = scan(&[dir.path().to_path_buf()], &default_opts())?;
    assert!(langs.get(&tokei::LanguageType::JavaScript).is_some());
    Ok(())
}

#[test]
fn scan_detects_c() -> Result<()> {
    let dir = TempDir::new()?;
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
fn scan_detects_go() -> Result<()> {
    let dir = TempDir::new()?;
    write_file(&dir, "main.go", "package main\n\nfunc main() {}\n");
    let langs = scan(&[dir.path().to_path_buf()], &default_opts())?;
    assert!(langs.get(&tokei::LanguageType::Go).is_some());
    Ok(())
}

#[test]
fn scan_detects_toml() -> Result<()> {
    let dir = TempDir::new()?;
    write_file(&dir, "config.toml", "[package]\nname = \"test\"\n");
    let langs = scan(&[dir.path().to_path_buf()], &default_opts())?;
    assert!(langs.get(&tokei::LanguageType::Toml).is_some());
    Ok(())
}

#[test]
fn scan_detects_json() -> Result<()> {
    let dir = TempDir::new()?;
    write_file(&dir, "data.json", "{\"key\": \"value\"}\n");
    let langs = scan(&[dir.path().to_path_buf()], &default_opts())?;
    assert!(langs.get(&tokei::LanguageType::Json).is_some());
    Ok(())
}

#[test]
fn scan_detects_multiple_languages() -> Result<()> {
    let dir = TempDir::new()?;
    write_file(&dir, "main.rs", "fn main() {}\n");
    write_file(&dir, "app.py", "print('hi')\n");
    write_file(&dir, "script.js", "console.log('x');\n");
    let langs = scan(&[dir.path().to_path_buf()], &default_opts())?;
    assert!(langs.get(&tokei::LanguageType::Rust).is_some());
    assert!(langs.get(&tokei::LanguageType::Python).is_some());
    assert!(langs.get(&tokei::LanguageType::JavaScript).is_some());
    Ok(())
}

// ===========================================================================
// 3  Line counting accuracy
// ===========================================================================

#[test]
fn scan_counts_code_lines_correctly() -> Result<()> {
    let dir = TempDir::new()?;
    // 3 code lines, 1 comment, 1 blank
    write_file(
        &dir,
        "exact.py",
        "# comment\n\ndef hello():\n    pass\n    return\n",
    );
    let langs = scan(&[dir.path().to_path_buf()], &default_opts())?;
    let py = langs.get(&tokei::LanguageType::Python).unwrap();
    assert_eq!(py.code, 3, "3 code lines");
    assert_eq!(py.comments, 1, "1 comment line");
    assert_eq!(py.blanks, 1, "1 blank line");
    Ok(())
}

#[test]
fn scan_line_sum_property() -> Result<()> {
    // Property: code + blanks + comments = lines()
    let langs = scan(&[crate_src()], &default_opts())?;
    let rust = langs.get(&tokei::LanguageType::Rust).unwrap();
    assert_eq!(
        rust.code + rust.blanks + rust.comments,
        rust.lines(),
        "code + blanks + comments must equal total lines"
    );
    Ok(())
}

#[test]
fn scan_single_code_line() -> Result<()> {
    let dir = TempDir::new()?;
    write_file(&dir, "one.py", "x = 1\n");
    let langs = scan(&[dir.path().to_path_buf()], &default_opts())?;
    let py = langs.get(&tokei::LanguageType::Python).unwrap();
    assert_eq!(py.code, 1);
    assert_eq!(py.blanks, 0);
    assert_eq!(py.comments, 0);
    Ok(())
}

#[test]
fn scan_only_comments() -> Result<()> {
    let dir = TempDir::new()?;
    write_file(
        &dir,
        "comments.py",
        "# line one\n# line two\n# line three\n",
    );
    let langs = scan(&[dir.path().to_path_buf()], &default_opts())?;
    let py = langs.get(&tokei::LanguageType::Python).unwrap();
    assert_eq!(py.comments, 3);
    assert_eq!(py.code, 0);
    Ok(())
}

#[test]
fn scan_only_blanks_not_detected() -> Result<()> {
    let dir = TempDir::new()?;
    // A file with only blank lines — tokei may or may not count it
    write_file(&dir, "blank.py", "\n\n\n");
    let langs = scan(&[dir.path().to_path_buf()], &default_opts())?;
    // Either it's detected with 0 code or not detected at all
    if let Some(py) = langs.get(&tokei::LanguageType::Python) {
        assert_eq!(py.code, 0, "only blanks means 0 code");
    }
    Ok(())
}

// ===========================================================================
// 4  Config flag behaviour
// ===========================================================================

#[test]
fn scan_with_config_mode_auto() -> Result<()> {
    let langs = scan(
        &[crate_src()],
        &ScanOptions {
            config: ConfigMode::Auto,
            ..default_opts()
        },
    )?;
    assert!(!langs.is_empty());
    Ok(())
}

#[test]
fn scan_with_config_mode_none() -> Result<()> {
    let langs = scan(
        &[crate_src()],
        &ScanOptions {
            config: ConfigMode::None,
            ..default_opts()
        },
    )?;
    assert!(!langs.is_empty());
    Ok(())
}

#[test]
fn scan_with_hidden_flag() -> Result<()> {
    let langs = scan(
        &[crate_src()],
        &ScanOptions {
            hidden: true,
            ..default_opts()
        },
    )?;
    assert!(!langs.is_empty());
    Ok(())
}

#[test]
fn scan_with_no_ignore_flag() -> Result<()> {
    let langs = scan(
        &[crate_src()],
        &ScanOptions {
            no_ignore: true,
            ..default_opts()
        },
    )?;
    assert!(!langs.is_empty());
    Ok(())
}

#[test]
fn scan_with_all_no_ignore_sub_flags() -> Result<()> {
    let langs = scan(
        &[crate_src()],
        &ScanOptions {
            no_ignore_parent: true,
            no_ignore_dot: true,
            no_ignore_vcs: true,
            ..default_opts()
        },
    )?;
    assert!(!langs.is_empty());
    Ok(())
}

#[test]
fn scan_with_doc_strings_as_comments() -> Result<()> {
    let dir = TempDir::new()?;
    write_file(
        &dir,
        "docstrings.py",
        "\"\"\"Module docstring.\"\"\"\n\ndef foo():\n    \"\"\"Func docstring.\"\"\"\n    pass\n",
    );
    let normal = scan(&[dir.path().to_path_buf()], &default_opts())?;
    let with_docs = scan(
        &[dir.path().to_path_buf()],
        &ScanOptions {
            treat_doc_strings_as_comments: true,
            ..default_opts()
        },
    )?;
    // With treat_doc_strings_as_comments, comment count should be >= normal
    let normal_py = normal.get(&tokei::LanguageType::Python).unwrap();
    let docs_py = with_docs.get(&tokei::LanguageType::Python).unwrap();
    assert!(
        docs_py.comments >= normal_py.comments,
        "doc-string-as-comment mode should have >= comments"
    );
    Ok(())
}

#[test]
fn scan_with_excluded_patterns() -> Result<()> {
    let dir = TempDir::new()?;
    write_file(&dir, "keep.rs", "fn main() {}\n");
    write_file(&dir, "skip.py", "print('hi')\n");
    let langs = scan(
        &[dir.path().to_path_buf()],
        &ScanOptions {
            excluded: vec!["*.py".to_string()],
            ..default_opts()
        },
    )?;
    assert!(langs.get(&tokei::LanguageType::Rust).is_some());
    assert!(
        langs.get(&tokei::LanguageType::Python).is_none(),
        "*.py should be excluded"
    );
    Ok(())
}

#[test]
fn scan_with_all_flags_combined() -> Result<()> {
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
    let langs = scan(&[crate_src()], &opts)?;
    assert!(!langs.is_empty());
    Ok(())
}

// ===========================================================================
// 5  Embedded/children language handling
// ===========================================================================

#[test]
fn scan_html_with_embedded_js() -> Result<()> {
    let dir = TempDir::new()?;
    write_file(
        &dir,
        "page.html",
        r#"<!DOCTYPE html>
<html>
<head>
<script>
function hello() {
    console.log("hi");
}
</script>
</head>
<body><p>Hello</p></body>
</html>
"#,
    );
    let langs = scan(&[dir.path().to_path_buf()], &default_opts())?;
    assert!(
        langs.get(&tokei::LanguageType::Html).is_some(),
        "HTML should be detected"
    );
    Ok(())
}

#[test]
fn scan_html_with_embedded_css() -> Result<()> {
    let dir = TempDir::new()?;
    write_file(
        &dir,
        "styled.html",
        r#"<html>
<head>
<style>
body { color: red; }
h1 { font-size: 2em; }
</style>
</head>
<body></body>
</html>
"#,
    );
    let langs = scan(&[dir.path().to_path_buf()], &default_opts())?;
    assert!(langs.get(&tokei::LanguageType::Html).is_some());
    Ok(())
}

// ===========================================================================
// 6  Property: line counts sum correctly
// ===========================================================================

#[test]
fn property_all_languages_lines_sum() -> Result<()> {
    let dir = TempDir::new()?;
    write_file(
        &dir,
        "a.rs",
        "fn main() {\n    // hi\n\n    let x = 1;\n}\n",
    );
    write_file(&dir, "b.py", "# comment\nx = 1\n\ny = 2\n");
    write_file(&dir, "c.js", "// comment\nconst x = 1;\n\nconst y = 2;\n");
    let langs = scan(&[dir.path().to_path_buf()], &default_opts())?;

    for (lang_type, lang) in langs.iter() {
        let sum = lang.code + lang.blanks + lang.comments;
        assert_eq!(
            sum,
            lang.lines(),
            "line sum mismatch for {lang_type:?}: {sum} != {}",
            lang.lines()
        );
    }
    Ok(())
}

#[test]
fn property_code_lines_are_usize() -> Result<()> {
    let langs = scan(&[crate_src()], &default_opts())?;
    for (_, lang) in langs.iter() {
        // usize fields are inherently non-negative; verify they sum correctly
        let sum = lang.code + lang.blanks + lang.comments;
        assert_eq!(sum, lang.lines(), "line sum should hold");
    }
    Ok(())
}

// ===========================================================================
// 7  BDD scenarios
// ===========================================================================

#[test]
fn given_rust_file_when_scanned_then_language_and_counts_correct() -> Result<()> {
    // Given: a temp dir with one Rust file
    let dir = TempDir::new()?;
    write_file(
        &dir,
        "lib.rs",
        "// docs\nfn add(a: i32, b: i32) -> i32 {\n    a + b\n}\n",
    );

    // When: scanning
    let langs = scan(&[dir.path().to_path_buf()], &default_opts())?;

    // Then: Rust detected with correct counts
    let rust = langs
        .get(&tokei::LanguageType::Rust)
        .expect("Rust should be detected");
    assert_eq!(rust.code, 3, "3 code lines");
    assert_eq!(rust.comments, 1, "1 comment line");
    assert_eq!(rust.blanks, 0, "0 blank lines");
    Ok(())
}

#[test]
fn given_python_file_when_scanned_then_comments_counted() -> Result<()> {
    // Given
    let dir = TempDir::new()?;
    write_file(&dir, "main.py", "# comment 1\n# comment 2\nprint('hi')\n");

    // When
    let langs = scan(&[dir.path().to_path_buf()], &default_opts())?;

    // Then
    let py = langs.get(&tokei::LanguageType::Python).unwrap();
    assert_eq!(py.comments, 2);
    assert_eq!(py.code, 1);
    Ok(())
}

#[test]
fn given_excluded_pattern_when_scanned_then_language_missing() -> Result<()> {
    // Given
    let dir = TempDir::new()?;
    write_file(&dir, "keep.rs", "fn f() {}\n");
    write_file(&dir, "drop.py", "x = 1\n");

    // When: excluding *.py
    let langs = scan(
        &[dir.path().to_path_buf()],
        &ScanOptions {
            excluded: vec!["*.py".to_string()],
            ..default_opts()
        },
    )?;

    // Then: Python not in results
    assert!(langs.get(&tokei::LanguageType::Rust).is_some());
    assert!(langs.get(&tokei::LanguageType::Python).is_none());
    Ok(())
}

#[test]
fn given_empty_dir_when_scanned_then_no_languages() -> Result<()> {
    let dir = TempDir::new()?;
    let langs = scan(&[dir.path().to_path_buf()], &default_opts())?;
    assert!(langs.is_empty());
    Ok(())
}

// ===========================================================================
// 8  Edge cases
// ===========================================================================

#[test]
fn scan_empty_file() -> Result<()> {
    let dir = TempDir::new()?;
    write_file(&dir, "empty.rs", "");
    let langs = scan(&[dir.path().to_path_buf()], &default_opts())?;
    // Empty file may or may not be detected
    if let Some(rust) = langs.get(&tokei::LanguageType::Rust) {
        assert_eq!(rust.code, 0);
    }
    Ok(())
}

#[test]
fn scan_file_with_no_extension() -> Result<()> {
    let dir = TempDir::new()?;
    write_file(&dir, "Makefile", "all:\n\techo hello\n");
    let langs = scan(&[dir.path().to_path_buf()], &default_opts())?;
    assert!(langs.get(&tokei::LanguageType::Makefile).is_some());
    Ok(())
}

#[test]
fn scan_file_with_sh_extension() -> Result<()> {
    let dir = TempDir::new()?;
    write_file(&dir, "script.sh", "#!/bin/bash\necho hello\n");
    let langs = scan(&[dir.path().to_path_buf()], &default_opts())?;
    // tokei may detect this as Bash or Sh depending on version
    let detected = !langs.is_empty();
    assert!(
        detected,
        "shell script with .sh extension should be detected"
    );
    Ok(())
}

#[test]
fn scan_very_large_file() -> Result<()> {
    let dir = TempDir::new()?;
    let mut content = String::with_capacity(100_000);
    for i in 0..1000 {
        content.push_str(&format!("x_{i} = {i}\n"));
    }
    write_file(&dir, "big.py", &content);
    let langs = scan(&[dir.path().to_path_buf()], &default_opts())?;
    let py = langs.get(&tokei::LanguageType::Python).unwrap();
    assert_eq!(py.code, 1000, "1000 code lines expected");
    Ok(())
}

#[test]
fn scan_single_line_file() -> Result<()> {
    let dir = TempDir::new()?;
    write_file(&dir, "one.py", "x = 42\n");
    let langs = scan(&[dir.path().to_path_buf()], &default_opts())?;
    let py = langs.get(&tokei::LanguageType::Python).unwrap();
    assert_eq!(py.code, 1);
    assert_eq!(py.lines(), 1);
    Ok(())
}

#[test]
fn scan_file_no_trailing_newline() -> Result<()> {
    let dir = TempDir::new()?;
    write_file(&dir, "no_nl.py", "x = 1");
    let langs = scan(&[dir.path().to_path_buf()], &default_opts())?;
    let py = langs.get(&tokei::LanguageType::Python).unwrap();
    assert!(py.code >= 1, "code line should still be counted");
    Ok(())
}

#[test]
fn scan_multiple_paths() -> Result<()> {
    let dir1 = TempDir::new()?;
    let dir2 = TempDir::new()?;
    write_file(&dir1, "a.rs", "fn a() {}\n");
    write_file(&dir2, "b.rs", "fn b() {}\n");
    let langs = scan(
        &[dir1.path().to_path_buf(), dir2.path().to_path_buf()],
        &default_opts(),
    )?;
    let rust = langs.get(&tokei::LanguageType::Rust).unwrap();
    // Should aggregate both files
    assert!(rust.code >= 2);
    Ok(())
}

#[test]
fn scan_mixed_nonexistent_and_valid_returns_error() {
    let dir = TempDir::new().unwrap();
    write_file(&dir, "a.rs", "fn f() {}\n");
    let bad = dir.path().join("ghost");
    let result = scan(&[dir.path().to_path_buf(), bad], &default_opts());
    assert!(result.is_err(), "should error on any nonexistent path");
}

// ===========================================================================
// 9  Determinism
// ===========================================================================

#[test]
fn scan_deterministic_repeated_calls() -> Result<()> {
    let dir = TempDir::new()?;
    write_file(&dir, "main.py", "x = 1\ny = 2\n# comment\n\nz = 3\n");
    let r1 = scan(&[dir.path().to_path_buf()], &default_opts())?;
    let r2 = scan(&[dir.path().to_path_buf()], &default_opts())?;

    let py1 = r1.get(&tokei::LanguageType::Python).unwrap();
    let py2 = r2.get(&tokei::LanguageType::Python).unwrap();
    assert_eq!(py1.code, py2.code);
    assert_eq!(py1.blanks, py2.blanks);
    assert_eq!(py1.comments, py2.comments);
    Ok(())
}

#[test]
fn scan_same_results_with_config_none_twice() -> Result<()> {
    let opts = ScanOptions {
        config: ConfigMode::None,
        ..default_opts()
    };
    let r1 = scan(&[crate_src()], &opts)?;
    let r2 = scan(&[crate_src()], &opts)?;
    let rust1 = r1.get(&tokei::LanguageType::Rust).unwrap();
    let rust2 = r2.get(&tokei::LanguageType::Rust).unwrap();
    assert_eq!(rust1.code, rust2.code);
    assert_eq!(rust1.lines(), rust2.lines());
    Ok(())
}

// ===========================================================================
// 10  Property tests
// ===========================================================================

mod properties {
    use super::*;
    use proptest::prelude::*;

    fn arb_scan_options() -> impl Strategy<Value = ScanOptions> {
        (
            any::<bool>(),
            any::<bool>(),
            any::<bool>(),
            any::<bool>(),
            any::<bool>(),
            any::<bool>(),
        )
            .prop_map(
                |(
                    hidden,
                    no_ignore,
                    no_ignore_parent,
                    no_ignore_dot,
                    no_ignore_vcs,
                    doc_comments,
                )| {
                    ScanOptions {
                        excluded: vec![],
                        config: ConfigMode::None,
                        hidden,
                        no_ignore,
                        no_ignore_parent,
                        no_ignore_dot,
                        no_ignore_vcs,
                        treat_doc_strings_as_comments: doc_comments,
                    }
                },
            )
    }

    proptest! {
        #[test]
        fn scan_never_panics_with_arbitrary_flags(opts in arb_scan_options()) {
            let dir = TempDir::new().unwrap();
            fs::write(dir.path().join("test.py"), "x = 1\n").unwrap();
            let _ = scan(&[dir.path().to_path_buf()], &opts);
        }

        #[test]
        fn line_sum_always_equals_total(opts in arb_scan_options()) {
            let dir = TempDir::new().unwrap();
            fs::write(dir.path().join("main.py"), "# c\nx = 1\n\ny = 2\n").unwrap();
            if let Ok(langs) = scan(&[dir.path().to_path_buf()], &opts) {
                for (_, lang) in langs.iter() {
                    let sum = lang.code + lang.blanks + lang.comments;
                    prop_assert_eq!(sum, lang.lines());
                }
            }
        }

        #[test]
        fn code_fields_sum_correctly(opts in arb_scan_options()) {
            let dir = TempDir::new().unwrap();
            fs::write(dir.path().join("f.rs"), "fn f() {}\n").unwrap();
            if let Ok(langs) = scan(&[dir.path().to_path_buf()], &opts) {
                for (_, lang) in langs.iter() {
                    let sum = lang.code + lang.blanks + lang.comments;
                    prop_assert_eq!(sum, lang.lines());
                }
            }
        }
    }
}

// ===========================================================================
// 11  Boundary tests
// ===========================================================================

#[test]
fn scan_file_with_only_newlines() -> Result<()> {
    let dir = TempDir::new()?;
    write_file(&dir, "newlines.py", "\n\n\n\n\n");
    let langs = scan(&[dir.path().to_path_buf()], &default_opts())?;
    if let Some(py) = langs.get(&tokei::LanguageType::Python) {
        assert_eq!(py.code, 0);
        assert!(py.blanks > 0);
    }
    Ok(())
}

#[test]
fn scan_file_with_mixed_line_endings() -> Result<()> {
    let dir = TempDir::new()?;
    write_file(&dir, "mixed.py", "x = 1\r\ny = 2\nz = 3\r\n");
    let langs = scan(&[dir.path().to_path_buf()], &default_opts())?;
    let py = langs.get(&tokei::LanguageType::Python).unwrap();
    assert_eq!(
        py.code, 3,
        "should count 3 code lines regardless of line endings"
    );
    Ok(())
}

#[test]
fn scan_deeply_nested_source_file() -> Result<()> {
    let dir = TempDir::new()?;
    write_file(&dir, "a/b/c/d/e/deep.py", "x = 1\n");
    let langs = scan(&[dir.path().to_path_buf()], &default_opts())?;
    let py = langs.get(&tokei::LanguageType::Python).unwrap();
    assert_eq!(py.code, 1);
    Ok(())
}

#[test]
fn scan_many_small_files() -> Result<()> {
    let dir = TempDir::new()?;
    for i in 0..100 {
        write_file(&dir, &format!("f{i:03}.py"), &format!("x_{i} = {i}\n"));
    }
    let langs = scan(&[dir.path().to_path_buf()], &default_opts())?;
    let py = langs.get(&tokei::LanguageType::Python).unwrap();
    assert_eq!(py.code, 100, "100 single-line files = 100 code lines");
    Ok(())
}

#[test]
fn scan_file_with_unicode_content() -> Result<()> {
    let dir = TempDir::new()?;
    write_file(&dir, "unicode.py", "# 日本語コメント\nx = '你好世界'\n");
    let langs = scan(&[dir.path().to_path_buf()], &default_opts())?;
    let py = langs.get(&tokei::LanguageType::Python).unwrap();
    assert_eq!(py.code, 1);
    assert_eq!(py.comments, 1);
    Ok(())
}

#[test]
fn scan_markdown_file() -> Result<()> {
    let dir = TempDir::new()?;
    write_file(
        &dir,
        "README.md",
        "# Title\n\nSome text.\n\n## Section\n\nMore text.\n",
    );
    let langs = scan(&[dir.path().to_path_buf()], &default_opts())?;
    assert!(langs.get(&tokei::LanguageType::Markdown).is_some());
    Ok(())
}

#[test]
fn scan_yaml_file() -> Result<()> {
    let dir = TempDir::new()?;
    write_file(
        &dir,
        "config.yaml",
        "key: value\nlist:\n  - item1\n  - item2\n",
    );
    let langs = scan(&[dir.path().to_path_buf()], &default_opts())?;
    assert!(langs.get(&tokei::LanguageType::Yaml).is_some());
    Ok(())
}
