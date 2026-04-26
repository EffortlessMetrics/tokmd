//! Deep coverage tests for `tokmd-analysis-api-surface`.
//!
//! Exercises public function detection, export counting, edge cases
//! (empty files, no exports, all private), and deterministic output ordering.

use std::fs;
use std::path::PathBuf;

use tokmd_analysis_api_surface::build_api_surface_report;
use tokmd_analysis_types::AnalysisLimits;
use tokmd_analysis_types::ApiSurfaceReport;
use tokmd_types::{ChildIncludeMode, ExportData, FileKind, FileRow};

// ── Helpers ─────────────────────────────────────────────────────

fn make_row(path: &str, module: &str, lang: &str) -> FileRow {
    FileRow {
        path: path.to_string(),
        module: module.to_string(),
        lang: lang.to_string(),
        kind: FileKind::Parent,
        code: 10,
        comments: 2,
        blanks: 1,
        lines: 13,
        bytes: 100,
        tokens: 30,
    }
}

fn make_export(rows: Vec<FileRow>) -> ExportData {
    ExportData {
        rows,
        module_roots: vec![],
        module_depth: 1,
        children: ChildIncludeMode::Separate,
    }
}

fn default_limits() -> AnalysisLimits {
    AnalysisLimits::default()
}

fn write_temp_files(files: &[(&str, &str)]) -> (tempfile::TempDir, Vec<PathBuf>) {
    let dir = tempfile::tempdir().expect("create tempdir");
    let mut paths = Vec::new();
    for (rel, content) in files {
        let full = dir.path().join(rel);
        if let Some(parent) = full.parent() {
            fs::create_dir_all(parent).unwrap();
        }
        fs::write(&full, content).unwrap();
        paths.push(PathBuf::from(rel));
    }
    (dir, paths)
}

fn build_single_file(code: &str, filename: &str, lang: &str) -> ApiSurfaceReport {
    let (dir, paths) = write_temp_files(&[(filename, code)]);
    let export = make_export(vec![make_row(filename, "src", lang)]);
    build_api_surface_report(dir.path(), &paths, &export, &default_limits()).unwrap()
}

// ===========================================================================
// Public function detection – Rust
// ===========================================================================

#[test]
fn rust_pub_fn_counted() {
    let report = build_single_file("pub fn hello() {}\n", "lib.rs", "Rust");
    assert_eq!(report.public_items, 1);
    assert_eq!(report.total_items, 1);
}

#[test]
fn rust_pub_struct_counted() {
    let report = build_single_file("pub struct Foo;\n", "lib.rs", "Rust");
    assert_eq!(report.public_items, 1);
}

#[test]
fn rust_pub_enum_counted() {
    let report = build_single_file("pub enum Bar { A, B }\n", "lib.rs", "Rust");
    assert_eq!(report.public_items, 1);
}

#[test]
fn rust_pub_trait_counted() {
    let report = build_single_file("pub trait MyTrait {}\n", "lib.rs", "Rust");
    assert_eq!(report.public_items, 1);
}

#[test]
fn rust_pub_const_counted() {
    let report = build_single_file("pub const VALUE: i32 = 42;\n", "lib.rs", "Rust");
    assert_eq!(report.public_items, 1);
}

#[test]
fn rust_pub_async_fn_counted() {
    let report = build_single_file("pub async fn run() {}\n", "lib.rs", "Rust");
    assert_eq!(report.public_items, 1);
}

#[test]
fn rust_pub_crate_counted_as_public() {
    let report = build_single_file("pub(crate) fn internal() {}\n", "lib.rs", "Rust");
    assert_eq!(report.public_items, 1);
}

#[test]
fn rust_private_fn_not_public() {
    let report = build_single_file("fn private_fn() {}\n", "lib.rs", "Rust");
    assert_eq!(report.public_items, 0);
    assert_eq!(report.internal_items, 1);
}

// ===========================================================================
// Public function detection – JavaScript/TypeScript
// ===========================================================================

