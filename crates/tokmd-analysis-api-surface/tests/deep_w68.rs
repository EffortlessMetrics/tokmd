//! W68 deep tests for `tokmd-analysis-api-surface`.
//!
//! Covers multi-language report building, determinism, ratio calculations,
//! module breakdowns, top-exporter limits, limit behavior, mixed-language
//! aggregation, and structural invariants.

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

// ═══════════════════════════════════════════════════════════════════
// § 1. Multi-language report aggregation
// ═══════════════════════════════════════════════════════════════════

#[test]
fn multi_language_report_aggregates_all_languages() {
    let rust_code = "pub fn greet() {}\nfn helper() {}\n";
    let py_code = "def public_fn():\n    pass\ndef _private():\n    pass\n";
    let js_code = "export function api() {}\nfunction internal() {}\n";
    let (dir, paths) = write_temp_files(&[
        ("lib.rs", rust_code),
        ("main.py", py_code),
        ("index.js", js_code),
    ]);
    let export = make_export(vec![
        make_row("lib.rs", "src", "Rust"),
        make_row("main.py", "src", "Python"),
        make_row("index.js", "src", "JavaScript"),
    ]);
    let r = build_api_surface_report(dir.path(), &paths, &export, &default_limits()).unwrap();
    assert_eq!(r.by_language.len(), 3);
    assert!(r.by_language.contains_key("Rust"));
    assert!(r.by_language.contains_key("Python"));
    assert!(r.by_language.contains_key("JavaScript"));
    assert_eq!(r.total_items, 6);
    assert_eq!(r.public_items, 3);
    assert_eq!(r.internal_items, 3);
}

// ═══════════════════════════════════════════════════════════════════
// § 2. Determinism — same input always produces same output
// ═══════════════════════════════════════════════════════════════════

#[test]
fn report_is_deterministic_across_runs() {
    let code = "pub fn a() {}\npub fn b() {}\nfn c() {}\n";
    let (dir, paths) = write_temp_files(&[("lib.rs", code)]);
    let export = make_export(vec![make_row("lib.rs", ".", "Rust")]);
    let lim = default_limits();
    let r1 = build_api_surface_report(dir.path(), &paths, &export, &lim).unwrap();
    let r2 = build_api_surface_report(dir.path(), &paths, &export, &lim).unwrap();
    assert_eq!(r1.total_items, r2.total_items);
    assert_eq!(r1.public_items, r2.public_items);
    assert_eq!(r1.internal_items, r2.internal_items);
    assert_eq!(r1.public_ratio, r2.public_ratio);
    assert_eq!(r1.documented_ratio, r2.documented_ratio);
    assert_eq!(r1.by_language.len(), r2.by_language.len());
    assert_eq!(r1.by_module.len(), r2.by_module.len());
    assert_eq!(r1.top_exporters.len(), r2.top_exporters.len());
}

// ═══════════════════════════════════════════════════════════════════
// § 3. Public ratio calculations
// ═══════════════════════════════════════════════════════════════════

#[test]
fn public_ratio_all_public() {
    let code = "pub fn a() {}\npub fn b() {}\n";
    let (dir, paths) = write_temp_files(&[("lib.rs", code)]);
    let export = make_export(vec![make_row("lib.rs", ".", "Rust")]);
    let r = build_api_surface_report(dir.path(), &paths, &export, &default_limits()).unwrap();
    assert_eq!(r.public_ratio, 1.0);
}

#[test]
fn public_ratio_all_internal() {
    let code = "fn a() {}\nfn b() {}\n";
    let (dir, paths) = write_temp_files(&[("lib.rs", code)]);
    let export = make_export(vec![make_row("lib.rs", ".", "Rust")]);
    let r = build_api_surface_report(dir.path(), &paths, &export, &default_limits()).unwrap();
    assert_eq!(r.public_ratio, 0.0);
}

