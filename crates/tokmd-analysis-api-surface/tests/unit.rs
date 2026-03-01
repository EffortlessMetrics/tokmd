//! Unit-level integration tests for `tokmd-analysis-api-surface`.
//!
//! These tests exercise `build_api_surface_report` with focused scenarios
//! covering edge cases, trait implementations, limit behavior, and
//! multi-language/multi-file interactions.

use std::fs;
use std::path::PathBuf;

use tokmd_analysis_api_surface::build_api_surface_report;
use tokmd_analysis_types::{ApiExportItem, ApiSurfaceReport, LangApiSurface, ModuleApiRow};
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
// Trait implementation smoke tests
// ---------------------------------------------------------------------------

#[test]
fn api_surface_report_is_debug() {
    let dir = tempfile::tempdir().unwrap();
    let export = make_export(vec![]);
    let report = build_api_surface_report(dir.path(), &[], &export, &default_limits()).unwrap();
    let debug = format!("{:?}", report);
    assert!(debug.contains("total_items"));
}

#[test]
fn api_surface_report_is_clone() {
    let dir = tempfile::tempdir().unwrap();
    let export = make_export(vec![]);
    let report = build_api_surface_report(dir.path(), &[], &export, &default_limits()).unwrap();
    let cloned = report.clone();
    assert_eq!(cloned.total_items, report.total_items);
    assert_eq!(cloned.public_items, report.public_items);
    assert_eq!(cloned.internal_items, report.internal_items);
}

#[test]
fn lang_api_surface_debug_and_clone() {
    let surface = LangApiSurface {
        total_items: 5,
        public_items: 3,
        internal_items: 2,
        public_ratio: 0.6,
    };
    let debug = format!("{:?}", surface);
    assert!(debug.contains("LangApiSurface"));
    let cloned = surface.clone();
    assert_eq!(cloned.total_items, 5);
}

#[test]
fn module_api_row_debug_and_clone() {
    let row = ModuleApiRow {
        module: "src".to_string(),
        total_items: 10,
        public_items: 7,
        public_ratio: 0.7,
    };
    let debug = format!("{:?}", row);
    assert!(debug.contains("ModuleApiRow"));
    let cloned = row.clone();
    assert_eq!(cloned.module, "src");
}

#[test]
fn api_export_item_debug_and_clone() {
    let item = ApiExportItem {
        path: "lib.rs".to_string(),
        lang: "Rust".to_string(),
        public_items: 3,
        total_items: 5,
    };
    let debug = format!("{:?}", item);
    assert!(debug.contains("ApiExportItem"));
    let cloned = item.clone();
    assert_eq!(cloned.path, "lib.rs");
}

// ---------------------------------------------------------------------------
// Go var/const declarations
// ---------------------------------------------------------------------------

#[test]
fn go_var_and_const_detected() {
    let code =
        "var PublicVar int = 42\nvar privateVar string\nconst MaxRetries = 3\nconst maxBuf = 64\n";
    let (dir, paths) = write_temp_files(&[("main.go", code)]);
    let export = make_export(vec![make_row("main.go", ".", "Go")]);
    let report = build_api_surface_report(dir.path(), &paths, &export, &default_limits()).unwrap();

    // PublicVar (pub), privateVar (internal), MaxRetries (pub), maxBuf (internal)
    assert_eq!(report.total_items, 4);
    assert_eq!(report.public_items, 2);
    assert_eq!(report.internal_items, 2);
}

// ---------------------------------------------------------------------------
// Java abstract/final/record/sealed
// ---------------------------------------------------------------------------

#[test]
fn java_abstract_and_final_classes() {
    let code = "public abstract class Base {}\npublic final class Derived {}\nabstract class Internal {}\n";
    let (dir, paths) = write_temp_files(&[("App.java", code)]);
    let export = make_export(vec![make_row("App.java", ".", "Java")]);
    let report = build_api_surface_report(dir.path(), &paths, &export, &default_limits()).unwrap();

    assert_eq!(report.public_items, 2);
    assert_eq!(report.internal_items, 1);
}

