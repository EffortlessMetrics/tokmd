//! BDD-style integration tests for `tokmd-analysis-api-surface`.
//!
//! These tests exercise `build_api_surface_report` end-to-end by
//! writing temp source files and feeding matching `ExportData` rows.

use std::fs;
use std::path::PathBuf;

use tokmd_analysis_api_surface::build_api_surface_report;
use tokmd_analysis_util::AnalysisLimits;
use tokmd_types::{ChildIncludeMode, ExportData, FileKind, FileRow};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

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

/// Write a file inside a temp dir and return (root, relative_paths).
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

// ---------------------------------------------------------------------------
// Scenario: Empty inputs
// ---------------------------------------------------------------------------

#[test]
fn given_no_files_report_is_empty() {
    let dir = tempfile::tempdir().unwrap();
    let export = make_export(vec![]);
    let report = build_api_surface_report(dir.path(), &[], &export, &default_limits()).unwrap();

    assert_eq!(report.total_items, 0);
    assert_eq!(report.public_items, 0);
    assert_eq!(report.internal_items, 0);
    assert_eq!(report.public_ratio, 0.0);
    assert_eq!(report.documented_ratio, 0.0);
    assert!(report.by_language.is_empty());
    assert!(report.by_module.is_empty());
    assert!(report.top_exporters.is_empty());
}

#[test]
fn given_no_matching_rows_report_is_empty() {
    let (dir, paths) = write_temp_files(&[("src/lib.rs", "pub fn hello() {}\n")]);
    // Export data has no rows matching the file
    let export = make_export(vec![]);
    let report = build_api_surface_report(dir.path(), &paths, &export, &default_limits()).unwrap();

    assert_eq!(report.total_items, 0);
}

// ---------------------------------------------------------------------------
// Scenario: Rust public function detection
// ---------------------------------------------------------------------------

#[test]
fn given_rust_file_with_pub_fn_detects_public_item() {
    let code = "pub fn greet() {}\nfn helper() {}\n";
    let (dir, paths) = write_temp_files(&[("src/lib.rs", code)]);
    let export = make_export(vec![make_row("src/lib.rs", "src", "Rust")]);
    let report = build_api_surface_report(dir.path(), &paths, &export, &default_limits()).unwrap();

    assert_eq!(report.total_items, 2);
    assert_eq!(report.public_items, 1);
    assert_eq!(report.internal_items, 1);
    assert_eq!(report.public_ratio, 0.5);
}

#[test]
fn given_rust_file_all_public_ratio_is_one() {
    let code = "pub fn a() {}\npub fn b() {}\npub struct C;\n";
    let (dir, paths) = write_temp_files(&[("lib.rs", code)]);
    let export = make_export(vec![make_row("lib.rs", ".", "Rust")]);
    let report = build_api_surface_report(dir.path(), &paths, &export, &default_limits()).unwrap();

    assert_eq!(report.public_items, 3);
    assert_eq!(report.public_ratio, 1.0);
}

#[test]
fn given_rust_file_all_internal_ratio_is_zero() {
    let code = "fn a() {}\nfn b() {}\nstruct C;\n";
    let (dir, paths) = write_temp_files(&[("lib.rs", code)]);
    let export = make_export(vec![make_row("lib.rs", ".", "Rust")]);
    let report = build_api_surface_report(dir.path(), &paths, &export, &default_limits()).unwrap();

    assert_eq!(report.public_items, 0);
    assert_eq!(report.internal_items, 3);
    assert_eq!(report.public_ratio, 0.0);
}

// ---------------------------------------------------------------------------
// Scenario: Documented ratio
// ---------------------------------------------------------------------------

#[test]
fn given_documented_public_items_ratio_is_correct() {
    let code = "/// Documented\npub fn doc_fn() {}\npub fn undoc_fn() {}\n";
    let (dir, paths) = write_temp_files(&[("lib.rs", code)]);
    let export = make_export(vec![make_row("lib.rs", ".", "Rust")]);
    let report = build_api_surface_report(dir.path(), &paths, &export, &default_limits()).unwrap();

    assert_eq!(report.public_items, 2);
    assert_eq!(report.documented_ratio, 0.5);
}

#[test]
fn given_no_public_items_documented_ratio_is_zero() {
    let code = "fn internal() {}\n";
    let (dir, paths) = write_temp_files(&[("lib.rs", code)]);
    let export = make_export(vec![make_row("lib.rs", ".", "Rust")]);
    let report = build_api_surface_report(dir.path(), &paths, &export, &default_limits()).unwrap();

    assert_eq!(report.documented_ratio, 0.0);
}

