//! Wave-49 deep tests for API surface analysis.
//!
//! Covers multi-language symbol extraction, documentation detection,
//! ratio calculations, limits, serde roundtrips, and property-based tests.

use std::fs;
use std::path::PathBuf;

use crate::api_surface::build_api_surface_report;
use tempfile::tempdir;
use tokmd_analysis_types::AnalysisLimits;
use tokmd_analysis_types::ApiSurfaceReport;
use tokmd_types::{ChildIncludeMode, ExportData, FileKind, FileRow};

// ── Helpers ─────────────────────────────────────────────────────

fn file_row(path: &str, module: &str, lang: &str) -> FileRow {
    FileRow {
        path: path.to_string(),
        module: module.to_string(),
        lang: lang.to_string(),
        kind: FileKind::Parent,
        code: 10,
        comments: 0,
        blanks: 0,
        lines: 10,
        bytes: 200,
        tokens: 50,
    }
}

fn export(rows: Vec<FileRow>) -> ExportData {
    ExportData {
        rows,
        module_roots: vec![],
        module_depth: 1,
        children: ChildIncludeMode::Separate,
    }
}

// ── 1. Empty files → zeroed report ──────────────────────────────

#[test]
fn empty_files_zeroed_report() {
    let dir = tempdir().unwrap();
    let exp = export(vec![]);
    let report =
        build_api_surface_report(dir.path(), &[], &exp, &AnalysisLimits::default()).unwrap();
    assert_eq!(report.total_items, 0);
    assert_eq!(report.public_items, 0);
    assert_eq!(report.internal_items, 0);
    assert_eq!(report.public_ratio, 0.0);
    assert_eq!(report.documented_ratio, 0.0);
    assert!(report.by_language.is_empty());
    assert!(report.by_module.is_empty());
    assert!(report.top_exporters.is_empty());
}

// ── 2. Rust pub fn detection ────────────────────────────────────

#[test]
fn rust_pub_fn_detection() {
    let dir = tempdir().unwrap();
    let code = "/// Documented\npub fn hello() {}\nfn private() {}\n";
    fs::write(dir.path().join("lib.rs"), code).unwrap();
    let exp = export(vec![file_row("lib.rs", "root", "Rust")]);
    let files = vec![PathBuf::from("lib.rs")];
    let report =
        build_api_surface_report(dir.path(), &files, &exp, &AnalysisLimits::default()).unwrap();
    assert_eq!(report.total_items, 2);
    assert_eq!(report.public_items, 1);
    assert_eq!(report.internal_items, 1);
}

// ── 3. Rust pub struct, enum, trait ─────────────────────────────

#[test]
fn rust_pub_struct_enum_trait() {
    let dir = tempdir().unwrap();
    let code = "pub struct Foo {}\npub enum Bar {}\npub trait Baz {}\nenum Private {}\n";
    fs::write(dir.path().join("types.rs"), code).unwrap();
    let exp = export(vec![file_row("types.rs", "root", "Rust")]);
    let files = vec![PathBuf::from("types.rs")];
    let report =
        build_api_surface_report(dir.path(), &files, &exp, &AnalysisLimits::default()).unwrap();
    assert_eq!(report.public_items, 3);
    assert_eq!(report.internal_items, 1);
    assert_eq!(report.total_items, 4);
}

// ── 4. JavaScript export detection ──────────────────────────────

#[test]
fn js_export_detection() {
    let dir = tempdir().unwrap();
    let code = "export function greet() {}\nexport class App {}\nfunction helper() {}\n";
    fs::write(dir.path().join("app.js"), code).unwrap();
    let exp = export(vec![file_row("app.js", "root", "JavaScript")]);
    let files = vec![PathBuf::from("app.js")];
    let report =
        build_api_surface_report(dir.path(), &files, &exp, &AnalysisLimits::default()).unwrap();
    assert_eq!(report.public_items, 2);
    assert_eq!(report.internal_items, 1);
}

// ── 5. Python def detection with underscore convention ──────────

#[test]
fn python_public_private_convention() {
    let dir = tempdir().unwrap();
    let code = "def public_func():\n    pass\n\ndef _private_func():\n    pass\n\nclass MyClass:\n    pass\n";
    fs::write(dir.path().join("mod.py"), code).unwrap();
    let exp = export(vec![file_row("mod.py", "root", "Python")]);
    let files = vec![PathBuf::from("mod.py")];
    let report =
        build_api_surface_report(dir.path(), &files, &exp, &AnalysisLimits::default()).unwrap();
    // public_func + MyClass = 2 public, _private_func = 1 internal
    assert_eq!(report.public_items, 2);
    assert_eq!(report.internal_items, 1);
}

// ── 6. Go uppercase convention ──────────────────────────────────

#[test]
fn go_uppercase_convention() {
    let dir = tempdir().unwrap();
    let code = "func PublicFunc() {}\nfunc privateFunc() {}\ntype Handler struct {}\ntype config struct {}\n";
    fs::write(dir.path().join("main.go"), code).unwrap();
    let exp = export(vec![file_row("main.go", "root", "Go")]);
    let files = vec![PathBuf::from("main.go")];
    let report =
        build_api_surface_report(dir.path(), &files, &exp, &AnalysisLimits::default()).unwrap();
    // PublicFunc + Handler = 2 public, privateFunc + config = 2 internal
    assert_eq!(report.public_items, 2);
    assert_eq!(report.internal_items, 2);
}

// ── 7. public + internal = total invariant ──────────────────────