#[test]
fn public_ratio_mixed_items() {
    let code = "pub fn a() {}\nfn b() {}\npub fn c() {}\nfn d() {}\n";
    let (dir, paths) = write_temp_files(&[("lib.rs", code)]);
    let export = make_export(vec![make_row("lib.rs", ".", "Rust")]);
    let r = build_api_surface_report(dir.path(), &paths, &export, &default_limits()).unwrap();
    assert_eq!(r.public_ratio, 0.5);
}

// ═══════════════════════════════════════════════════════════════════
// § 4. Documented ratio calculations
// ═══════════════════════════════════════════════════════════════════

#[test]
fn documented_ratio_fully_documented() {
    let code = "/// Doc A\npub fn a() {}\n/// Doc B\npub fn b() {}\n";
    let (dir, paths) = write_temp_files(&[("lib.rs", code)]);
    let export = make_export(vec![make_row("lib.rs", ".", "Rust")]);
    let r = build_api_surface_report(dir.path(), &paths, &export, &default_limits()).unwrap();
    assert_eq!(r.documented_ratio, 1.0);
}

#[test]
fn documented_ratio_half_documented() {
    let code = "/// Documented\npub fn a() {}\npub fn b() {}\n";
    let (dir, paths) = write_temp_files(&[("lib.rs", code)]);
    let export = make_export(vec![make_row("lib.rs", ".", "Rust")]);
    let r = build_api_surface_report(dir.path(), &paths, &export, &default_limits()).unwrap();
    assert_eq!(r.documented_ratio, 0.5);
}

#[test]
fn documented_ratio_zero_when_no_public_items() {
    let code = "fn a() {}\nfn b() {}\n";
    let (dir, paths) = write_temp_files(&[("lib.rs", code)]);
    let export = make_export(vec![make_row("lib.rs", ".", "Rust")]);
    let r = build_api_surface_report(dir.path(), &paths, &export, &default_limits()).unwrap();
    assert_eq!(r.documented_ratio, 0.0);
}

// ═══════════════════════════════════════════════════════════════════
// § 5. Child FileKind rows are skipped
// ═══════════════════════════════════════════════════════════════════

#[test]
fn child_filerow_skipped() {
    let code = "pub fn visible() {}\n";
    let (dir, paths) = write_temp_files(&[("lib.rs", code)]);
    let mut row = make_row("lib.rs", ".", "Rust");
    row.kind = FileKind::Child;
    let export = make_export(vec![row]);
    let r = build_api_surface_report(dir.path(), &paths, &export, &default_limits()).unwrap();
    assert_eq!(r.total_items, 0);
}

// ═══════════════════════════════════════════════════════════════════
// § 6. Unsupported language files are skipped
// ═══════════════════════════════════════════════════════════════════

#[test]
fn unsupported_language_skipped() {
    let code = "# Heading\nSome text.\n";
    let (dir, paths) = write_temp_files(&[("README.md", code)]);
    let export = make_export(vec![make_row("README.md", ".", "Markdown")]);
    let r = build_api_surface_report(dir.path(), &paths, &export, &default_limits()).unwrap();
    assert_eq!(r.total_items, 0);
    assert!(r.by_language.is_empty());
}

// ═══════════════════════════════════════════════════════════════════
// § 7. Empty file list produces empty report
// ═══════════════════════════════════════════════════════════════════

#[test]
fn empty_files_empty_report() {
    let (dir, _) = write_temp_files(&[]);
    let export = make_export(vec![]);
    let r = build_api_surface_report(dir.path(), &[], &export, &default_limits()).unwrap();
    assert_eq!(r.total_items, 0);
    assert_eq!(r.public_items, 0);
    assert_eq!(r.internal_items, 0);
    assert_eq!(r.public_ratio, 0.0);
    assert_eq!(r.documented_ratio, 0.0);
    assert!(r.by_language.is_empty());
    assert!(r.by_module.is_empty());
    assert!(r.top_exporters.is_empty());
}

// ═══════════════════════════════════════════════════════════════════
// § 8. Per-language surface breakdown
// ═══════════════════════════════════════════════════════════════════