// ---------------------------------------------------------------------------
// Scenario: Per-language breakdown
// ---------------------------------------------------------------------------

#[test]
fn given_multiple_languages_by_language_tracks_each() {
    let rust_code = "pub fn rust_pub() {}\nfn rust_priv() {}\n";
    let py_code = "def public_func():\n    pass\ndef _private():\n    pass\n";
    let (dir, paths) = write_temp_files(&[("lib.rs", rust_code), ("main.py", py_code)]);
    let export = make_export(vec![
        make_row("lib.rs", ".", "Rust"),
        make_row("main.py", ".", "Python"),
    ]);
    let report = build_api_surface_report(dir.path(), &paths, &export, &default_limits()).unwrap();

    assert_eq!(report.by_language.len(), 2);

    let rust = &report.by_language["Rust"];
    assert_eq!(rust.public_items, 1);
    assert_eq!(rust.internal_items, 1);

    let py = &report.by_language["Python"];
    assert_eq!(py.public_items, 1);
    assert_eq!(py.internal_items, 1);
}

// ---------------------------------------------------------------------------
// Scenario: Per-module breakdown
// ---------------------------------------------------------------------------

#[test]
fn given_files_in_different_modules_by_module_tracks_each() {
    let code_a = "pub fn a() {}\n";
    let code_b = "pub fn b() {}\npub fn c() {}\n";
    let (dir, paths) = write_temp_files(&[("src/a.rs", code_a), ("src/b.rs", code_b)]);
    let export = make_export(vec![
        make_row("src/a.rs", "mod_a", "Rust"),
        make_row("src/b.rs", "mod_b", "Rust"),
    ]);
    let report = build_api_surface_report(dir.path(), &paths, &export, &default_limits()).unwrap();

    assert_eq!(report.by_module.len(), 2);
    // Sorted by total_items descending → mod_b first (2 items) then mod_a (1 item)
    assert_eq!(report.by_module[0].module, "mod_b");
    assert_eq!(report.by_module[0].total_items, 2);
    assert_eq!(report.by_module[1].module, "mod_a");
    assert_eq!(report.by_module[1].total_items, 1);
}

// ---------------------------------------------------------------------------
// Scenario: Top exporters
// ---------------------------------------------------------------------------

#[test]
fn given_files_with_public_items_top_exporters_sorted() {
    let code_few = "pub fn a() {}\n";
    let code_many = "pub fn x() {}\npub fn y() {}\npub fn z() {}\n";
    let (dir, paths) = write_temp_files(&[("few.rs", code_few), ("many.rs", code_many)]);
    let export = make_export(vec![
        make_row("few.rs", ".", "Rust"),
        make_row("many.rs", ".", "Rust"),
    ]);
    let report = build_api_surface_report(dir.path(), &paths, &export, &default_limits()).unwrap();

    assert_eq!(report.top_exporters.len(), 2);
    // Sorted by public_items desc → many.rs first
    assert_eq!(report.top_exporters[0].path, "many.rs");
    assert_eq!(report.top_exporters[0].public_items, 3);
    assert_eq!(report.top_exporters[1].path, "few.rs");
    assert_eq!(report.top_exporters[1].public_items, 1);
}

#[test]
fn given_file_with_no_public_items_not_in_top_exporters() {
    let code = "fn internal_only() {}\n";
    let (dir, paths) = write_temp_files(&[("lib.rs", code)]);
    let export = make_export(vec![make_row("lib.rs", ".", "Rust")]);
    let report = build_api_surface_report(dir.path(), &paths, &export, &default_limits()).unwrap();

    assert!(report.top_exporters.is_empty());
}

// ---------------------------------------------------------------------------
// Scenario: Unsupported languages are skipped
// ---------------------------------------------------------------------------

#[test]
fn given_unsupported_language_file_is_skipped() {
    let code = "# This is Markdown\n## Heading\n";
    let (dir, paths) = write_temp_files(&[("README.md", code)]);
    let export = make_export(vec![make_row("README.md", ".", "Markdown")]);
    let report = build_api_surface_report(dir.path(), &paths, &export, &default_limits()).unwrap();

    assert_eq!(report.total_items, 0);
}

// ---------------------------------------------------------------------------
// Scenario: Child-kind rows are ignored
// ---------------------------------------------------------------------------