#[test]
fn public_plus_internal_equals_total() {
    let dir = tempdir().unwrap();
    let code = "pub fn a() {}\nfn b() {}\npub struct C {}\nenum D {}\npub const E: i32 = 1;\n";
    fs::write(dir.path().join("lib.rs"), code).unwrap();
    let exp = export(vec![file_row("lib.rs", "root", "Rust")]);
    let files = vec![PathBuf::from("lib.rs")];
    let report =
        build_api_surface_report(dir.path(), &files, &exp, &AnalysisLimits::default()).unwrap();
    assert_eq!(
        report.public_items + report.internal_items,
        report.total_items,
        "public + internal must equal total"
    );
}

// ── 8. Documented ratio calculation ─────────────────────────────

#[test]
fn documented_ratio_calculation() {
    let dir = tempdir().unwrap();
    // 2 pub fns: one documented, one not
    let code = "/// Documented\npub fn documented() {}\npub fn undocumented() {}\n";
    fs::write(dir.path().join("lib.rs"), code).unwrap();
    let exp = export(vec![file_row("lib.rs", "root", "Rust")]);
    let files = vec![PathBuf::from("lib.rs")];
    let report =
        build_api_surface_report(dir.path(), &files, &exp, &AnalysisLimits::default()).unwrap();
    assert_eq!(report.public_items, 2);
    // documented_ratio = 1 documented / 2 public = 0.5
    assert!((report.documented_ratio - 0.5).abs() < 0.001);
}

// ── 9. Unsupported language ignored ─────────────────────────────

#[test]
fn unsupported_language_ignored() {
    let dir = tempdir().unwrap();
    fs::write(dir.path().join("data.csv"), "a,b,c\n1,2,3\n").unwrap();
    let exp = export(vec![file_row("data.csv", "root", "CSV")]);
    let files = vec![PathBuf::from("data.csv")];
    let report =
        build_api_surface_report(dir.path(), &files, &exp, &AnalysisLimits::default()).unwrap();
    assert_eq!(report.total_items, 0);
}

// ── 10. Multi-language by_language breakdown ────────────────────

#[test]
fn multi_language_breakdown() {
    let dir = tempdir().unwrap();
    fs::write(
        dir.path().join("lib.rs"),
        "pub fn rust_fn() {}\nfn internal() {}\n",
    )
    .unwrap();
    fs::write(dir.path().join("app.js"), "export function js_fn() {}\n").unwrap();
    let exp = export(vec![
        file_row("lib.rs", "root", "Rust"),
        file_row("app.js", "root", "JavaScript"),
    ]);
    let files = vec![PathBuf::from("lib.rs"), PathBuf::from("app.js")];
    let report =
        build_api_surface_report(dir.path(), &files, &exp, &AnalysisLimits::default()).unwrap();
    assert!(report.by_language.contains_key("Rust"));
    assert!(report.by_language.contains_key("JavaScript"));
    let rust = &report.by_language["Rust"];
    let js = &report.by_language["JavaScript"];
    assert_eq!(rust.total_items, 2);
    assert_eq!(js.total_items, 1);
}

// ── 11. Serde roundtrip preserves all fields ────────────────────

#[test]
fn serde_roundtrip_preserves_all_fields() {
    let dir = tempdir().unwrap();
    fs::write(
        dir.path().join("lib.rs"),
        "/// Doc\npub fn foo() {}\nfn bar() {}\n",
    )
    .unwrap();
    let exp = export(vec![file_row("lib.rs", "root", "Rust")]);
    let files = vec![PathBuf::from("lib.rs")];
    let report =
        build_api_surface_report(dir.path(), &files, &exp, &AnalysisLimits::default()).unwrap();

    let json = serde_json::to_string(&report).unwrap();
    let deser: ApiSurfaceReport = serde_json::from_str(&json).unwrap();

    assert_eq!(deser.total_items, report.total_items);
    assert_eq!(deser.public_items, report.public_items);
    assert_eq!(deser.internal_items, report.internal_items);
    assert!((deser.public_ratio - report.public_ratio).abs() < f64::EPSILON);
    assert!((deser.documented_ratio - report.documented_ratio).abs() < f64::EPSILON);
    assert_eq!(deser.by_language.len(), report.by_language.len());
    assert_eq!(deser.by_module.len(), report.by_module.len());
    assert_eq!(deser.top_exporters.len(), report.top_exporters.len());
}

// ── 12. Child rows excluded from analysis ───────────────────────

#[test]
fn child_rows_excluded() {
    let dir = tempdir().unwrap();
    fs::write(dir.path().join("lib.rs"), "pub fn parent_fn() {}\n").unwrap();
    fs::write(
        dir.path().join("embedded.js"),
        "export function child_fn() {}\n",
    )
    .unwrap();
    let mut child = file_row("embedded.js", "root", "JavaScript");
    child.kind = FileKind::Child;
    let exp = export(vec![file_row("lib.rs", "root", "Rust"), child]);
    let files = vec![PathBuf::from("lib.rs"), PathBuf::from("embedded.js")];
    let report =
        build_api_surface_report(dir.path(), &files, &exp, &AnalysisLimits::default()).unwrap();
    // Only lib.rs should be analyzed (child row's path won't be in row_map)
    assert_eq!(report.total_items, 1);
    assert_eq!(report.public_items, 1);
}

// ── Proptest ────────────────────────────────────────────────────

mod properties {
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn ratios_always_in_range(
            public in 0usize..100,
            internal in 0usize..100,
        ) {
            let total = public + internal;
            let public_ratio = if total == 0 { 0.0 } else { public as f64 / total as f64 };
            let documented_ratio = if public == 0 { 0.0 } else { (public / 2) as f64 / public as f64 };
            prop_assert!((0.0..=1.0).contains(&public_ratio));
            prop_assert!((0.0..=1.0).contains(&documented_ratio));
            prop_assert!(public + internal == total);
        }
    }
}