#[test]
fn js_export_function_counted() {
    let report = build_single_file("export function greet() {}\n", "app.js", "JavaScript");
    assert_eq!(report.public_items, 1);
}

#[test]
fn ts_export_class_counted() {
    let report = build_single_file("export class Widget {}\n", "widget.ts", "TypeScript");
    assert_eq!(report.public_items, 1);
}

#[test]
fn ts_export_interface_counted() {
    let report = build_single_file("export interface Config {}\n", "config.ts", "TypeScript");
    assert_eq!(report.public_items, 1);
}

#[test]
fn js_non_export_function_is_internal() {
    let report = build_single_file("function helper() {}\n", "util.js", "JavaScript");
    assert_eq!(report.public_items, 0);
    assert_eq!(report.internal_items, 1);
}

// ===========================================================================
// Public function detection – Python
// ===========================================================================

#[test]
fn python_public_def_counted() {
    let report = build_single_file("def greet():\n    pass\n", "app.py", "Python");
    assert_eq!(report.public_items, 1);
}

#[test]
fn python_class_counted() {
    let report = build_single_file("class Foo:\n    pass\n", "foo.py", "Python");
    assert_eq!(report.public_items, 1);
}

#[test]
fn python_private_def_is_internal() {
    let report = build_single_file("def _helper():\n    pass\n", "util.py", "Python");
    assert_eq!(report.public_items, 0);
    assert_eq!(report.internal_items, 1);
}

// ===========================================================================
// Public function detection – Go
// ===========================================================================

#[test]
fn go_uppercase_func_is_public() {
    let report = build_single_file("func Hello() {}\n", "main.go", "Go");
    assert_eq!(report.public_items, 1);
}

#[test]
fn go_lowercase_func_is_internal() {
    let report = build_single_file("func hello() {}\n", "main.go", "Go");
    assert_eq!(report.public_items, 0);
    assert_eq!(report.internal_items, 1);
}

#[test]
fn go_uppercase_type_is_public() {
    let report = build_single_file("type Config struct {}\n", "config.go", "Go");
    assert_eq!(report.public_items, 1);
}

// ===========================================================================
// Public function detection – Java
// ===========================================================================

#[test]
fn java_public_class_counted() {
    let report = build_single_file("public class App {}\n", "App.java", "Java");
    assert_eq!(report.public_items, 1);
}

#[test]
fn java_private_class_is_internal() {
    let report = build_single_file("private class Helper {}\n", "Helper.java", "Java");
    assert_eq!(report.public_items, 0);
    assert_eq!(report.internal_items, 1);
}

// ===========================================================================
// Export counting – multiple symbols
// ===========================================================================

#[test]
fn rust_multiple_symbols_counted() {
    let code = "pub fn a() {}\npub fn b() {}\nfn c() {}\npub struct D;\n";
    let report = build_single_file(code, "lib.rs", "Rust");
    assert_eq!(report.public_items, 3);
    assert_eq!(report.internal_items, 1);
    assert_eq!(report.total_items, 4);
}

// ===========================================================================
// Edge case: empty file
// ===========================================================================

#[test]
fn empty_file_produces_zero_items() {
    let report = build_single_file("", "empty.rs", "Rust");
    assert_eq!(report.total_items, 0);
    assert_eq!(report.public_items, 0);
    assert_eq!(report.internal_items, 0);
}

// ===========================================================================
// Edge case: no exports (all private)
// ===========================================================================

#[test]
fn all_private_rust_file() {
    let code = "fn private_a() {}\nfn private_b() {}\nstruct Internal;\n";
    let report = build_single_file(code, "lib.rs", "Rust");
    assert_eq!(report.public_items, 0);
    assert_eq!(report.internal_items, 3);
    assert!(report.top_exporters.is_empty());
}

// ===========================================================================
// Edge case: file with only comments
// ===========================================================================

#[test]
fn comments_only_file_produces_zero_items() {
    let code = "// This is a comment\n// Another comment\n/// Doc comment\n";
    let report = build_single_file(code, "lib.rs", "Rust");
    assert_eq!(report.total_items, 0);
}

