//! Deep coverage tests for tokmd-analysis-api-surface.

use std::path::PathBuf;

use tokmd_analysis_api_surface::build_api_surface_report;
use tokmd_analysis_util::AnalysisLimits;
use tokmd_types::{ChildIncludeMode, ExportData, FileKind, FileRow};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn default_limits() -> AnalysisLimits {
    AnalysisLimits {
        max_files: None,
        max_bytes: None,
        max_file_bytes: None,
        max_commits: None,
        max_commit_files: None,
    }
}

fn make_row(path: &str, module: &str, lang: &str, kind: FileKind) -> FileRow {
    FileRow {
        path: path.to_string(),
        module: module.to_string(),
        lang: lang.to_string(),
        kind,
        code: 100,
        comments: 10,
        blanks: 5,
        lines: 115,
        bytes: 2000,
        tokens: 500,
    }
}

fn make_export(rows: Vec<FileRow>) -> ExportData {
    ExportData {
        rows,
        module_roots: vec![],
        module_depth: 1,
        children: ChildIncludeMode::ParentsOnly,
    }
}

fn write_file(dir: &std::path::Path, rel: &str, content: &str) -> PathBuf {
    let full = dir.join(rel);
    if let Some(parent) = full.parent() {
        std::fs::create_dir_all(parent).unwrap();
    }
    std::fs::write(&full, content).unwrap();
    PathBuf::from(rel)
}

// ---------------------------------------------------------------------------
// Empty / no files
// ---------------------------------------------------------------------------

#[test]
fn empty_files_empty_report() {
    let tmp = tempfile::tempdir().unwrap();
    let export = make_export(vec![]);
    let report = build_api_surface_report(tmp.path(), &[], &export, &default_limits()).unwrap();
    assert_eq!(report.total_items, 0);
    assert_eq!(report.public_items, 0);
    assert_eq!(report.internal_items, 0);
    assert_eq!(report.public_ratio, 0.0);
    assert_eq!(report.documented_ratio, 0.0);
    assert!(report.by_language.is_empty());
    assert!(report.by_module.is_empty());
    assert!(report.top_exporters.is_empty());
}

// ---------------------------------------------------------------------------
// Rust: public / internal detection
// ---------------------------------------------------------------------------

#[test]
fn rust_all_public() {
    let tmp = tempfile::tempdir().unwrap();
    let code = "pub fn foo() {}\npub struct Bar;\npub enum Baz {}\n";
    let rel = write_file(tmp.path(), "src/lib.rs", code);
    let rows = vec![make_row("src/lib.rs", "src", "Rust", FileKind::Parent)];
    let export = make_export(rows);

    let report = build_api_surface_report(tmp.path(), &[rel], &export, &default_limits()).unwrap();

    assert_eq!(report.total_items, 3);
    assert_eq!(report.public_items, 3);
    assert_eq!(report.internal_items, 0);
    assert!((report.public_ratio - 1.0).abs() < f64::EPSILON);
}

#[test]
fn rust_all_internal() {
    let tmp = tempfile::tempdir().unwrap();
    let code = "fn foo() {}\nstruct Bar;\nenum Baz {}\n";
    let rel = write_file(tmp.path(), "src/lib.rs", code);
    let rows = vec![make_row("src/lib.rs", "src", "Rust", FileKind::Parent)];
    let export = make_export(rows);

    let report = build_api_surface_report(tmp.path(), &[rel], &export, &default_limits()).unwrap();

    assert_eq!(report.total_items, 3);
    assert_eq!(report.public_items, 0);
    assert_eq!(report.internal_items, 3);
    assert!((report.public_ratio - 0.0).abs() < f64::EPSILON);
}

#[test]
fn rust_mixed_visibility() {
    let tmp = tempfile::tempdir().unwrap();
    let code = "pub fn public_fn() {}\nfn private_fn() {}\npub struct MyType;\n";
    let rel = write_file(tmp.path(), "src/lib.rs", code);
    let rows = vec![make_row("src/lib.rs", "src", "Rust", FileKind::Parent)];
    let export = make_export(rows);

    let report = build_api_surface_report(tmp.path(), &[rel], &export, &default_limits()).unwrap();

    assert_eq!(report.total_items, 3);
    assert_eq!(report.public_items, 2);
    assert_eq!(report.internal_items, 1);
}

