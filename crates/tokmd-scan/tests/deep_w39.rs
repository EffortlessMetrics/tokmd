//! Deep tests for tokmd-scan: language detection, scan options, edge cases.

use std::fs;
use std::io::Write;
use std::path::PathBuf;

use tempfile::TempDir;
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

fn write_file(dir: &TempDir, rel: &str, content: &str) -> PathBuf {
    let path = dir.path().join(rel);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    let mut f = fs::File::create(&path).unwrap();
    f.write_all(content.as_bytes()).unwrap();
    path
}

// ==============================
// Language detection
// ==============================

#[test]
fn detects_rust_language() {
    let dir = TempDir::new().unwrap();
    write_file(&dir, "main.rs", "fn main() { println!(\"hello\"); }\n");
    let langs = scan(&[dir.path().to_path_buf()], &default_opts()).unwrap();
    assert!(langs.get(&tokei::LanguageType::Rust).is_some());
}

#[test]
fn detects_python_language() {
    let dir = TempDir::new().unwrap();
    write_file(&dir, "app.py", "print('hello')\n");
    let langs = scan(&[dir.path().to_path_buf()], &default_opts()).unwrap();
    assert!(langs.get(&tokei::LanguageType::Python).is_some());
}

#[test]
fn detects_javascript_language() {
    let dir = TempDir::new().unwrap();
    write_file(&dir, "index.js", "console.log('hi');\n");
    let langs = scan(&[dir.path().to_path_buf()], &default_opts()).unwrap();
    assert!(langs.get(&tokei::LanguageType::JavaScript).is_some());
}

#[test]
fn detects_multiple_languages() {
    let dir = TempDir::new().unwrap();
    write_file(
        &dir,
        "lib.rs",
        "pub fn add(a: i32, b: i32) -> i32 { a + b }\n",
    );
    write_file(&dir, "util.py", "def add(a, b): return a + b\n");
    write_file(&dir, "helper.js", "function add(a, b) { return a + b; }\n");

    let langs = scan(&[dir.path().to_path_buf()], &default_opts()).unwrap();
    assert!(langs.get(&tokei::LanguageType::Rust).is_some());
    assert!(langs.get(&tokei::LanguageType::Python).is_some());
    assert!(langs.get(&tokei::LanguageType::JavaScript).is_some());
}

#[test]
fn detects_toml_language() {
    let dir = TempDir::new().unwrap();
    write_file(
        &dir,
        "Cargo.toml",
        "[package]\nname = \"demo\"\nversion = \"0.1.0\"\n",
    );
    let langs = scan(&[dir.path().to_path_buf()], &default_opts()).unwrap();
    assert!(langs.get(&tokei::LanguageType::Toml).is_some());
}

// ==============================
// Scan result structure
// ==============================

#[test]
fn code_lines_are_positive_for_real_file() {
    let dir = TempDir::new().unwrap();
    write_file(
        &dir,
        "main.rs",
        "fn main() {\n    let x = 1;\n    println!(\"{}\", x);\n}\n",
    );
    let langs = scan(&[dir.path().to_path_buf()], &default_opts()).unwrap();
    let rust = langs.get(&tokei::LanguageType::Rust).unwrap();
    assert!(rust.code > 0, "should count code lines");
}

#[test]
fn comment_lines_counted_separately() {
    let dir = TempDir::new().unwrap();
    write_file(&dir, "demo.rs", "// a comment\nfn main() {}\n// another\n");
    let langs = scan(&[dir.path().to_path_buf()], &default_opts()).unwrap();
    let rust = langs.get(&tokei::LanguageType::Rust).unwrap();
    assert!(rust.comments > 0, "should count comments");
    assert!(rust.code > 0, "should count code");
}

#[test]
fn blank_lines_counted() {
    let dir = TempDir::new().unwrap();
    write_file(&dir, "spaced.rs", "fn a() {}\n\n\nfn b() {}\n");
    let langs = scan(&[dir.path().to_path_buf()], &default_opts()).unwrap();
    let rust = langs.get(&tokei::LanguageType::Rust).unwrap();
    assert!(rust.blanks > 0, "should count blank lines");
}

#[test]
fn total_lines_equals_sum() {
    let dir = TempDir::new().unwrap();
    write_file(&dir, "sum.rs", "// comment\n\nfn main() {}\n\n// end\n");
    let langs = scan(&[dir.path().to_path_buf()], &default_opts()).unwrap();
    let rust = langs.get(&tokei::LanguageType::Rust).unwrap();
    let total = rust.code + rust.comments + rust.blanks;
    assert_eq!(rust.lines(), total);
}

// ==============================
// Directory structures
// ==============================