#[test]
fn by_language_has_correct_counts() {
    let rust = "pub fn x() {}\nfn y() {}\n";
    let ts = "export function a() {}\nfunction b() {}\nfunction c() {}\n";
    let (dir, paths) = write_temp_files(&[("lib.rs", rust), ("app.ts", ts)]);
    let export = make_export(vec![
        make_row("lib.rs", "src", "Rust"),
        make_row("app.ts", "src", "TypeScript"),
    ]);
    let r = build_api_surface_report(dir.path(), &paths, &export, &default_limits()).unwrap();
    let rs = r.by_language.get("Rust").unwrap();
    assert_eq!(rs.public_items, 1);
    assert_eq!(rs.internal_items, 1);
    assert_eq!(rs.total_items, 2);
    let ts_surf = r.by_language.get("TypeScript").unwrap();
    assert_eq!(ts_surf.public_items, 1);
    assert_eq!(ts_surf.internal_items, 2);
    assert_eq!(ts_surf.total_items, 3);
}

#[test]
fn by_language_keys_sorted_alphabetically() {
    let rust = "pub fn x() {}\n";
    let go = "func PublicFn() {}\n";
    let java = "public class Main {}\n";
    let (dir, paths) = write_temp_files(&[("lib.rs", rust), ("main.go", go), ("Main.java", java)]);
    let export = make_export(vec![
        make_row("lib.rs", ".", "Rust"),
        make_row("main.go", ".", "Go"),
        make_row("Main.java", ".", "Java"),
    ]);
    let r = build_api_surface_report(dir.path(), &paths, &export, &default_limits()).unwrap();
    let keys: Vec<&String> = r.by_language.keys().collect();
    assert_eq!(keys, vec!["Go", "Java", "Rust"]);
}

// ═══════════════════════════════════════════════════════════════════
// § 9. Per-module breakdown
// ═══════════════════════════════════════════════════════════════════

#[test]
fn by_module_groups_files_from_same_module() {
    let a = "pub fn a() {}\nfn b() {}\n";
    let c = "pub fn c() {}\n";
    let (dir, paths) = write_temp_files(&[("src/a.rs", a), ("src/c.rs", c)]);
    let export = make_export(vec![
        make_row("src/a.rs", "src", "Rust"),
        make_row("src/c.rs", "src", "Rust"),
    ]);
    let r = build_api_surface_report(dir.path(), &paths, &export, &default_limits()).unwrap();
    assert_eq!(r.by_module.len(), 1);
    assert_eq!(r.by_module[0].module, "src");
    assert_eq!(r.by_module[0].total_items, 3);
    assert_eq!(r.by_module[0].public_items, 2);
}

#[test]
fn by_module_sorted_by_total_items_descending() {
    let small = "pub fn x() {}\n";
    let large = "pub fn a() {}\npub fn b() {}\npub fn c() {}\nfn d() {}\n";
    let (dir, paths) = write_temp_files(&[("small/lib.rs", small), ("large/lib.rs", large)]);
    let export = make_export(vec![
        make_row("small/lib.rs", "small", "Rust"),
        make_row("large/lib.rs", "large", "Rust"),
    ]);
    let r = build_api_surface_report(dir.path(), &paths, &export, &default_limits()).unwrap();
    assert_eq!(r.by_module[0].module, "large");
    assert_eq!(r.by_module[1].module, "small");
}

// ═══════════════════════════════════════════════════════════════════
// § 10. Top exporters
// ═══════════════════════════════════════════════════════════════════

#[test]
fn top_exporters_sorted_by_public_items_desc() {
    let few = "pub fn one() {}\n";
    let many = "pub fn a() {}\npub fn b() {}\npub fn c() {}\n";
    let (dir, paths) = write_temp_files(&[("few.rs", few), ("many.rs", many)]);
    let export = make_export(vec![
        make_row("few.rs", ".", "Rust"),
        make_row("many.rs", ".", "Rust"),
    ]);
    let r = build_api_surface_report(dir.path(), &paths, &export, &default_limits()).unwrap();
    assert_eq!(r.top_exporters.len(), 2);
    assert_eq!(r.top_exporters[0].public_items, 3);
    assert_eq!(r.top_exporters[1].public_items, 1);
}