#[test]
fn rust_documented_public() {
    let tmp = tempfile::tempdir().unwrap();
    let code = "/// Documented\npub fn documented() {}\npub fn undocumented() {}\n";
    let rel = write_file(tmp.path(), "src/lib.rs", code);
    let rows = vec![make_row("src/lib.rs", "src", "Rust", FileKind::Parent)];
    let export = make_export(rows);

    let report = build_api_surface_report(tmp.path(), &[rel], &export, &default_limits()).unwrap();

    assert_eq!(report.public_items, 2);
    // 1 documented out of 2 public -> 0.5
    assert!((report.documented_ratio - 0.5).abs() < f64::EPSILON);
}

// ---------------------------------------------------------------------------
// JavaScript / TypeScript
// ---------------------------------------------------------------------------

#[test]
fn js_export_detection() {
    let tmp = tempfile::tempdir().unwrap();
    let code = "export function handler() {}\nfunction internal() {}\n";
    let rel = write_file(tmp.path(), "src/index.js", code);
    let rows = vec![make_row(
        "src/index.js",
        "src",
        "JavaScript",
        FileKind::Parent,
    )];
    let export = make_export(rows);

    let report = build_api_surface_report(tmp.path(), &[rel], &export, &default_limits()).unwrap();

    assert_eq!(report.total_items, 2);
    assert_eq!(report.public_items, 1);
    assert_eq!(report.internal_items, 1);
}

#[test]
fn ts_multiple_exports() {
    let tmp = tempfile::tempdir().unwrap();
    let code = "export interface IFoo {}\nexport type Bar = string;\nexport class Baz {}\nconst internal = 1;\n";
    let rel = write_file(tmp.path(), "src/types.ts", code);
    let rows = vec![make_row(
        "src/types.ts",
        "src",
        "TypeScript",
        FileKind::Parent,
    )];
    let export = make_export(rows);

    let report = build_api_surface_report(tmp.path(), &[rel], &export, &default_limits()).unwrap();

    assert_eq!(report.public_items, 3);
    assert_eq!(report.internal_items, 1);
    assert_eq!(report.total_items, 4);
}

// ---------------------------------------------------------------------------
// Python
// ---------------------------------------------------------------------------

#[test]
fn python_public_private() {
    let tmp = tempfile::tempdir().unwrap();
    let code = "def public_func():\n    pass\n\ndef _private_func():\n    pass\n\nclass MyClass:\n    pass\n";
    let rel = write_file(tmp.path(), "lib/main.py", code);
    let rows = vec![make_row("lib/main.py", "lib", "Python", FileKind::Parent)];
    let export = make_export(rows);

    let report = build_api_surface_report(tmp.path(), &[rel], &export, &default_limits()).unwrap();

    assert_eq!(report.total_items, 3);
    assert_eq!(report.public_items, 2);
    assert_eq!(report.internal_items, 1);
}

#[test]
fn python_docstring_detection() {
    let tmp = tempfile::tempdir().unwrap();
    let code = "def documented():\n    \"\"\"Has docstring.\"\"\"\n    pass\n\ndef undocumented():\n    pass\n";
    let rel = write_file(tmp.path(), "lib/utils.py", code);
    let rows = vec![make_row("lib/utils.py", "lib", "Python", FileKind::Parent)];
    let export = make_export(rows);

    let report = build_api_surface_report(tmp.path(), &[rel], &export, &default_limits()).unwrap();

    assert_eq!(report.public_items, 2);
    // 1 out of 2 documented -> 0.5
    assert!((report.documented_ratio - 0.5).abs() < f64::EPSILON);
}

// ---------------------------------------------------------------------------
// Go
// ---------------------------------------------------------------------------

#[test]
fn go_capitalization_visibility() {
    let tmp = tempfile::tempdir().unwrap();
    let code = "func PublicFunc() {}\nfunc privateFunc() {}\ntype MyStruct struct {}\n";
    let rel = write_file(tmp.path(), "pkg/main.go", code);
    let rows = vec![make_row("pkg/main.go", "pkg", "Go", FileKind::Parent)];
    let export = make_export(rows);

    let report = build_api_surface_report(tmp.path(), &[rel], &export, &default_limits()).unwrap();

    assert_eq!(report.total_items, 3);
    assert_eq!(report.public_items, 2);
    assert_eq!(report.internal_items, 1);
}

