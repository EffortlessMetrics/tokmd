//! Deep tests for `tokmd-analysis-api-surface` (wave 39).
//!
//! Exercises symbol extraction across Rust, Python, JavaScript/TypeScript,
//! documentation detection, ratio calculations, multi-line signatures,
//! and end-to-end `build_api_surface_report` integration.

use std::fs;
use std::path::PathBuf;

use tokmd_analysis_api_surface::build_api_surface_report;
use tokmd_analysis_types::AnalysisLimits;
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

/// Helper: build report from a single file with given lang.
fn report_for(path: &str, lang: &str, content: &str) -> tokmd_analysis_types::ApiSurfaceReport {
    let (dir, paths) = write_temp_files(&[(path, content)]);
    let export = make_export(vec![make_row(path, "root", lang)]);
    build_api_surface_report(dir.path(), &paths, &export, &default_limits()).unwrap()
}

// ── Rust symbol extraction ──────────────────────────────────────

#[test]
fn rust_pub_fn_detected() {
    let r = report_for("lib.rs", "Rust", "pub fn hello() {}\n");
    assert_eq!(r.public_items, 1);
    assert_eq!(r.total_items, 1);
}

#[test]
fn rust_pub_struct_enum_trait() {
    let code = "pub struct S;\npub enum E {}\npub trait T {}\n";
    let r = report_for("lib.rs", "Rust", code);
    assert_eq!(r.public_items, 3);
    assert_eq!(r.internal_items, 0);
}

#[test]
fn rust_private_fn_is_internal() {
    let r = report_for("lib.rs", "Rust", "fn private() {}\n");
    assert_eq!(r.internal_items, 1);
    assert_eq!(r.public_items, 0);
}

#[test]
fn rust_pub_crate_counted_as_public() {
    let r = report_for("lib.rs", "Rust", "pub(crate) fn internal_api() {}\n");
    assert_eq!(r.public_items, 1);
}

#[test]
fn rust_pub_super_counted_as_public() {
    let r = report_for("lib.rs", "Rust", "pub(super) struct S;\n");
    assert_eq!(r.public_items, 1);
}

#[test]
fn rust_impl_block_items_not_counted_as_top_level() {
    // Lines starting with `impl` are not items the extractor picks up
    let code = "impl Foo {\n    pub fn bar() {}\n}\n";
    let r = report_for("lib.rs", "Rust", code);
    // The indented `pub fn bar()` IS picked up (trimmed)
    assert_eq!(r.public_items, 1);
}

#[test]
fn rust_async_unsafe_pub_fn() {
    let code = "pub async fn a() {}\npub unsafe fn b() {}\n";
    let r = report_for("lib.rs", "Rust", code);
    assert_eq!(r.public_items, 2);
}

#[test]
fn rust_pub_const_static_mod_type() {
    let code = "pub const C: u8 = 1;\npub static S: u8 = 2;\npub mod m;\npub type T = u8;\n";
    let r = report_for("lib.rs", "Rust", code);
    assert_eq!(r.public_items, 4);
}

#[test]
fn rust_mixed_pub_and_private() {
    let code = "pub fn a() {}\nfn b() {}\npub struct S;\nstruct P;\n";
    let r = report_for("lib.rs", "Rust", code);
    assert_eq!(r.public_items, 2);
    assert_eq!(r.internal_items, 2);
    assert_eq!(r.total_items, 4);
}

#[test]
fn rust_doc_comment_triple_slash() {
    let code = "/// Documented\npub fn documented() {}\npub fn undocumented() {}\n";
    let r = report_for("lib.rs", "Rust", code);
    assert_eq!(r.public_items, 2);
    // 1 of 2 public items documented → 0.5
    assert!((r.documented_ratio - 0.5).abs() < 0.01);
}

#[test]
fn rust_doc_attribute() {
    let code = "#[doc = \"documented\"]\npub fn foo() {}\n";
    let r = report_for("lib.rs", "Rust", code);
    assert_eq!(r.documented_ratio, 1.0);
}

#[test]
fn rust_inner_doc_comment() {
    let code = "//! Module doc\npub fn foo() {}\n";
    let r = report_for("lib.rs", "Rust", code);
    assert_eq!(r.documented_ratio, 1.0);
}

#[test]
fn rust_multiline_fn_signature() {
    // Multi-line fn sig — only the first line has `pub fn`
    let code = "pub fn complex(\n    x: u32,\n    y: u32,\n) -> u32 {\n    x + y\n}\n";
    let r = report_for("lib.rs", "Rust", code);
    assert_eq!(r.public_items, 1);
}