#[test]
fn java_record_and_sealed() {
    let code = "public record Point(int x, int y) {}\npublic sealed class Shape {}\nrecord Internal(String s) {}\n";
    let (dir, paths) = write_temp_files(&[("Point.java", code)]);
    let export = make_export(vec![make_row("Point.java", ".", "Java")]);
    let report = build_api_surface_report(dir.path(), &paths, &export, &default_limits()).unwrap();

    assert_eq!(report.public_items, 2);
    assert_eq!(report.internal_items, 1);
}

// ---------------------------------------------------------------------------
// JS/TS additional export forms
// ---------------------------------------------------------------------------

#[test]
fn ts_export_abstract_class() {
    let code = "export abstract class Base {}\nexport async function fetchData() {}\n";
    let (dir, paths) = write_temp_files(&[("mod.ts", code)]);
    let export = make_export(vec![make_row("mod.ts", ".", "TypeScript")]);
    let report = build_api_surface_report(dir.path(), &paths, &export, &default_limits()).unwrap();

    assert_eq!(report.public_items, 2);
}

#[test]
fn js_export_let() {
    let code = "export let counter = 0;\nlet internal = 1;\n";
    let (dir, paths) = write_temp_files(&[("mod.js", code)]);
    let export = make_export(vec![make_row("mod.js", ".", "JavaScript")]);
    let report = build_api_surface_report(dir.path(), &paths, &export, &default_limits()).unwrap();

    assert_eq!(report.public_items, 1);
    assert_eq!(report.internal_items, 1);
}

// ---------------------------------------------------------------------------
// Python async def
// ---------------------------------------------------------------------------

#[test]
fn python_async_def_detected() {
    let code = "async def fetch_data():\n    pass\nasync def _internal():\n    pass\n";
    let (dir, paths) = write_temp_files(&[("async_mod.py", code)]);
    let export = make_export(vec![make_row("async_mod.py", ".", "Python")]);
    let report = build_api_surface_report(dir.path(), &paths, &export, &default_limits()).unwrap();

    assert_eq!(report.total_items, 2);
    assert_eq!(report.public_items, 1);
    assert_eq!(report.internal_items, 1);
}

// ---------------------------------------------------------------------------
// Nonexistent file in paths list — gracefully skipped
// ---------------------------------------------------------------------------

#[test]
fn nonexistent_file_gracefully_skipped() {
    let dir = tempfile::tempdir().unwrap();
    let paths = vec![PathBuf::from("does_not_exist.rs")];
    let export = make_export(vec![make_row("does_not_exist.rs", ".", "Rust")]);
    let report = build_api_surface_report(dir.path(), &paths, &export, &default_limits()).unwrap();

    assert_eq!(report.total_items, 0);
}

// ---------------------------------------------------------------------------
// File with only whitespace
// ---------------------------------------------------------------------------

#[test]
fn whitespace_only_file_yields_no_symbols() {
    let code = "   \n  \n\t\n";
    let (dir, paths) = write_temp_files(&[("blank.rs", code)]);
    let export = make_export(vec![make_row("blank.rs", ".", "Rust")]);
    let report = build_api_surface_report(dir.path(), &paths, &export, &default_limits()).unwrap();

    assert_eq!(report.total_items, 0);
}

// ---------------------------------------------------------------------------
// max_file_bytes limit truncates large files
// ---------------------------------------------------------------------------

#[test]
fn max_file_bytes_limits_individual_file_read() {
    // Create a file where the public symbol is near the end, beyond the limit
    let mut code = String::new();
    for _ in 0..200 {
        code.push_str("// padding line\n");
    }
    code.push_str("pub fn hidden() {}\n");

    let (dir, paths) = write_temp_files(&[("big.rs", &code)]);
    let export = make_export(vec![make_row("big.rs", ".", "Rust")]);
    // Set max_file_bytes to a small value that will truncate before the pub fn
    let limits = AnalysisLimits {
        max_file_bytes: Some(100),
        ..Default::default()
    };
    let report = build_api_surface_report(dir.path(), &paths, &export, &limits).unwrap();

    // The pub fn is beyond 100 bytes so should not be found
    assert_eq!(report.public_items, 0);
}

// ---------------------------------------------------------------------------
// Multiple files in the same module accumulate correctly
// ---------------------------------------------------------------------------

