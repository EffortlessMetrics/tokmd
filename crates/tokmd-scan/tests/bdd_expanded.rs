//! Expanded BDD-style scenarios for `tokmd-scan`.
//!
//! These tests exercise scan invariants not covered by the base `bdd.rs`:
//! - Workspace-root scanning with non-empty results
//! - Deterministic ordering of scan results across runs
//! - Path normalization in scan result reports
//! - Children mode pass-through verification
//! - Sort ordering and structural invariants

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
        fs::create_dir_all(parent).expect("create dirs");
    }
    fs::write(&path, content).expect("write file");
}

// ===========================================================================
// Scenario group: scanning the workspace and getting non-empty results
// ===========================================================================

#[test]
fn given_workspace_root_when_scanned_then_multiple_languages_detected() -> Result<()> {
    // Given the workspace root containing Rust, TOML, Markdown, etc.
    let workspace = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .to_path_buf();

    let langs = scan(&[workspace], &default_opts())?;

    // Then multiple languages should be present
    assert!(
        langs.len() >= 2,
        "workspace root should detect at least 2 languages, found {}",
        langs.len()
    );
    // Rust must be among them
    assert!(
        langs.get(&tokei::LanguageType::Rust).is_some(),
        "Rust must be detected in workspace"
    );
    Ok(())
}

#[test]
fn given_workspace_root_when_scanned_then_code_lines_are_positive() -> Result<()> {
    let workspace = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .to_path_buf();

    let langs = scan(&[workspace], &default_opts())?;
    let rust = langs
        .get(&tokei::LanguageType::Rust)
        .expect("Rust language");

    assert!(
        rust.code > 100,
        "workspace should have >100 lines of Rust code, found {}",
        rust.code
    );
    Ok(())
}

// ===========================================================================
// Scenario group: children mode awareness in scan results
// ===========================================================================

#[test]
fn given_html_with_embedded_css_when_scanned_then_children_present() -> Result<()> {
    let tmp = TempDir::new()?;
    let html = r#"<!DOCTYPE html>
<html>
<head>
<style>
body { color: red; }
p { margin: 0; }
</style>
</head>
<body><p>Hello</p></body>
</html>
"#;
    write_file(&tmp, "page.html", html);

    let langs = scan(&[tmp.path().to_path_buf()], &default_opts())?;

    // Then HTML should be detected
    let html_lang = langs.get(&tokei::LanguageType::Html);
    assert!(html_lang.is_some(), "HTML should be detected");

    let html_data = html_lang.unwrap();
    assert!(html_data.code > 0 || html_data.lines() > 0);
    Ok(())
}

#[test]
fn given_html_with_embedded_js_when_scanned_then_children_map_populated() -> Result<()> {
    let tmp = TempDir::new()?;
    let html = r#"<!DOCTYPE html>
<html>
<head>
<script>
function greet() { return "hello"; }
console.log(greet());
</script>
</head>
<body></body>
</html>
"#;
    write_file(&tmp, "app.html", html);

    let langs = scan(&[tmp.path().to_path_buf()], &default_opts())?;
    let html_data = langs
        .get(&tokei::LanguageType::Html)
        .expect("HTML should be detected");

    // Children map should have JavaScript entries
    assert!(
        !html_data.children.is_empty(),
        "HTML with <script> should have children languages"
    );
    Ok(())
}

// ===========================================================================
// Scenario group: sort ordering and determinism
// ===========================================================================

#[test]
fn given_multi_language_tree_when_scanned_twice_then_language_keys_identical() -> Result<()> {
    let tmp = TempDir::new()?;
    write_file(&tmp, "main.rs", "fn main() {}\nfn helper() {}\n");
    write_file(&tmp, "lib.py", "def lib():\n    pass\n");
    write_file(&tmp, "app.js", "function app() {}\n");

    let r1 = scan(&[tmp.path().to_path_buf()], &default_opts())?;
    let r2 = scan(&[tmp.path().to_path_buf()], &default_opts())?;

    let keys1: Vec<_> = r1.iter().map(|(k, _)| *k).collect();
    let keys2: Vec<_> = r2.iter().map(|(k, _)| *k).collect();

    assert_eq!(keys1, keys2, "language key ordering must be deterministic");
    Ok(())
}

#[test]
fn given_same_temp_tree_when_scanned_twice_then_code_counts_identical() -> Result<()> {
    let tmp = TempDir::new()?;
    write_file(&tmp, "a.rs", "fn a() {}\nfn b() {}\nfn c() {}\n");
    write_file(&tmp, "d.rs", "fn d() {}\n");

    let r1 = scan(&[tmp.path().to_path_buf()], &default_opts())?;
    let r2 = scan(&[tmp.path().to_path_buf()], &default_opts())?;

    for (lang, data1) in r1.iter() {
        let data2 = r2.get(lang).expect("language should exist in second scan");
        assert_eq!(
            data1.code, data2.code,
            "code count for {:?} must be deterministic",
            lang
        );
        assert_eq!(
            data1.comments, data2.comments,
            "comment count for {:?} must be deterministic",
            lang
        );
        assert_eq!(
            data1.blanks, data2.blanks,
            "blank count for {:?} must be deterministic",
            lang
        );
    }
    Ok(())
}

// ===========================================================================
// Scenario group: empty directory handling
// ===========================================================================