// ---------------------------------------------------------------------------
// Java
// ---------------------------------------------------------------------------

#[test]
fn java_public_private() {
    let tmp = tempfile::tempdir().unwrap();
    let code = "public class MyClass {}\nprivate void helper() {}\n";
    let rel = write_file(tmp.path(), "src/MyClass.java", code);
    let rows = vec![make_row(
        "src/MyClass.java",
        "src",
        "Java",
        FileKind::Parent,
    )];
    let export = make_export(rows);

    let report = build_api_surface_report(tmp.path(), &[rel], &export, &default_limits()).unwrap();

    assert_eq!(report.total_items, 2);
    assert_eq!(report.public_items, 1);
    assert_eq!(report.internal_items, 1);
}

// ---------------------------------------------------------------------------
// Unsupported language is ignored
// ---------------------------------------------------------------------------

#[test]
fn unsupported_language_excluded() {
    let tmp = tempfile::tempdir().unwrap();
    let code = "# some markdown\n## heading\n";
    let rel = write_file(tmp.path(), "docs/readme.md", code);
    let rows = vec![make_row(
        "docs/readme.md",
        "docs",
        "Markdown",
        FileKind::Parent,
    )];
    let export = make_export(rows);

    let report = build_api_surface_report(tmp.path(), &[rel], &export, &default_limits()).unwrap();

    assert_eq!(report.total_items, 0);
}

// ---------------------------------------------------------------------------
// Child rows are skipped (only Parent kind is scanned)
// ---------------------------------------------------------------------------

#[test]
fn child_kind_rows_skipped() {
    let tmp = tempfile::tempdir().unwrap();
    let code = "pub fn child_fn() {}\n";
    let rel = write_file(tmp.path(), "src/child.rs", code);
    let rows = vec![make_row("src/child.rs", "src", "Rust", FileKind::Child)];
    let export = make_export(rows);

    let report = build_api_surface_report(tmp.path(), &[rel], &export, &default_limits()).unwrap();

    assert_eq!(report.total_items, 0);
}

// ---------------------------------------------------------------------------
// Per-language breakdown
// ---------------------------------------------------------------------------

#[test]
fn by_language_breakdown() {
    let tmp = tempfile::tempdir().unwrap();
    let rust_code = "pub fn rust_fn() {}\nfn internal_fn() {}\n";
    let js_code = "export function jsFunc() {}\n";
    let rust_rel = write_file(tmp.path(), "src/lib.rs", rust_code);
    let js_rel = write_file(tmp.path(), "src/index.js", js_code);
    let rows = vec![
        make_row("src/lib.rs", "src", "Rust", FileKind::Parent),
        make_row("src/index.js", "src", "JavaScript", FileKind::Parent),
    ];
    let export = make_export(rows);

    let report =
        build_api_surface_report(tmp.path(), &[rust_rel, js_rel], &export, &default_limits())
            .unwrap();

    assert!(report.by_language.contains_key("Rust"));
    assert!(report.by_language.contains_key("JavaScript"));

    let rust = &report.by_language["Rust"];
    assert_eq!(rust.total_items, 2);
    assert_eq!(rust.public_items, 1);
    assert_eq!(rust.internal_items, 1);

    let js = &report.by_language["JavaScript"];
    assert_eq!(js.total_items, 1);
    assert_eq!(js.public_items, 1);
}

// ---------------------------------------------------------------------------
// Per-module breakdown
// ---------------------------------------------------------------------------

#[test]
fn by_module_breakdown() {
    let tmp = tempfile::tempdir().unwrap();
    let code_a = "pub fn a() {}\n";
    let code_b = "pub fn b() {}\nfn c() {}\n";
    let rel_a = write_file(tmp.path(), "mod_a/lib.rs", code_a);
    let rel_b = write_file(tmp.path(), "mod_b/lib.rs", code_b);
    let rows = vec![
        make_row("mod_a/lib.rs", "mod_a", "Rust", FileKind::Parent),
        make_row("mod_b/lib.rs", "mod_b", "Rust", FileKind::Parent),
    ];
    let export = make_export(rows);

    let report =
        build_api_surface_report(tmp.path(), &[rel_a, rel_b], &export, &default_limits()).unwrap();

    assert_eq!(report.by_module.len(), 2);
    // Sorted by total_items descending
    assert_eq!(report.by_module[0].module, "mod_b");
    assert_eq!(report.by_module[0].total_items, 2);
    assert_eq!(report.by_module[1].module, "mod_a");
    assert_eq!(report.by_module[1].total_items, 1);
}