#[test]
fn given_child_kind_rows_they_are_excluded() {
    let code = "pub fn visible() {}\n";
    let (dir, paths) = write_temp_files(&[("lib.rs", code)]);
    let mut row = make_row("lib.rs", ".", "Rust");
    row.kind = FileKind::Child;
    let export = make_export(vec![row]);
    let report = build_api_surface_report(dir.path(), &paths, &export, &default_limits()).unwrap();

    assert_eq!(report.total_items, 0);
}

// ---------------------------------------------------------------------------
// Scenario: JavaScript / TypeScript detection
// ---------------------------------------------------------------------------

#[test]
fn given_js_file_detects_exports() {
    let code = "export function greet() {}\nfunction helper() {}\n";
    let (dir, paths) = write_temp_files(&[("index.js", code)]);
    let export = make_export(vec![make_row("index.js", ".", "JavaScript")]);
    let report = build_api_surface_report(dir.path(), &paths, &export, &default_limits()).unwrap();

    assert_eq!(report.public_items, 1);
    assert_eq!(report.internal_items, 1);
}

#[test]
fn given_ts_file_detects_interface_and_type_exports() {
    let code = "export interface IUser {}\nexport type Id = string;\ninterface Internal {}\n";
    let (dir, paths) = write_temp_files(&[("types.ts", code)]);
    let export = make_export(vec![make_row("types.ts", ".", "TypeScript")]);
    let report = build_api_surface_report(dir.path(), &paths, &export, &default_limits()).unwrap();

    assert_eq!(report.public_items, 2);
    assert_eq!(report.internal_items, 1);
}

// ---------------------------------------------------------------------------
// Scenario: Go public/private by case
// ---------------------------------------------------------------------------

#[test]
fn given_go_file_uppercase_is_public() {
    let code = "func PublicFunc() {}\nfunc privateFunc() {}\ntype MyStruct struct {}\n";
    let (dir, paths) = write_temp_files(&[("main.go", code)]);
    let export = make_export(vec![make_row("main.go", ".", "Go")]);
    let report = build_api_surface_report(dir.path(), &paths, &export, &default_limits()).unwrap();

    assert_eq!(report.public_items, 2);
    assert_eq!(report.internal_items, 1);
}

// ---------------------------------------------------------------------------
// Scenario: Java detection
// ---------------------------------------------------------------------------

#[test]
fn given_java_file_detects_public_class() {
    let code = "public class App {\n}\nclass Internal {\n}\n";
    let (dir, paths) = write_temp_files(&[("App.java", code)]);
    let export = make_export(vec![make_row("App.java", ".", "Java")]);
    let report = build_api_surface_report(dir.path(), &paths, &export, &default_limits()).unwrap();

    assert_eq!(report.public_items, 1);
    assert_eq!(report.internal_items, 1);
}

// ---------------------------------------------------------------------------
// Scenario: Python detection with docstrings
// ---------------------------------------------------------------------------

#[test]
fn given_python_file_with_docstrings_documented_ratio_correct() {
    let code = "\
def documented():\n\
    \"\"\"Has docstring.\"\"\"\n\
    pass\n\
\n\
def undocumented():\n\
    pass\n\
\n\
def _private():\n\
    \"\"\"Private doc.\"\"\"\n\
    pass\n";
    let (dir, paths) = write_temp_files(&[("mod.py", code)]);
    let export = make_export(vec![make_row("mod.py", ".", "Python")]);
    let report = build_api_surface_report(dir.path(), &paths, &export, &default_limits()).unwrap();

    // 2 public (documented, undocumented), 1 private (_private)
    assert_eq!(report.public_items, 2);
    assert_eq!(report.internal_items, 1);
    // Only "documented()" has docstring → 1/2 = 0.5
    assert_eq!(report.documented_ratio, 0.5);
}

// ---------------------------------------------------------------------------
// Scenario: Limits — max_bytes budget stops scanning
// ---------------------------------------------------------------------------

#[test]
fn given_max_bytes_limit_scanning_stops_early() {
    // Create two files; set max_bytes so only the first is scanned.
    let code_a = "pub fn a() {}\n";
    let code_b = "pub fn b() {}\npub fn c() {}\n";
    let (dir, paths) = write_temp_files(&[("a.rs", code_a), ("b.rs", code_b)]);
    let export = make_export(vec![
        make_row("a.rs", ".", "Rust"),
        make_row("b.rs", ".", "Rust"),
    ]);
    let limits = AnalysisLimits {
        max_bytes: Some(code_a.len() as u64),
        ..Default::default()
    };
    let report = build_api_surface_report(dir.path(), &paths, &export, &limits).unwrap();

    // Only first file should be scanned (budget reached after reading it)
    assert!(
        report.total_items <= 1,
        "expected at most 1 item, got {}",
        report.total_items
    );
}