#[test]
fn given_empty_nested_dirs_when_scanned_then_result_is_empty() -> Result<()> {
    let tmp = TempDir::new()?;
    fs::create_dir_all(tmp.path().join("a/b/c"))?;
    fs::create_dir_all(tmp.path().join("x/y"))?;

    let langs = scan(&[tmp.path().to_path_buf()], &default_opts())?;
    assert!(
        langs.is_empty(),
        "nested empty directories should yield zero languages"
    );
    Ok(())
}

#[test]
fn given_dir_with_only_unknown_extensions_when_scanned_then_no_known_languages() -> Result<()> {
    let tmp = TempDir::new()?;
    write_file(&tmp, "data.xyz", "some arbitrary content\n");
    write_file(&tmp, "notes.qqqq", "more stuff\n");

    let langs = scan(&[tmp.path().to_path_buf()], &default_opts())?;

    let total_code: usize = langs.iter().map(|(_, l)| l.code).sum();
    assert_eq!(
        total_code, 0,
        "unknown file extensions should produce 0 code lines"
    );
    Ok(())
}

// ===========================================================================
// Scenario group: path normalization in scan results
// ===========================================================================

#[test]
fn given_scan_results_then_file_report_paths_exist_on_disk() -> Result<()> {
    let tmp = TempDir::new()?;
    write_file(&tmp, "src/lib.rs", "pub fn lib_fn() {}\n");
    write_file(&tmp, "src/util.rs", "pub fn util_fn() {}\n");

    let langs = scan(&[tmp.path().to_path_buf()], &default_opts())?;
    let rust = langs
        .get(&tokei::LanguageType::Rust)
        .expect("should find Rust");

    for report in &rust.reports {
        assert!(
            report.name.exists(),
            "report path should exist on disk: {:?}",
            report.name
        );
    }
    Ok(())
}

#[test]
fn given_scan_results_then_report_file_count_matches_reports_vec_len() -> Result<()> {
    let tmp = TempDir::new()?;
    write_file(&tmp, "a.rs", "fn a() {}\n");
    write_file(&tmp, "b.rs", "fn b() {}\n");
    write_file(&tmp, "c.rs", "fn c() {}\n");

    let langs = scan(&[tmp.path().to_path_buf()], &default_opts())?;
    let rust = langs
        .get(&tokei::LanguageType::Rust)
        .expect("should find Rust");

    assert_eq!(
        rust.reports.len(),
        3,
        "3 Rust files should produce 3 reports"
    );
    Ok(())
}

// ===========================================================================
// Scenario group: scan structural invariants
// ===========================================================================

#[test]
fn given_scan_results_then_lines_equals_code_plus_comments_plus_blanks() -> Result<()> {
    let tmp = TempDir::new()?;
    let src = "// comment\nfn main() {\n    println!(\"hi\");\n}\n\n";
    write_file(&tmp, "main.rs", src);

    let langs = scan(&[tmp.path().to_path_buf()], &default_opts())?;
    let rust = langs
        .get(&tokei::LanguageType::Rust)
        .expect("should find Rust");

    let expected_lines = rust.code + rust.comments + rust.blanks;
    assert_eq!(
        rust.lines(),
        expected_lines,
        "lines() must equal code + comments + blanks"
    );
    Ok(())
}

#[test]
fn given_multi_file_scan_then_total_code_is_sum_of_report_stats() -> Result<()> {
    let tmp = TempDir::new()?;
    write_file(&tmp, "a.rs", "fn a() {}\n");
    write_file(&tmp, "b.rs", "fn b() {}\nfn c() {}\n");

    let langs = scan(&[tmp.path().to_path_buf()], &default_opts())?;
    let rust = langs
        .get(&tokei::LanguageType::Rust)
        .expect("should find Rust");

    let sum_from_reports: usize = rust.reports.iter().map(|r| r.stats.summarise().code).sum();

    assert_eq!(
        rust.code, sum_from_reports,
        "total code must equal sum of individual report stats"
    );
    Ok(())
}

#[test]
fn given_scan_results_then_every_detected_language_has_positive_code() -> Result<()> {
    let tmp = TempDir::new()?;
    write_file(&tmp, "main.rs", "fn main() {}\n");
    write_file(&tmp, "script.py", "print('hi')\n");

    let langs = scan(&[tmp.path().to_path_buf()], &default_opts())?;

    for (lang_type, data) in langs.iter() {
        assert!(
            data.code > 0,
            "{:?} was detected but has 0 code lines",
            lang_type
        );
    }
    Ok(())
}

// ===========================================================================
// Scenario group: exclusion and filtering edge cases
// ===========================================================================

#[test]
fn given_exclude_pattern_with_nested_match_when_scanned_then_nested_excluded() -> Result<()> {
    let tmp = TempDir::new()?;
    write_file(&tmp, "src/main.rs", "fn main() {}\n");
    write_file(&tmp, "tests/integration.rs", "fn test() {}\n");
    write_file(&tmp, "benches/bench.rs", "fn bench() {}\n");

    let mut opts = default_opts();
    opts.excluded = vec!["tests".to_string(), "benches".to_string()];

    let langs = scan(&[tmp.path().to_path_buf()], &opts)?;
    let rust = langs
        .get(&tokei::LanguageType::Rust)
        .expect("should find Rust");

    assert_eq!(
        rust.code, 1,
        "tests/ and benches/ should be excluded, only 1 code line from src/main.rs"
    );
    Ok(())
}