// ===========================================================================
// Edge case: unsupported language skipped
// ===========================================================================

#[test]
fn unsupported_language_produces_empty_report() {
    let (dir, paths) = write_temp_files(&[("style.css", "body { color: red; }")]);
    let export = make_export(vec![make_row("style.css", "src", "CSS")]);
    let report = build_api_surface_report(dir.path(), &paths, &export, &default_limits()).unwrap();
    assert_eq!(report.total_items, 0);
}

// ===========================================================================
// Deterministic output ordering
// ===========================================================================

#[test]
fn report_deterministic_across_calls() {
    let code = "pub fn alpha() {}\npub fn beta() {}\nfn gamma() {}\n";
    let r1 = build_single_file(code, "lib.rs", "Rust");
    let r2 = build_single_file(code, "lib.rs", "Rust");
    let j1 = serde_json::to_string(&r1).unwrap();
    let j2 = serde_json::to_string(&r2).unwrap();
    assert_eq!(j1, j2);
}

#[test]
fn by_language_map_sorted_by_key() {
    let files: &[(&str, &str)] = &[
        ("app.py", "def greet():\n    pass\n"),
        ("lib.rs", "pub fn hello() {}\n"),
        ("main.go", "func Hello() {}\n"),
    ];
    let (dir, paths) = write_temp_files(files);
    let export = make_export(vec![
        make_row("app.py", "src", "Python"),
        make_row("lib.rs", "src", "Rust"),
        make_row("main.go", "src", "Go"),
    ]);
    let report = build_api_surface_report(dir.path(), &paths, &export, &default_limits()).unwrap();
    let keys: Vec<&String> = report.by_language.keys().collect();
    // BTreeMap ensures sorted order
    let mut sorted = keys.clone();
    sorted.sort();
    assert_eq!(keys, sorted);
}

#[test]
fn top_exporters_sorted_by_public_items_desc() {
    let files: &[(&str, &str)] = &[
        ("a.rs", "pub fn x() {}\n"),
        ("b.rs", "pub fn y() {}\npub fn z() {}\npub fn w() {}\n"),
    ];
    let (dir, paths) = write_temp_files(files);
    let export = make_export(vec![
        make_row("a.rs", "src", "Rust"),
        make_row("b.rs", "src", "Rust"),
    ]);
    let report = build_api_surface_report(dir.path(), &paths, &export, &default_limits()).unwrap();
    if report.top_exporters.len() >= 2 {
        assert!(report.top_exporters[0].public_items >= report.top_exporters[1].public_items);
    }
}

// ===========================================================================
// Public ratio calculations
// ===========================================================================

#[test]
fn public_ratio_is_zero_when_no_items() {
    let report = build_single_file("", "empty.rs", "Rust");
    assert_eq!(report.public_ratio, 0.0);
}

#[test]
fn public_ratio_is_one_when_all_public() {
    let report = build_single_file("pub fn a() {}\npub fn b() {}\n", "lib.rs", "Rust");
    assert!((report.public_ratio - 1.0).abs() < 0.001);
}

#[test]
fn documented_ratio_calculated() {
    let code = "/// Documented\npub fn documented() {}\npub fn undocumented() {}\n";
    let report = build_single_file(code, "lib.rs", "Rust");
    // 1 documented out of 2 public = 0.5
    assert!((report.documented_ratio - 0.5).abs() < 0.001);
}

// ===========================================================================
// Serialization roundtrip
// ===========================================================================

#[test]
fn report_serializes_and_deserializes() {
    let report = build_single_file("pub fn hello() {}\nfn world() {}\n", "lib.rs", "Rust");
    let json = serde_json::to_string(&report).unwrap();
    let deser: ApiSurfaceReport = serde_json::from_str(&json).unwrap();
    assert_eq!(deser.total_items, report.total_items);
    assert_eq!(deser.public_items, report.public_items);
    assert_eq!(deser.internal_items, report.internal_items);
}