// ---------------------------------------------------------------------------
// Scenario: Aggregate totals are consistent
// ---------------------------------------------------------------------------

#[test]
fn given_multiple_files_totals_equal_sum_of_parts() {
    let rust = "pub fn r1() {}\nfn r2() {}\n";
    let py = "def py_pub():\n    pass\ndef _py_priv():\n    pass\n";
    let go = "func GoPublic() {}\nfunc goPrivate() {}\n";
    let (dir, paths) = write_temp_files(&[("lib.rs", rust), ("main.py", py), ("main.go", go)]);
    let export = make_export(vec![
        make_row("lib.rs", "src", "Rust"),
        make_row("main.py", "py", "Python"),
        make_row("main.go", "go", "Go"),
    ]);
    let report = build_api_surface_report(dir.path(), &paths, &export, &default_limits()).unwrap();

    // Sum across by_language should equal top-level totals
    let lang_total: usize = report.by_language.values().map(|l| l.total_items).sum();
    let lang_public: usize = report.by_language.values().map(|l| l.public_items).sum();
    let lang_internal: usize = report.by_language.values().map(|l| l.internal_items).sum();

    assert_eq!(report.total_items, lang_total);
    assert_eq!(report.public_items, lang_public);
    assert_eq!(report.internal_items, lang_internal);
    assert_eq!(
        report.total_items,
        report.public_items + report.internal_items
    );
}

// ---------------------------------------------------------------------------
// Scenario: Rust pub(crate) / pub(super) visibility
// ---------------------------------------------------------------------------

#[test]
fn given_rust_pub_super_treated_as_public() {
    let code = "pub(super) fn sup() {}\npub(in crate::foo) fn scoped() {}\n";
    let (dir, paths) = write_temp_files(&[("lib.rs", code)]);
    let export = make_export(vec![make_row("lib.rs", ".", "Rust")]);
    let report = build_api_surface_report(dir.path(), &paths, &export, &default_limits()).unwrap();

    assert_eq!(report.public_items, 2);
}

// ---------------------------------------------------------------------------
// Scenario: Rust async/unsafe variants
// ---------------------------------------------------------------------------

#[test]
fn given_rust_async_and_unsafe_pub_detected() {
    let code = "pub async fn async_fn() {}\npub unsafe fn unsafe_fn() {}\npub unsafe trait UnsafeTrait {}\n";
    let (dir, paths) = write_temp_files(&[("lib.rs", code)]);
    let export = make_export(vec![make_row("lib.rs", ".", "Rust")]);
    let report = build_api_surface_report(dir.path(), &paths, &export, &default_limits()).unwrap();

    assert_eq!(report.public_items, 3);
}

// ---------------------------------------------------------------------------
// Scenario: Go method receivers
// ---------------------------------------------------------------------------

#[test]
fn given_go_method_receiver_detects_visibility_by_name() {
    let code = "func (s *Srv) Handle() {}\nfunc (s *Srv) handle() {}\n";
    let (dir, paths) = write_temp_files(&[("srv.go", code)]);
    let export = make_export(vec![make_row("srv.go", ".", "Go")]);
    let report = build_api_surface_report(dir.path(), &paths, &export, &default_limits()).unwrap();

    assert_eq!(report.public_items, 1);
    assert_eq!(report.internal_items, 1);
}

// ---------------------------------------------------------------------------
// Scenario: Empty source files
// ---------------------------------------------------------------------------

#[test]
fn given_empty_source_file_report_is_empty() {
    let (dir, paths) = write_temp_files(&[("lib.rs", "")]);
    let export = make_export(vec![make_row("lib.rs", ".", "Rust")]);
    let report = build_api_surface_report(dir.path(), &paths, &export, &default_limits()).unwrap();

    assert_eq!(report.total_items, 0);
}

// ---------------------------------------------------------------------------
// Scenario: Comments-only file
// ---------------------------------------------------------------------------

#[test]
fn given_file_with_only_comments_no_symbols() {
    let code = "// This is a comment\n// Another comment\n/* block */\n";
    let (dir, paths) = write_temp_files(&[("lib.rs", code)]);
    let export = make_export(vec![make_row("lib.rs", ".", "Rust")]);
    let report = build_api_surface_report(dir.path(), &paths, &export, &default_limits()).unwrap();

    assert_eq!(report.total_items, 0);
}