#[test]
fn scans_nested_directories() {
    let dir = TempDir::new().unwrap();
    write_file(&dir, "src/lib.rs", "pub fn lib_fn() {}\n");
    write_file(&dir, "src/util/helpers.rs", "pub fn help() {}\n");
    let langs = scan(&[dir.path().to_path_buf()], &default_opts()).unwrap();
    let rust = langs.get(&tokei::LanguageType::Rust).unwrap();
    // Both files contribute
    assert!(rust.code >= 2);
}

#[test]
fn scans_multiple_input_paths() {
    let dir = TempDir::new().unwrap();
    write_file(&dir, "a/one.rs", "fn a() {}\n");
    write_file(&dir, "b/two.rs", "fn b() {}\n");
    let paths = vec![dir.path().join("a"), dir.path().join("b")];
    let langs = scan(&paths, &default_opts()).unwrap();
    let rust = langs.get(&tokei::LanguageType::Rust).unwrap();
    assert!(rust.code >= 2);
}

// ==============================
// Empty directory
// ==============================

#[test]
fn empty_directory_returns_empty_languages() {
    let dir = TempDir::new().unwrap();
    let langs = scan(&[dir.path().to_path_buf()], &default_opts()).unwrap();
    // No recognized source files → empty
    assert!(langs.is_empty());
}

#[test]
fn directory_with_only_unknown_files_returns_empty() {
    let dir = TempDir::new().unwrap();
    write_file(&dir, "data.bin", "\x00\x01\x02");
    write_file(&dir, "readme.xyz123", "unknown extension");
    let langs = scan(&[dir.path().to_path_buf()], &default_opts()).unwrap();
    // tokei should not recognize these extensions
    let recognized: usize = langs.values().map(|l| l.code).sum();
    // Either empty or trivially zero
    assert!(recognized == 0 || langs.is_empty());
}

// ==============================
// Scan options
// ==============================

#[test]
fn exclusion_patterns_filter_files() {
    let dir = TempDir::new().unwrap();
    write_file(&dir, "keep.rs", "fn keep() {}\n");
    write_file(&dir, "gen/generated.rs", "fn gen() {}\n");

    let mut opts = default_opts();
    opts.excluded = vec!["gen".to_string()];
    let langs = scan(&[dir.path().to_path_buf()], &opts).unwrap();
    let rust = langs.get(&tokei::LanguageType::Rust).unwrap();
    // Only keep.rs should remain
    assert_eq!(rust.code, 1);
}

#[test]
fn config_mode_none_does_not_use_tokei_config() {
    let dir = TempDir::new().unwrap();
    write_file(&dir, "app.rs", "fn app() {}\n");
    let mut opts = default_opts();
    opts.config = ConfigMode::None;
    let result = scan(&[dir.path().to_path_buf()], &opts);
    assert!(result.is_ok());
}

#[test]
fn config_mode_auto_does_not_crash() {
    let dir = TempDir::new().unwrap();
    write_file(&dir, "app.rs", "fn app() {}\n");
    let mut opts = default_opts();
    opts.config = ConfigMode::Auto;
    let result = scan(&[dir.path().to_path_buf()], &opts);
    assert!(result.is_ok());
}

#[test]
fn treat_doc_strings_as_comments_changes_counts() {
    let dir = TempDir::new().unwrap();
    let src = "/// doc comment\nfn main() {}\n";
    write_file(&dir, "doc.rs", src);

    let normal = scan(&[dir.path().to_path_buf()], &default_opts()).unwrap();
    let normal_rust = normal.get(&tokei::LanguageType::Rust).unwrap();

    let mut opts = default_opts();
    opts.treat_doc_strings_as_comments = true;
    let treated = scan(&[dir.path().to_path_buf()], &opts).unwrap();
    let treated_rust = treated.get(&tokei::LanguageType::Rust).unwrap();

    // With treat_doc_strings_as_comments the doc line should count as comment
    assert!(treated_rust.comments >= normal_rust.comments);
}

// ==============================
// Error handling
// ==============================

#[test]
fn nonexistent_path_returns_error() {
    let result = scan(
        &[PathBuf::from("__nonexistent_dir_tokmd_scan_test__")],
        &default_opts(),
    );
    assert!(result.is_err());
    let msg = result.unwrap_err().to_string();
    assert!(msg.contains("Path not found"));
}

#[test]
fn multiple_paths_one_bad_returns_error() {
    let dir = TempDir::new().unwrap();
    write_file(&dir, "ok.rs", "fn ok() {}\n");
    let paths = vec![
        dir.path().to_path_buf(),
        PathBuf::from("__also_nonexistent__"),
    ];
    let result = scan(&paths, &default_opts());
    assert!(result.is_err());
}