#[test]
fn same_module_multiple_files_accumulate() {
    let code_a = "pub fn a() {}\nfn b() {}\n";
    let code_b = "pub fn c() {}\npub fn d() {}\n";
    let (dir, paths) = write_temp_files(&[("src/a.rs", code_a), ("src/b.rs", code_b)]);
    let export = make_export(vec![
        make_row("src/a.rs", "src", "Rust"),
        make_row("src/b.rs", "src", "Rust"),
    ]);
    let report = build_api_surface_report(dir.path(), &paths, &export, &default_limits()).unwrap();

    // Single module "src" with accumulated items
    assert_eq!(report.by_module.len(), 1);
    assert_eq!(report.by_module[0].module, "src");
    assert_eq!(report.by_module[0].total_items, 4);
    assert_eq!(report.by_module[0].public_items, 3);
}

// ---------------------------------------------------------------------------
// All documented public items → documented_ratio = 1.0
// ---------------------------------------------------------------------------

#[test]
fn all_public_items_documented_ratio_is_one() {
    let code = "/// Doc A\npub fn a() {}\n/// Doc B\npub fn b() {}\n";
    let (dir, paths) = write_temp_files(&[("lib.rs", code)]);
    let export = make_export(vec![make_row("lib.rs", ".", "Rust")]);
    let report = build_api_surface_report(dir.path(), &paths, &export, &default_limits()).unwrap();

    assert_eq!(report.public_items, 2);
    assert_eq!(report.documented_ratio, 1.0);
}

// ---------------------------------------------------------------------------
// Mixed languages in single report — totals consistent
// ---------------------------------------------------------------------------