#[test]
fn top_exporters_only_files_with_public_items() {
    let no_pub = "fn internal() {}\n";
    let has_pub = "pub fn visible() {}\n";
    let (dir, paths) = write_temp_files(&[("no.rs", no_pub), ("yes.rs", has_pub)]);
    let export = make_export(vec![
        make_row("no.rs", ".", "Rust"),
        make_row("yes.rs", ".", "Rust"),
    ]);
    let r = build_api_surface_report(dir.path(), &paths, &export, &default_limits()).unwrap();
    assert_eq!(r.top_exporters.len(), 1);
    assert_eq!(r.top_exporters[0].path, "yes.rs");
}

// ═══════════════════════════════════════════════════════════════════
// § 11. Go language – mixed visibility
// ═══════════════════════════════════════════════════════════════════

#[test]
fn go_mixed_visibility_in_report() {
    let code =
        "func PublicFunc() {}\nfunc privateFunc() {}\ntype MyStruct struct{}\nvar counter int\n";
    let (dir, paths) = write_temp_files(&[("main.go", code)]);
    let export = make_export(vec![make_row("main.go", ".", "Go")]);
    let r = build_api_surface_report(dir.path(), &paths, &export, &default_limits()).unwrap();
    assert_eq!(r.public_items, 2); // PublicFunc + MyStruct
    assert_eq!(r.internal_items, 2); // privateFunc + counter
}

// ═══════════════════════════════════════════════════════════════════
// § 12. Java language – mixed visibility in report
// ═══════════════════════════════════════════════════════════════════

#[test]
fn java_mixed_visibility_in_report() {
    let code = "public class App {}\nprivate void helper() {}\nprotected void mid() {}\n";
    let (dir, paths) = write_temp_files(&[("App.java", code)]);
    let export = make_export(vec![make_row("App.java", ".", "Java")]);
    let r = build_api_surface_report(dir.path(), &paths, &export, &default_limits()).unwrap();
    assert_eq!(r.public_items, 1);
    assert_eq!(r.internal_items, 2);
}

// ═══════════════════════════════════════════════════════════════════
// § 13. Python language – private underscore convention in report
// ═══════════════════════════════════════════════════════════════════

#[test]
fn python_underscore_convention_in_report() {
    let code = "def public_api():\n    pass\ndef _internal():\n    pass\nclass MyClass:\n    pass\nclass _Helper:\n    pass\n";
    let (dir, paths) = write_temp_files(&[("mod.py", code)]);
    let export = make_export(vec![make_row("mod.py", ".", "Python")]);
    let r = build_api_surface_report(dir.path(), &paths, &export, &default_limits()).unwrap();
    assert_eq!(r.public_items, 2);
    assert_eq!(r.internal_items, 2);
}

// ═══════════════════════════════════════════════════════════════════
// § 14. Total invariant: public + internal == total
// ═══════════════════════════════════════════════════════════════════

#[test]
fn total_equals_public_plus_internal() {
    let code = "pub fn a() {}\nfn b() {}\npub struct S;\nenum E {}\n";
    let (dir, paths) = write_temp_files(&[("lib.rs", code)]);
    let export = make_export(vec![make_row("lib.rs", ".", "Rust")]);
    let r = build_api_surface_report(dir.path(), &paths, &export, &default_limits()).unwrap();
    assert_eq!(r.total_items, r.public_items + r.internal_items);
}

// ═══════════════════════════════════════════════════════════════════
// § 15. Public ratio per language matches hand calculation
// ═══════════════════════════════════════════════════════════════════

#[test]
fn per_language_public_ratio_correct() {
    let code = "pub fn a() {}\nfn b() {}\nfn c() {}\n";
    let (dir, paths) = write_temp_files(&[("lib.rs", code)]);
    let export = make_export(vec![make_row("lib.rs", ".", "Rust")]);
    let r = build_api_surface_report(dir.path(), &paths, &export, &default_limits()).unwrap();
    let rust_surf = r.by_language.get("Rust").unwrap();
    // 1 public out of 3 total → 0.3333
    assert_eq!(rust_surf.public_ratio, 0.3333);
}