// ---------------------------------------------------------------------------
// Top exporters
// ---------------------------------------------------------------------------

#[test]
fn top_exporters_sorted_by_public_items() {
    let tmp = tempfile::tempdir().unwrap();
    let code_few = "pub fn one() {}\n";
    let code_many = "pub fn a() {}\npub fn b() {}\npub fn c() {}\n";
    let rel_few = write_file(tmp.path(), "src/few.rs", code_few);
    let rel_many = write_file(tmp.path(), "src/many.rs", code_many);
    let rows = vec![
        make_row("src/few.rs", "src", "Rust", FileKind::Parent),
        make_row("src/many.rs", "src", "Rust", FileKind::Parent),
    ];
    let export = make_export(rows);

    let report =
        build_api_surface_report(tmp.path(), &[rel_few, rel_many], &export, &default_limits())
            .unwrap();

    assert!(!report.top_exporters.is_empty());
    // First exporter should have more public items
    assert!(report.top_exporters[0].public_items >= report.top_exporters[1].public_items);
    assert_eq!(report.top_exporters[0].public_items, 3);
}

// ---------------------------------------------------------------------------
// Deterministic output
// ---------------------------------------------------------------------------

#[test]
fn deterministic_output() {
    let tmp = tempfile::tempdir().unwrap();
    let code = "pub fn foo() {}\nfn bar() {}\npub struct S;\n";
    let rel = write_file(tmp.path(), "src/lib.rs", code);
    let rows = vec![make_row("src/lib.rs", "src", "Rust", FileKind::Parent)];
    let export = make_export(rows);

    let r1 = build_api_surface_report(
        tmp.path(),
        std::slice::from_ref(&rel),
        &export,
        &default_limits(),
    )
    .unwrap();
    let r2 = build_api_surface_report(tmp.path(), &[rel], &export, &default_limits()).unwrap();

    assert_eq!(r1.total_items, r2.total_items);
    assert_eq!(r1.public_items, r2.public_items);
    assert_eq!(r1.internal_items, r2.internal_items);
    assert!((r1.public_ratio - r2.public_ratio).abs() < f64::EPSILON);
    assert!((r1.documented_ratio - r2.documented_ratio).abs() < f64::EPSILON);
}

// ---------------------------------------------------------------------------
// Public ratio calculations
// ---------------------------------------------------------------------------

#[test]
fn public_ratio_zero_when_no_public() {
    let tmp = tempfile::tempdir().unwrap();
    let code = "fn private_a() {}\nfn private_b() {}\n";
    let rel = write_file(tmp.path(), "src/lib.rs", code);
    let rows = vec![make_row("src/lib.rs", "src", "Rust", FileKind::Parent)];
    let export = make_export(rows);

    let report = build_api_surface_report(tmp.path(), &[rel], &export, &default_limits()).unwrap();

    assert!((report.public_ratio - 0.0).abs() < f64::EPSILON);
}

#[test]
fn public_ratio_one_when_all_public() {
    let tmp = tempfile::tempdir().unwrap();
    let code = "pub fn a() {}\npub fn b() {}\n";
    let rel = write_file(tmp.path(), "src/lib.rs", code);
    let rows = vec![make_row("src/lib.rs", "src", "Rust", FileKind::Parent)];
    let export = make_export(rows);

    let report = build_api_surface_report(tmp.path(), &[rel], &export, &default_limits()).unwrap();

    assert!((report.public_ratio - 1.0).abs() < f64::EPSILON);
}