// ── Python symbol extraction ────────────────────────────────────

#[test]
fn python_def_public() {
    let r = report_for("main.py", "Python", "def hello():\n    pass\n");
    assert_eq!(r.public_items, 1);
}

#[test]
fn python_def_private_underscore() {
    let r = report_for("main.py", "Python", "def _private():\n    pass\n");
    assert_eq!(r.internal_items, 1);
    assert_eq!(r.public_items, 0);
}

#[test]
fn python_class_public() {
    let r = report_for("main.py", "Python", "class MyClass:\n    pass\n");
    assert_eq!(r.public_items, 1);
}

#[test]
fn python_class_private() {
    let r = report_for("main.py", "Python", "class _Internal:\n    pass\n");
    assert_eq!(r.internal_items, 1);
}

#[test]
fn python_async_def() {
    let r = report_for("main.py", "Python", "async def fetch():\n    pass\n");
    assert_eq!(r.public_items, 1);
}

#[test]
fn python_docstring_detection() {
    let code = "def documented():\n    \"\"\"This is documented.\"\"\"\n    pass\n";
    let r = report_for("main.py", "Python", code);
    assert_eq!(r.documented_ratio, 1.0);
}

#[test]
fn python_single_quote_docstring() {
    let code = "def documented():\n    '''Single quote docstring.'''\n    pass\n";
    let r = report_for("main.py", "Python", code);
    assert_eq!(r.documented_ratio, 1.0);
}

#[test]
fn python_indented_method_not_counted() {
    let code = "class Foo:\n    def method(self):\n        pass\n";
    let r = report_for("main.py", "Python", code);
    // Only top-level class is counted
    assert_eq!(r.total_items, 1);
}

// ── JavaScript/TypeScript symbol extraction ─────────────────────

#[test]
fn js_export_function() {
    let r = report_for("index.js", "JavaScript", "export function foo() {}\n");
    assert_eq!(r.public_items, 1);
}

#[test]
fn js_export_class() {
    let r = report_for("index.js", "JavaScript", "export class MyClass {}\n");
    assert_eq!(r.public_items, 1);
}

#[test]
fn js_export_const() {
    let r = report_for("index.js", "JavaScript", "export const X = 1;\n");
    assert_eq!(r.public_items, 1);
}

#[test]
fn js_export_default() {
    let r = report_for(
        "index.js",
        "JavaScript",
        "export default function main() {}\n",
    );
    assert_eq!(r.public_items, 1);
}

#[test]
fn js_internal_function() {
    let r = report_for("index.js", "JavaScript", "function internal() {}\n");
    assert_eq!(r.internal_items, 1);
    assert_eq!(r.public_items, 0);
}

#[test]
fn ts_export_interface() {
    let r = report_for("types.ts", "TypeScript", "export interface IFoo {}\n");
    assert_eq!(r.public_items, 1);
}

#[test]
fn ts_export_type_and_enum() {
    let code = "export type Bar = string;\nexport enum Dir { Up, Down }\n";
    let r = report_for("types.ts", "TypeScript", code);
    assert_eq!(r.public_items, 2);
}

#[test]
fn ts_export_abstract_class() {
    let r = report_for("base.ts", "TypeScript", "export abstract class Base {}\n");
    assert_eq!(r.public_items, 1);
}

#[test]
fn js_documented_with_jsdoc() {
    let code = "/** JSDoc */\nexport function foo() {}\n";
    let r = report_for("index.js", "JavaScript", code);
    assert_eq!(r.documented_ratio, 1.0);
}

// ── Ratio calculations ─────────────────────────────────────────

#[test]
fn public_ratio_zero_when_all_internal() {
    let code = "fn a() {}\nfn b() {}\n";
    let r = report_for("lib.rs", "Rust", code);
    assert_eq!(r.public_ratio, 0.0);
    assert_eq!(r.total_items, 2);
}

#[test]
fn public_ratio_one_when_all_public() {
    let code = "pub fn a() {}\npub fn b() {}\n";
    let r = report_for("lib.rs", "Rust", code);
    assert_eq!(r.public_ratio, 1.0);
}

#[test]
fn documented_ratio_zero_when_no_docs() {
    let code = "pub fn a() {}\npub fn b() {}\n";
    let r = report_for("lib.rs", "Rust", code);
    assert_eq!(r.documented_ratio, 0.0);
}