// ═══════════════════════════════════════════════════════════════════
// § 16. Max bytes limit stops scanning
// ═══════════════════════════════════════════════════════════════════

#[test]
fn max_bytes_limit_constrains_scanning() {
    let code_a = "pub fn a() {}\n";
    let code_b = "pub fn b() {}\npub fn c() {}\n";
    let (dir, paths) = write_temp_files(&[("a.rs", code_a), ("b.rs", code_b)]);
    let export = make_export(vec![
        make_row("a.rs", ".", "Rust"),
        make_row("b.rs", ".", "Rust"),
    ]);
    let limits = AnalysisLimits {
        max_bytes: Some(1),
        ..AnalysisLimits::default()
    };
    let r = build_api_surface_report(dir.path(), &paths, &export, &limits).unwrap();
    // With max_bytes=1, only the first file should be scanned
    assert!(r.total_items <= 1);
}

// ═══════════════════════════════════════════════════════════════════
// § 17. File not on disk is gracefully skipped
// ═══════════════════════════════════════════════════════════════════

#[test]
fn missing_file_gracefully_skipped() {
    let (dir, _) = write_temp_files(&[]);
    let paths = vec![PathBuf::from("nonexistent.rs")];
    let export = make_export(vec![make_row("nonexistent.rs", ".", "Rust")]);
    let r = build_api_surface_report(dir.path(), &paths, &export, &default_limits()).unwrap();
    assert_eq!(r.total_items, 0);
}

// ═══════════════════════════════════════════════════════════════════
// § 18. TypeScript export forms detected in report
// ═══════════════════════════════════════════════════════════════════

#[test]
fn typescript_export_forms_in_report() {
    let code = "export interface IUser {}\nexport type ID = string;\nexport enum Status { A }\nexport abstract class Base {}\nfunction helper() {}\n";
    let (dir, paths) = write_temp_files(&[("types.ts", code)]);
    let export = make_export(vec![make_row("types.ts", ".", "TypeScript")]);
    let r = build_api_surface_report(dir.path(), &paths, &export, &default_limits()).unwrap();
    assert_eq!(r.public_items, 4);
    assert_eq!(r.internal_items, 1);
}

// ═══════════════════════════════════════════════════════════════════
// § 19. Module public ratio calculation
// ═══════════════════════════════════════════════════════════════════

#[test]
fn module_public_ratio_correct() {
    let code = "pub fn a() {}\nfn b() {}\n";
    let (dir, paths) = write_temp_files(&[("lib.rs", code)]);
    let export = make_export(vec![make_row("lib.rs", "mymod", "Rust")]);
    let r = build_api_surface_report(dir.path(), &paths, &export, &default_limits()).unwrap();
    assert_eq!(r.by_module.len(), 1);
    assert_eq!(r.by_module[0].module, "mymod");
    assert_eq!(r.by_module[0].public_ratio, 0.5);
}

// ═══════════════════════════════════════════════════════════════════
// § 20. Report with all six supported languages
// ═══════════════════════════════════════════════════════════════════

#[test]
fn all_six_languages_in_single_report() {
    let (dir, paths) = write_temp_files(&[
        ("lib.rs", "pub fn rust_fn() {}\n"),
        ("app.js", "export function jsFn() {}\n"),
        ("app.ts", "export function tsFn() {}\n"),
        ("app.py", "def py_fn():\n    pass\n"),
        ("main.go", "func GoFn() {}\n"),
        ("Main.java", "public class Main {}\n"),
    ]);
    let export = make_export(vec![
        make_row("lib.rs", ".", "Rust"),
        make_row("app.js", ".", "JavaScript"),
        make_row("app.ts", ".", "TypeScript"),
        make_row("app.py", ".", "Python"),
        make_row("main.go", ".", "Go"),
        make_row("Main.java", ".", "Java"),
    ]);
    let r = build_api_surface_report(dir.path(), &paths, &export, &default_limits()).unwrap();
    assert_eq!(r.by_language.len(), 6);
    assert_eq!(r.public_items, 6);
    assert_eq!(r.total_items, 6);
}