#[test]
fn documented_ratio_zero_when_no_docs() {
    let tmp = tempfile::tempdir().unwrap();
    let code = "pub fn undoc() {}\n";
    let rel = write_file(tmp.path(), "src/lib.rs", code);
    let rows = vec![make_row("src/lib.rs", "src", "Rust", FileKind::Parent)];
    let export = make_export(rows);

    let report = build_api_surface_report(tmp.path(), &[rel], &export, &default_limits()).unwrap();

    assert!((report.documented_ratio - 0.0).abs() < f64::EPSILON);
}

#[test]
fn documented_ratio_one_when_all_documented() {
    let tmp = tempfile::tempdir().unwrap();
    let code = "/// doc\npub fn a() {}\n/// doc\npub fn b() {}\n";
    let rel = write_file(tmp.path(), "src/lib.rs", code);
    let rows = vec![make_row("src/lib.rs", "src", "Rust", FileKind::Parent)];
    let export = make_export(rows);

    let report = build_api_surface_report(tmp.path(), &[rel], &export, &default_limits()).unwrap();

    assert!((report.documented_ratio - 1.0).abs() < f64::EPSILON);
}

// ---------------------------------------------------------------------------
// File not found is gracefully skipped
// ---------------------------------------------------------------------------

#[test]
fn missing_file_skipped_gracefully() {
    let tmp = tempfile::tempdir().unwrap();
    let rel = PathBuf::from("src/nonexistent.rs");
    let rows = vec![make_row(
        "src/nonexistent.rs",
        "src",
        "Rust",
        FileKind::Parent,
    )];
    let export = make_export(rows);

    let report = build_api_surface_report(tmp.path(), &[rel], &export, &default_limits()).unwrap();

    assert_eq!(report.total_items, 0);
}

// ---------------------------------------------------------------------------
// Multiple files across languages
// ---------------------------------------------------------------------------

#[test]
fn multi_language_report() {
    let tmp = tempfile::tempdir().unwrap();
    let rust_code = "pub fn rust_pub() {}\n";
    let py_code = "def py_pub():\n    pass\ndef _py_priv():\n    pass\n";
    let go_code = "func GoPublic() {}\nfunc goPrivate() {}\n";

    let rust_rel = write_file(tmp.path(), "src/lib.rs", rust_code);
    let py_rel = write_file(tmp.path(), "lib/main.py", py_code);
    let go_rel = write_file(tmp.path(), "pkg/main.go", go_code);

    let rows = vec![
        make_row("src/lib.rs", "src", "Rust", FileKind::Parent),
        make_row("lib/main.py", "lib", "Python", FileKind::Parent),
        make_row("pkg/main.go", "pkg", "Go", FileKind::Parent),
    ];
    let export = make_export(rows);

    let report = build_api_surface_report(
        tmp.path(),
        &[rust_rel, py_rel, go_rel],
        &export,
        &default_limits(),
    )
    .unwrap();

    assert_eq!(report.total_items, 5);
    assert_eq!(report.public_items, 3);
    assert_eq!(report.internal_items, 2);
    assert_eq!(report.by_language.len(), 3);
}

// ---------------------------------------------------------------------------
// LangApiSurface public_ratio
// ---------------------------------------------------------------------------

#[test]
fn lang_surface_public_ratio_accurate() {
    let tmp = tempfile::tempdir().unwrap();
    let code = "pub fn a() {}\npub fn b() {}\nfn c() {}\nfn d() {}\n";
    let rel = write_file(tmp.path(), "src/lib.rs", code);
    let rows = vec![make_row("src/lib.rs", "src", "Rust", FileKind::Parent)];
    let export = make_export(rows);

    let report = build_api_surface_report(tmp.path(), &[rel], &export, &default_limits()).unwrap();

    let rust = &report.by_language["Rust"];
    // 2 public out of 4 total = 0.5
    assert!((rust.public_ratio - 0.5).abs() < f64::EPSILON);
}

// ---------------------------------------------------------------------------
// ModuleApiRow public_ratio
// ---------------------------------------------------------------------------

#[test]
fn module_row_public_ratio() {
    let tmp = tempfile::tempdir().unwrap();
    let code = "pub fn a() {}\nfn b() {}\nfn c() {}\n";
    let rel = write_file(tmp.path(), "src/lib.rs", code);
    let rows = vec![make_row("src/lib.rs", "src", "Rust", FileKind::Parent)];
    let export = make_export(rows);

    let report = build_api_surface_report(tmp.path(), &[rel], &export, &default_limits()).unwrap();

    assert_eq!(report.by_module.len(), 1);
    let m = &report.by_module[0];
    // 1 public out of 3 total ≈ 0.3333
    assert!((m.public_ratio - 0.3333).abs() < 0.001);
}