#[test]
fn documented_ratio_all_documented() {
    let code = "/// doc\npub fn a() {}\n/// doc\npub fn b() {}\n";
    let r = report_for("lib.rs", "Rust", code);
    assert_eq!(r.documented_ratio, 1.0);
}

// ── Multi-file integration via build_api_surface_report ─────────

#[test]
fn multi_file_multi_lang_report() {
    let rust_code = "pub fn r_pub() {}\nfn r_priv() {}\n";
    let py_code = "def py_pub():\n    pass\ndef _py_priv():\n    pass\n";
    let files = [("src/lib.rs", rust_code), ("src/main.py", py_code)];
    let (dir, paths) = write_temp_files(&files);
    let export = make_export(vec![
        make_row("src/lib.rs", "src", "Rust"),
        make_row("src/main.py", "src", "Python"),
    ]);
    let r = build_api_surface_report(dir.path(), &paths, &export, &default_limits()).unwrap();

    assert_eq!(r.total_items, 4);
    assert_eq!(r.public_items, 2);
    assert_eq!(r.internal_items, 2);
    assert!(r.by_language.contains_key("Rust"));
    assert!(r.by_language.contains_key("Python"));
}

#[test]
fn by_language_breakdown_correct() {
    let rust_code = "pub fn a() {}\npub fn b() {}\nfn c() {}\n";
    let (dir, paths) = write_temp_files(&[("lib.rs", rust_code)]);
    let export = make_export(vec![make_row("lib.rs", "root", "Rust")]);
    let r = build_api_surface_report(dir.path(), &paths, &export, &default_limits()).unwrap();

    let rust_surface = &r.by_language["Rust"];
    assert_eq!(rust_surface.total_items, 3);
    assert_eq!(rust_surface.public_items, 2);
    assert_eq!(rust_surface.internal_items, 1);
}

#[test]
fn by_module_populated() {
    let code = "pub fn a() {}\n";
    let (dir, paths) = write_temp_files(&[("src/lib.rs", code)]);
    let export = make_export(vec![make_row("src/lib.rs", "src", "Rust")]);
    let r = build_api_surface_report(dir.path(), &paths, &export, &default_limits()).unwrap();

    assert!(!r.by_module.is_empty());
    assert_eq!(r.by_module[0].module, "src");
}

#[test]
fn top_exporters_populated() {
    let code = "pub fn a() {}\npub fn b() {}\npub fn c() {}\n";
    let (dir, paths) = write_temp_files(&[("lib.rs", code)]);
    let export = make_export(vec![make_row("lib.rs", "root", "Rust")]);
    let r = build_api_surface_report(dir.path(), &paths, &export, &default_limits()).unwrap();

    assert!(!r.top_exporters.is_empty());
    assert_eq!(r.top_exporters[0].public_items, 3);
}

#[test]
fn empty_file_produces_zero_report() {
    let (dir, paths) = write_temp_files(&[("empty.rs", "")]);
    let export = make_export(vec![make_row("empty.rs", "root", "Rust")]);
    let r = build_api_surface_report(dir.path(), &paths, &export, &default_limits()).unwrap();

    assert_eq!(r.total_items, 0);
    assert_eq!(r.public_ratio, 0.0);
    assert_eq!(r.documented_ratio, 0.0);
}

#[test]
fn unsupported_lang_skipped() {
    let (dir, paths) = write_temp_files(&[("readme.md", "# Hello\n")]);
    let export = make_export(vec![make_row("readme.md", "root", "Markdown")]);
    let r = build_api_surface_report(dir.path(), &paths, &export, &default_limits()).unwrap();

    assert_eq!(r.total_items, 0);
}

#[test]
fn report_deterministic_across_runs() {
    let code = "pub fn z() {}\npub fn a() {}\nfn m() {}\n";
    let r1 = report_for("lib.rs", "Rust", code);
    let r2 = report_for("lib.rs", "Rust", code);

    assert_eq!(r1.total_items, r2.total_items);
    assert_eq!(r1.public_items, r2.public_items);
    assert_eq!(r1.internal_items, r2.internal_items);
    assert_eq!(r1.public_ratio, r2.public_ratio);
    assert_eq!(r1.documented_ratio, r2.documented_ratio);
}

#[test]
fn report_json_serializes() {
    let r = report_for("lib.rs", "Rust", "pub fn foo() {}\n");
    let json = serde_json::to_string(&r).unwrap();
    assert!(json.contains("\"total_items\":1"));
}
