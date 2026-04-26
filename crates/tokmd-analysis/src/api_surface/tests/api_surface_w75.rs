//! W75 tests for analysis API surface module.
//!
//! Covers Rust API surface detection, pub fn/struct/enum/trait counting,
//! and multi-language file analysis.

use std::fs;
use std::path::PathBuf;

use crate::api_surface::build_api_surface_report;
use tokmd_analysis_types::AnalysisLimits;
use tokmd_types::{ChildIncludeMode, ExportData, FileKind, FileRow};

// ── Helpers ─────────────────────────────────────────────────────────

fn make_row(path: &str, module: &str, lang: &str) -> FileRow {
    FileRow {
        path: path.to_string(),
        module: module.to_string(),
        lang: lang.to_string(),
        kind: FileKind::Parent,
        code: 20,
        comments: 5,
        blanks: 2,
        lines: 27,
        bytes: 500,
        tokens: 100,
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

fn write_temp(files: &[(&str, &str)]) -> (tempfile::TempDir, Vec<PathBuf>) {
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

// ═══════════════════════════════════════════════════════════════════
// § 1. Rust pub fn counting
// ═══════════════════════════════════════════════════════════════════

#[test]
fn rust_counts_pub_fns() {
    let code = "pub fn alpha() {}\npub fn beta() {}\nfn private() {}\n";
    let (dir, paths) = write_temp(&[("lib.rs", code)]);
    let export = make_export(vec![make_row("lib.rs", "src", "Rust")]);
    let r = build_api_surface_report(dir.path(), &paths, &export, &default_limits()).unwrap();
    assert_eq!(r.public_items, 2);
    assert_eq!(r.internal_items, 1);
    assert_eq!(r.total_items, 3);
}

#[test]
fn rust_pub_crate_counted_as_public() {
    let code = "pub(crate) fn scoped() {}\npub fn open() {}\n";
    let (dir, paths) = write_temp(&[("lib.rs", code)]);
    let export = make_export(vec![make_row("lib.rs", "src", "Rust")]);
    let r = build_api_surface_report(dir.path(), &paths, &export, &default_limits()).unwrap();
    assert_eq!(r.public_items, 2);
}

// ═══════════════════════════════════════════════════════════════════
// § 2. Struct/enum/trait counting
// ═══════════════════════════════════════════════════════════════════

#[test]
fn rust_counts_pub_struct_enum_trait() {
    let code = "pub struct Foo;\npub enum Bar { A, B }\npub trait Baz {}\n";
    let (dir, paths) = write_temp(&[("types.rs", code)]);
    let export = make_export(vec![make_row("types.rs", "src", "Rust")]);
    let r = build_api_surface_report(dir.path(), &paths, &export, &default_limits()).unwrap();
    assert_eq!(r.public_items, 3);
    assert_eq!(r.internal_items, 0);
}

#[test]
fn rust_internal_struct_enum_trait() {
    let code = "struct Private;\nenum Internal { X }\ntrait Hidden {}\n";
    let (dir, paths) = write_temp(&[("internal.rs", code)]);
    let export = make_export(vec![make_row("internal.rs", "src", "Rust")]);
    let r = build_api_surface_report(dir.path(), &paths, &export, &default_limits()).unwrap();
    assert_eq!(r.public_items, 0);
    assert_eq!(r.internal_items, 3);
}

#[test]
fn rust_mixed_pub_and_private() {
    let code = "\
pub struct Config;
pub fn init() {}
fn helper() {}
struct State;
pub enum Mode { A, B }
trait Internal {}
";
    let (dir, paths) = write_temp(&[("lib.rs", code)]);
    let export = make_export(vec![make_row("lib.rs", "src", "Rust")]);
    let r = build_api_surface_report(dir.path(), &paths, &export, &default_limits()).unwrap();
    assert_eq!(r.public_items, 3);
    assert_eq!(r.internal_items, 3);
    assert_eq!(r.total_items, 6);
}

// ═══════════════════════════════════════════════════════════════════
// § 3. Various language files
// ═══════════════════════════════════════════════════════════════════

#[test]
fn python_public_and_private() {
    let code = "def public_fn():\n    pass\ndef _private():\n    pass\nclass Widget:\n    pass\n";
    let (dir, paths) = write_temp(&[("app.py", code)]);
    let export = make_export(vec![make_row("app.py", "src", "Python")]);
    let r = build_api_surface_report(dir.path(), &paths, &export, &default_limits()).unwrap();
    assert_eq!(r.public_items, 2); // public_fn + Widget
    assert_eq!(r.internal_items, 1); // _private
}

#[test]
fn javascript_exports_and_internals() {
    let code = "export function api() {}\nexport class Service {}\nfunction helper() {}\n";
    let (dir, paths) = write_temp(&[("index.js", code)]);
    let export = make_export(vec![make_row("index.js", "src", "JavaScript")]);
    let r = build_api_surface_report(dir.path(), &paths, &export, &default_limits()).unwrap();
    assert_eq!(r.public_items, 2);
    assert_eq!(r.internal_items, 1);
}

#[test]
fn go_uppercase_is_public() {
    let code = "func PublicFunc() {}\nfunc privateFunc() {}\ntype Config struct {}\n";
    let (dir, paths) = write_temp(&[("main.go", code)]);
    let export = make_export(vec![make_row("main.go", "src", "Go")]);
    let r = build_api_surface_report(dir.path(), &paths, &export, &default_limits()).unwrap();
    assert_eq!(r.public_items, 2); // PublicFunc + Config
    assert_eq!(r.internal_items, 1); // privateFunc
}

#[test]
fn unsupported_language_ignored() {
    let code = "some random content\n";
    let (dir, paths) = write_temp(&[("data.csv", code)]);
    let export = make_export(vec![make_row("data.csv", "data", "CSV")]);
    let r = build_api_surface_report(dir.path(), &paths, &export, &default_limits()).unwrap();
    assert_eq!(r.total_items, 0);
    assert_eq!(r.public_items, 0);
}

#[test]
fn multi_language_aggregation() {
    let rust_code = "pub fn greet() {}\nfn helper() {}\n";
    let py_code = "def serve():\n    pass\ndef _internal():\n    pass\n";
    let (dir, paths) = write_temp(&[("lib.rs", rust_code), ("app.py", py_code)]);
    let export = make_export(vec![
        make_row("lib.rs", "core", "Rust"),
        make_row("app.py", "api", "Python"),
    ]);
    let r = build_api_surface_report(dir.path(), &paths, &export, &default_limits()).unwrap();
    assert_eq!(r.total_items, 4);
    assert_eq!(r.public_items, 2);
    assert_eq!(r.internal_items, 2);
    assert_eq!(r.by_language.len(), 2);
    assert!(r.by_language.contains_key("Rust"));
    assert!(r.by_language.contains_key("Python"));
}