// ---------------------------------------------------------------------------
// Files with no symbols are excluded from report
// ---------------------------------------------------------------------------

#[test]
fn file_with_no_symbols_excluded() {
    let tmp = tempfile::tempdir().unwrap();
    let code = "// just a comment\n\n";
    let rel = write_file(tmp.path(), "src/empty.rs", code);
    let rows = vec![make_row("src/empty.rs", "src", "Rust", FileKind::Parent)];
    let export = make_export(rows);

    let report = build_api_surface_report(tmp.path(), &[rel], &export, &default_limits()).unwrap();

    assert_eq!(report.total_items, 0);
    assert!(report.top_exporters.is_empty());
}

// ---------------------------------------------------------------------------
// Limits: max_bytes stops scanning
// ---------------------------------------------------------------------------

#[test]
fn max_bytes_limit_stops_scanning() {
    let tmp = tempfile::tempdir().unwrap();
    // Write enough content to exceed a small byte limit
    let code = "pub fn a() {}\npub fn b() {}\n";
    let rel_a = write_file(tmp.path(), "src/a.rs", code);
    let rel_b = write_file(tmp.path(), "src/b.rs", code);
    let rows = vec![
        make_row("src/a.rs", "src", "Rust", FileKind::Parent),
        make_row("src/b.rs", "src", "Rust", FileKind::Parent),
    ];
    let export = make_export(rows);

    let limits = AnalysisLimits {
        max_bytes: Some(1), // Only 1 byte budget -> stop after first file
        ..default_limits()
    };

    let report = build_api_surface_report(tmp.path(), &[rel_a, rel_b], &export, &limits).unwrap();

    // Should have processed at most the first file
    assert!(report.total_items <= 2);
}

// ---------------------------------------------------------------------------
// Rust pub(crate) treated as public for API surface
// ---------------------------------------------------------------------------

#[test]
fn rust_pub_crate_counted_as_public() {
    let tmp = tempfile::tempdir().unwrap();
    let code = "pub(crate) fn internal_but_pub() {}\nfn truly_private() {}\n";
    let rel = write_file(tmp.path(), "src/lib.rs", code);
    let rows = vec![make_row("src/lib.rs", "src", "Rust", FileKind::Parent)];
    let export = make_export(rows);

    let report = build_api_surface_report(tmp.path(), &[rel], &export, &default_limits()).unwrap();

    assert_eq!(report.total_items, 2);
    assert_eq!(report.public_items, 1);
    assert_eq!(report.internal_items, 1);
}

// ---------------------------------------------------------------------------
// Documented ratio is 0 when no public items exist
// ---------------------------------------------------------------------------

#[test]
fn documented_ratio_zero_when_no_public_items() {
    let tmp = tempfile::tempdir().unwrap();
    let code = "/// documented\nfn private_fn() {}\n";
    let rel = write_file(tmp.path(), "src/lib.rs", code);
    let rows = vec![make_row("src/lib.rs", "src", "Rust", FileKind::Parent)];
    let export = make_export(rows);

    let report = build_api_surface_report(tmp.path(), &[rel], &export, &default_limits()).unwrap();

    assert_eq!(report.public_items, 0);
    assert!((report.documented_ratio - 0.0).abs() < f64::EPSILON);
}

// ---------------------------------------------------------------------------
// Export abstract class in TypeScript
// ---------------------------------------------------------------------------

#[test]
fn ts_export_abstract_class() {
    let tmp = tempfile::tempdir().unwrap();
    let code = "export abstract class Base {}\n";
    let rel = write_file(tmp.path(), "src/base.ts", code);
    let rows = vec![make_row(
        "src/base.ts",
        "src",
        "TypeScript",
        FileKind::Parent,
    )];
    let export = make_export(rows);

    let report = build_api_surface_report(tmp.path(), &[rel], &export, &default_limits()).unwrap();

    assert_eq!(report.public_items, 1);
    assert_eq!(report.total_items, 1);
}