#[test]
fn six_language_report_totals_consistent() {
    let rust = "pub fn r() {}\nfn ri() {}\n";
    let js = "export function j() {}\nfunction ji() {}\n";
    let ts = "export interface TI {}\ninterface ti {}\n";
    let py = "def p():\n    pass\ndef _pi():\n    pass\n";
    let go = "func G() {}\nfunc g() {}\n";
    let java = "public class J {}\nclass ji {}\n";

    let (dir, paths) = write_temp_files(&[
        ("lib.rs", rust),
        ("index.js", js),
        ("types.ts", ts),
        ("main.py", py),
        ("main.go", go),
        ("App.java", java),
    ]);
    let export = make_export(vec![
        make_row("lib.rs", "rust", "Rust"),
        make_row("index.js", "js", "JavaScript"),
        make_row("types.ts", "ts", "TypeScript"),
        make_row("main.py", "py", "Python"),
        make_row("main.go", "go", "Go"),
        make_row("App.java", "java", "Java"),
    ]);
    let report = build_api_surface_report(dir.path(), &paths, &export, &default_limits()).unwrap();

    assert_eq!(report.by_language.len(), 6);
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
// Top exporters capped at 20
// ---------------------------------------------------------------------------

#[test]
fn top_exporters_capped_at_twenty() {
    let code = "pub fn f() {}\n";
    let mut file_list: Vec<(String, String)> = Vec::new();
    let mut rows = Vec::new();
    for i in 0..25 {
        let name = format!("mod{}.rs", i);
        file_list.push((name.clone(), code.to_string()));
        rows.push(make_row(&name, ".", "Rust"));
    }

    let dir = tempfile::tempdir().unwrap();
    let mut paths = Vec::new();
    for (name, content) in &file_list {
        fs::write(dir.path().join(name), content).unwrap();
        paths.push(PathBuf::from(name));
    }
    let export = make_export(rows);
    let report = build_api_surface_report(dir.path(), &paths, &export, &default_limits()).unwrap();

    assert!(report.top_exporters.len() <= 20);
    assert_eq!(report.public_items, 25);
}

// ---------------------------------------------------------------------------
// by_module capped at 50
// ---------------------------------------------------------------------------

#[test]
fn by_module_capped_at_fifty() {
    let code = "pub fn f() {}\n";
    let mut file_list: Vec<(String, String)> = Vec::new();
    let mut rows = Vec::new();
    for i in 0..55 {
        let name = format!("mod{}/lib.rs", i);
        let module = format!("mod{}", i);
        file_list.push((name.clone(), code.to_string()));
        rows.push(make_row(&name, &module, "Rust"));
    }

    let dir = tempfile::tempdir().unwrap();
    let mut paths = Vec::new();
    for (name, content) in &file_list {
        let full = dir.path().join(name);
        fs::create_dir_all(full.parent().unwrap()).unwrap();
        fs::write(&full, content).unwrap();
        paths.push(PathBuf::from(name));
    }
    let export = make_export(rows);
    let report = build_api_surface_report(dir.path(), &paths, &export, &default_limits()).unwrap();

    assert!(report.by_module.len() <= 50);
}

// ---------------------------------------------------------------------------
// Rust doc comment variants: //!, #[doc = ...]
// ---------------------------------------------------------------------------

#[test]
fn rust_doc_bang_comment_detected() {
    let code = "//! Module doc\npub fn after_module_doc() {}\n";
    let (dir, paths) = write_temp_files(&[("lib.rs", code)]);
    let export = make_export(vec![make_row("lib.rs", ".", "Rust")]);
    let report = build_api_surface_report(dir.path(), &paths, &export, &default_limits()).unwrap();

    assert_eq!(report.public_items, 1);
    assert_eq!(report.documented_ratio, 1.0);
}

#[test]
fn rust_doc_attribute_detected() {
    let code = "#[doc = \"documented\"]\npub fn attr_doc() {}\n";
    let (dir, paths) = write_temp_files(&[("lib.rs", code)]);
    let export = make_export(vec![make_row("lib.rs", ".", "Rust")]);
    let report = build_api_surface_report(dir.path(), &paths, &export, &default_limits()).unwrap();

    assert_eq!(report.documented_ratio, 1.0);
}

// ---------------------------------------------------------------------------
// Top exporters sorted by public_items, then by path
// ---------------------------------------------------------------------------

#[test]
fn top_exporters_tiebreak_by_path() {
    // Two files with equal public items — should be sorted by path
    let code = "pub fn f() {}\n";
    let (dir, paths) = write_temp_files(&[("b.rs", code), ("a.rs", code)]);
    let export = make_export(vec![
        make_row("b.rs", ".", "Rust"),
        make_row("a.rs", ".", "Rust"),
    ]);
    let report = build_api_surface_report(dir.path(), &paths, &export, &default_limits()).unwrap();

    assert_eq!(report.top_exporters.len(), 2);
    // Same public_items, so alphabetical by path
    assert_eq!(report.top_exporters[0].path, "a.rs");
    assert_eq!(report.top_exporters[1].path, "b.rs");
}

// ---------------------------------------------------------------------------
// by_module tiebreak by module name
// ---------------------------------------------------------------------------

#[test]
fn by_module_tiebreak_by_name() {
    let code = "pub fn f() {}\n";
    let (dir, paths) = write_temp_files(&[("b/lib.rs", code), ("a/lib.rs", code)]);
    let export = make_export(vec![
        make_row("b/lib.rs", "mod_b", "Rust"),
        make_row("a/lib.rs", "mod_a", "Rust"),
    ]);
    let report = build_api_surface_report(dir.path(), &paths, &export, &default_limits()).unwrap();

    assert_eq!(report.by_module.len(), 2);
    // Same total_items, so alphabetical by module name
    assert_eq!(report.by_module[0].module, "mod_a");
    assert_eq!(report.by_module[1].module, "mod_b");
}

// ---------------------------------------------------------------------------
// Serialization round-trip (serde)
// ---------------------------------------------------------------------------

#[test]
fn report_serializes_to_json_and_back() {
    let code = "pub fn a() {}\nfn b() {}\n/// Documented\npub fn c() {}\n";
    let (dir, paths) = write_temp_files(&[("lib.rs", code)]);
    let export = make_export(vec![make_row("lib.rs", "src", "Rust")]);
    let report = build_api_surface_report(dir.path(), &paths, &export, &default_limits()).unwrap();

    let json = serde_json::to_string(&report).unwrap();
    let deserialized: ApiSurfaceReport = serde_json::from_str(&json).unwrap();

    assert_eq!(deserialized.total_items, report.total_items);
    assert_eq!(deserialized.public_items, report.public_items);
    assert_eq!(deserialized.internal_items, report.internal_items);
    assert_eq!(deserialized.public_ratio, report.public_ratio);
    assert_eq!(deserialized.documented_ratio, report.documented_ratio);
    assert_eq!(deserialized.by_language.len(), report.by_language.len());
    assert_eq!(deserialized.by_module.len(), report.by_module.len());
    assert_eq!(deserialized.top_exporters.len(), report.top_exporters.len());
}
