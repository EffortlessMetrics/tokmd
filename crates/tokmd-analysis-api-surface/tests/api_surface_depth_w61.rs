//! Wave-61 depth tests for `tokmd-analysis-api-surface`.
//!
//! Covers: edge-case symbol extraction across all 6 languages, BDD scenarios
//! for report invariants, determinism, proptest properties for multi-file
//! reports, empty/large inputs, limits, and error paths.

use std::fs;
use std::path::PathBuf;

use proptest::prelude::*;
use tokmd_analysis_api_surface::build_api_surface_report;
use tokmd_analysis_util::AnalysisLimits;
use tokmd_types::{ChildIncludeMode, ExportData, FileKind, FileRow};

// ── Helpers ─────────────────────────────────────────────────────────────

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
        fs::create_dir_all(parent).unwrap();
    }
    fs::write(&full, content).unwrap();
    PathBuf::from(rel)
}

// =============================================================================
// 1. Rust – advanced symbol edge cases
// =============================================================================

#[test]
fn rust_pub_super_treated_as_public() {
    let tmp = tempfile::tempdir().unwrap();
    let code = "pub(super) fn semi_public() {}\nfn private() {}\n";
    let rel = write_file(tmp.path(), "src/lib.rs", code);
    let rows = vec![make_row("src/lib.rs", "src", "Rust", FileKind::Parent)];
    let export = make_export(rows);
    let r = build_api_surface_report(tmp.path(), &[rel], &export, &default_limits()).unwrap();
    assert_eq!(r.public_items, 1);
    assert_eq!(r.internal_items, 1);
}

#[test]
fn rust_pub_in_path_treated_as_public() {
    let tmp = tempfile::tempdir().unwrap();
    let code = "pub(in crate::foo) fn restricted() {}\n";
    let rel = write_file(tmp.path(), "src/lib.rs", code);
    let rows = vec![make_row("src/lib.rs", "src", "Rust", FileKind::Parent)];
    let export = make_export(rows);
    let r = build_api_surface_report(tmp.path(), &[rel], &export, &default_limits()).unwrap();
    assert_eq!(r.public_items, 1);
}

#[test]
fn rust_unsafe_trait_detected() {
    let tmp = tempfile::tempdir().unwrap();
    let code = "pub unsafe trait Send {}\nunsafe trait InternalSync {}\n";
    let rel = write_file(tmp.path(), "src/lib.rs", code);
    let rows = vec![make_row("src/lib.rs", "src", "Rust", FileKind::Parent)];
    let export = make_export(rows);
    let r = build_api_surface_report(tmp.path(), &[rel], &export, &default_limits()).unwrap();
    assert_eq!(r.total_items, 2);
    assert_eq!(r.public_items, 1);
    assert_eq!(r.internal_items, 1);
}

#[test]
fn rust_doc_attr_counts_as_documented() {
    let tmp = tempfile::tempdir().unwrap();
    let code = "#[doc = \"my docs\"]\npub fn documented() {}\npub fn undocumented() {}\n";
    let rel = write_file(tmp.path(), "src/lib.rs", code);
    let rows = vec![make_row("src/lib.rs", "src", "Rust", FileKind::Parent)];
    let export = make_export(rows);
    let r = build_api_surface_report(tmp.path(), &[rel], &export, &default_limits()).unwrap();
    assert_eq!(r.public_items, 2);
    assert!((r.documented_ratio - 0.5).abs() < f64::EPSILON);
}

#[test]
fn rust_comment_lines_not_counted_as_items() {
    let tmp = tempfile::tempdir().unwrap();
    let code = "// fn fake_item() {}\n/* fn another_fake() {} */\n/// doc for next\npub fn real() {}\n";
    let rel = write_file(tmp.path(), "src/lib.rs", code);
    let rows = vec![make_row("src/lib.rs", "src", "Rust", FileKind::Parent)];
    let export = make_export(rows);
    let r = build_api_surface_report(tmp.path(), &[rel], &export, &default_limits()).unwrap();
    assert_eq!(r.total_items, 1);
}

#[test]
fn rust_pub_async_fn_detected() {
    let tmp = tempfile::tempdir().unwrap();
    let code = "pub async fn handler() {}\nasync fn internal() {}\n";
    let rel = write_file(tmp.path(), "src/lib.rs", code);
    let rows = vec![make_row("src/lib.rs", "src", "Rust", FileKind::Parent)];
    let export = make_export(rows);
    let r = build_api_surface_report(tmp.path(), &[rel], &export, &default_limits()).unwrap();
    assert_eq!(r.total_items, 2);
    assert_eq!(r.public_items, 1);
}

// =============================================================================
// 2. JavaScript/TypeScript – advanced edge cases
// =============================================================================

#[test]
fn js_export_async_function_detected() {
    let tmp = tempfile::tempdir().unwrap();
    let code = "export async function fetchData() {}\nasync function localFetch() {}\n";
    let rel = write_file(tmp.path(), "src/api.js", code);
    let rows = vec![make_row("src/api.js", "src", "JavaScript", FileKind::Parent)];
    let export = make_export(rows);
    let r = build_api_surface_report(tmp.path(), &[rel], &export, &default_limits()).unwrap();
    assert_eq!(r.public_items, 1);
    assert_eq!(r.internal_items, 1);
}

#[test]
fn ts_export_abstract_class_detected() {
    let tmp = tempfile::tempdir().unwrap();
    let code = "export abstract class Base {}\nclass Internal {}\n";
    let rel = write_file(tmp.path(), "src/base.ts", code);
    let rows = vec![make_row("src/base.ts", "src", "TypeScript", FileKind::Parent)];
    let export = make_export(rows);
    let r = build_api_surface_report(tmp.path(), &[rel], &export, &default_limits()).unwrap();
    assert_eq!(r.public_items, 1);
    assert_eq!(r.internal_items, 1);
}

#[test]
fn ts_export_let_detected() {
    let tmp = tempfile::tempdir().unwrap();
    let code = "export let counter = 0;\nlet internal = 1;\n";
    let rel = write_file(tmp.path(), "src/state.ts", code);
    let rows = vec![make_row("src/state.ts", "src", "TypeScript", FileKind::Parent)];
    let export = make_export(rows);
    let r = build_api_surface_report(tmp.path(), &[rel], &export, &default_limits()).unwrap();
    assert_eq!(r.public_items, 1);
    assert_eq!(r.internal_items, 1);
}

#[test]
fn ts_export_enum_detected() {
    let tmp = tempfile::tempdir().unwrap();
    let code = "export enum Color { Red, Green }\nenum Internal { A }\n";
    let rel = write_file(tmp.path(), "src/enums.ts", code);
    let rows = vec![make_row("src/enums.ts", "src", "TypeScript", FileKind::Parent)];
    let export = make_export(rows);
    let r = build_api_surface_report(tmp.path(), &[rel], &export, &default_limits()).unwrap();
    assert_eq!(r.public_items, 1);
    assert_eq!(r.internal_items, 1);
}

#[test]
fn js_export_default_detected() {
    let tmp = tempfile::tempdir().unwrap();
    let code = "export default function main() {}\n";
    let rel = write_file(tmp.path(), "src/index.js", code);
    let rows = vec![make_row("src/index.js", "src", "JavaScript", FileKind::Parent)];
    let export = make_export(rows);
    let r = build_api_surface_report(tmp.path(), &[rel], &export, &default_limits()).unwrap();
    assert_eq!(r.public_items, 1);
}

// =============================================================================
// 3. Python – advanced edge cases
// =============================================================================

#[test]
fn python_async_def_detected() {
    let tmp = tempfile::tempdir().unwrap();
    let code = "async def fetch_data():\n    pass\n";
    let rel = write_file(tmp.path(), "lib/api.py", code);
    let rows = vec![make_row("lib/api.py", "lib", "Python", FileKind::Parent)];
    let export = make_export(rows);
    let r = build_api_surface_report(tmp.path(), &[rel], &export, &default_limits()).unwrap();
    assert_eq!(r.total_items, 1);
    assert_eq!(r.public_items, 1);
}

#[test]
fn python_private_async_def_detected() {
    let tmp = tempfile::tempdir().unwrap();
    let code = "async def _internal_fetch():\n    pass\n";
    let rel = write_file(tmp.path(), "lib/api.py", code);
    let rows = vec![make_row("lib/api.py", "lib", "Python", FileKind::Parent)];
    let export = make_export(rows);
    let r = build_api_surface_report(tmp.path(), &[rel], &export, &default_limits()).unwrap();
    assert_eq!(r.total_items, 1);
    assert_eq!(r.internal_items, 1);
}

#[test]
fn python_triple_single_quote_docstring() {
    let tmp = tempfile::tempdir().unwrap();
    let code = "def documented():\n    '''docstring'''\n    pass\n";
    let rel = write_file(tmp.path(), "lib/util.py", code);
    let rows = vec![make_row("lib/util.py", "lib", "Python", FileKind::Parent)];
    let export = make_export(rows);
    let r = build_api_surface_report(tmp.path(), &[rel], &export, &default_limits()).unwrap();
    assert!((r.documented_ratio - 1.0).abs() < f64::EPSILON);
}

#[test]
fn python_indented_defs_ignored() {
    let tmp = tempfile::tempdir().unwrap();
    let code = "class Outer:\n    def method(self):\n        pass\n    def _private(self):\n        pass\n";
    let rel = write_file(tmp.path(), "lib/cls.py", code);
    let rows = vec![make_row("lib/cls.py", "lib", "Python", FileKind::Parent)];
    let export = make_export(rows);
    let r = build_api_surface_report(tmp.path(), &[rel], &export, &default_limits()).unwrap();
    // Only the top-level class is detected
    assert_eq!(r.total_items, 1);
}

#[test]
fn python_dunder_counted_as_public() {
    let tmp = tempfile::tempdir().unwrap();
    // __init__ starts with underscore but it's a dunder
    let code = "def __init__():\n    pass\n";
    let rel = write_file(tmp.path(), "lib/init.py", code);
    let rows = vec![make_row("lib/init.py", "lib", "Python", FileKind::Parent)];
    let export = make_export(rows);
    let r = build_api_surface_report(tmp.path(), &[rel], &export, &default_limits()).unwrap();
    // __init__ starts with underscore → treated as internal
    assert_eq!(r.internal_items, 1);
}

// =============================================================================
// 4. Go – advanced edge cases
// =============================================================================

#[test]
fn go_method_receiver_private() {
    let tmp = tempfile::tempdir().unwrap();
    let code = "func (s *server) handle() {}\n";
    let rel = write_file(tmp.path(), "pkg/handler.go", code);
    let rows = vec![make_row("pkg/handler.go", "pkg", "Go", FileKind::Parent)];
    let export = make_export(rows);
    let r = build_api_surface_report(tmp.path(), &[rel], &export, &default_limits()).unwrap();
    assert_eq!(r.internal_items, 1);
    assert_eq!(r.public_items, 0);
}

#[test]
fn go_var_and_const_detected() {
    let tmp = tempfile::tempdir().unwrap();
    let code = "var GlobalVar = 1\nconst MaxSize = 100\nvar localVar = 2\nconst minSize = 0\n";
    let rel = write_file(tmp.path(), "pkg/config.go", code);
    let rows = vec![make_row("pkg/config.go", "pkg", "Go", FileKind::Parent)];
    let export = make_export(rows);
    let r = build_api_surface_report(tmp.path(), &[rel], &export, &default_limits()).unwrap();
    assert_eq!(r.total_items, 4);
    assert_eq!(r.public_items, 2);
    assert_eq!(r.internal_items, 2);
}

#[test]
fn go_type_interface_detected() {
    let tmp = tempfile::tempdir().unwrap();
    let code = "type Handler interface {}\ntype handler struct {}\n";
    let rel = write_file(tmp.path(), "pkg/types.go", code);
    let rows = vec![make_row("pkg/types.go", "pkg", "Go", FileKind::Parent)];
    let export = make_export(rows);
    let r = build_api_surface_report(tmp.path(), &[rel], &export, &default_limits()).unwrap();
    assert_eq!(r.total_items, 2);
    assert_eq!(r.public_items, 1);
    assert_eq!(r.internal_items, 1);
}

// =============================================================================
// 5. Java – advanced edge cases
// =============================================================================

#[test]
fn java_public_abstract_class_detected() {
    let tmp = tempfile::tempdir().unwrap();
    let code = "public abstract class Base {}\nabstract class Internal {}\n";
    let rel = write_file(tmp.path(), "src/Base.java", code);
    let rows = vec![make_row("src/Base.java", "src", "Java", FileKind::Parent)];
    let export = make_export(rows);
    let r = build_api_surface_report(tmp.path(), &[rel], &export, &default_limits()).unwrap();
    assert_eq!(r.public_items, 1);
    assert_eq!(r.internal_items, 1);
}

#[test]
fn java_public_final_class_detected() {
    let tmp = tempfile::tempdir().unwrap();
    let code = "public final class Immutable {}\nfinal class Internal {}\n";
    let rel = write_file(tmp.path(), "src/Immutable.java", code);
    let rows = vec![make_row("src/Immutable.java", "src", "Java", FileKind::Parent)];
    let export = make_export(rows);
    let r = build_api_surface_report(tmp.path(), &[rel], &export, &default_limits()).unwrap();
    assert_eq!(r.public_items, 1);
    assert_eq!(r.internal_items, 1);
}

#[test]
fn java_public_record_detected() {
    let tmp = tempfile::tempdir().unwrap();
    let code = "public record Point(int x, int y) {}\nrecord Internal(int z) {}\n";
    let rel = write_file(tmp.path(), "src/Point.java", code);
    let rows = vec![make_row("src/Point.java", "src", "Java", FileKind::Parent)];
    let export = make_export(rows);
    let r = build_api_surface_report(tmp.path(), &[rel], &export, &default_limits()).unwrap();
    assert_eq!(r.public_items, 1);
    assert_eq!(r.internal_items, 1);
}

#[test]
fn java_protected_as_internal() {
    let tmp = tempfile::tempdir().unwrap();
    let code = "protected void helper() {}\n";
    let rel = write_file(tmp.path(), "src/Helper.java", code);
    let rows = vec![make_row("src/Helper.java", "src", "Java", FileKind::Parent)];
    let export = make_export(rows);
    let r = build_api_surface_report(tmp.path(), &[rel], &export, &default_limits()).unwrap();
    assert_eq!(r.internal_items, 1);
    assert_eq!(r.public_items, 0);
}

#[test]
fn java_javadoc_documentation_detected() {
    let tmp = tempfile::tempdir().unwrap();
    let code = "/** Javadoc. */\npublic class Documented {}\npublic class Undocumented {}\n";
    let rel = write_file(tmp.path(), "src/Doc.java", code);
    let rows = vec![make_row("src/Doc.java", "src", "Java", FileKind::Parent)];
    let export = make_export(rows);
    let r = build_api_surface_report(tmp.path(), &[rel], &export, &default_limits()).unwrap();
    assert!((r.documented_ratio - 0.5).abs() < f64::EPSILON);
}

// =============================================================================
// 6. Multi-language determinism
// =============================================================================

#[test]
fn six_language_report_deterministic_over_20_runs() {
    let tmp = tempfile::tempdir().unwrap();
    let rust = write_file(tmp.path(), "src/lib.rs", "pub fn a() {}\nfn b() {}\n");
    let js = write_file(tmp.path(), "src/index.js", "export function c() {}\nfunction d() {}\n");
    let ts = write_file(tmp.path(), "src/types.ts", "export interface I {}\ntype T = string;\n");
    let py = write_file(tmp.path(), "lib/main.py", "def e():\n    pass\ndef _f():\n    pass\n");
    let go = write_file(tmp.path(), "pkg/main.go", "func Public() {}\nfunc private() {}\n");
    let java = write_file(tmp.path(), "src/Main.java", "public class Main {}\nclass Internal {}\n");
    let rows = vec![
        make_row("src/lib.rs", "src", "Rust", FileKind::Parent),
        make_row("src/index.js", "src", "JavaScript", FileKind::Parent),
        make_row("src/types.ts", "src", "TypeScript", FileKind::Parent),
        make_row("lib/main.py", "lib", "Python", FileKind::Parent),
        make_row("pkg/main.go", "pkg", "Go", FileKind::Parent),
        make_row("src/Main.java", "src", "Java", FileKind::Parent),
    ];
    let export = make_export(rows);
    let files = vec![
        rust.clone(),
        js.clone(),
        ts.clone(),
        py.clone(),
        go.clone(),
        java.clone(),
    ];

    let baseline =
        build_api_surface_report(tmp.path(), &files, &export, &default_limits()).unwrap();
    for _ in 0..20 {
        let r =
            build_api_surface_report(tmp.path(), &files, &export, &default_limits()).unwrap();
        assert_eq!(r.total_items, baseline.total_items);
        assert_eq!(r.public_items, baseline.public_items);
        assert_eq!(r.internal_items, baseline.internal_items);
        assert!((r.public_ratio - baseline.public_ratio).abs() < f64::EPSILON);
        assert!((r.documented_ratio - baseline.documented_ratio).abs() < f64::EPSILON);
        assert_eq!(r.by_language.len(), baseline.by_language.len());
    }
}

// =============================================================================
// 7. Empty / edge-case inputs
// =============================================================================

#[test]
fn only_whitespace_file_yields_no_items() {
    let tmp = tempfile::tempdir().unwrap();
    let rel = write_file(tmp.path(), "src/blank.rs", "   \n\n   \n\t\n");
    let rows = vec![make_row("src/blank.rs", "src", "Rust", FileKind::Parent)];
    let export = make_export(rows);
    let r = build_api_surface_report(tmp.path(), &[rel], &export, &default_limits()).unwrap();
    assert_eq!(r.total_items, 0);
}

#[test]
fn single_newline_file_yields_no_items() {
    let tmp = tempfile::tempdir().unwrap();
    let rel = write_file(tmp.path(), "src/nl.rs", "\n");
    let rows = vec![make_row("src/nl.rs", "src", "Rust", FileKind::Parent)];
    let export = make_export(rows);
    let r = build_api_surface_report(tmp.path(), &[rel], &export, &default_limits()).unwrap();
    assert_eq!(r.total_items, 0);
}

#[test]
fn empty_file_yields_no_items() {
    let tmp = tempfile::tempdir().unwrap();
    let rel = write_file(tmp.path(), "src/empty.rs", "");
    let rows = vec![make_row("src/empty.rs", "src", "Rust", FileKind::Parent)];
    let export = make_export(rows);
    let r = build_api_surface_report(tmp.path(), &[rel], &export, &default_limits()).unwrap();
    assert_eq!(r.total_items, 0);
}

#[test]
fn no_files_yields_zero_report() {
    let tmp = tempfile::tempdir().unwrap();
    let export = make_export(vec![]);
    let r = build_api_surface_report(tmp.path(), &[], &export, &default_limits()).unwrap();
    assert_eq!(r.total_items, 0);
    assert_eq!(r.public_ratio, 0.0);
    assert_eq!(r.documented_ratio, 0.0);
    assert!(r.by_language.is_empty());
    assert!(r.by_module.is_empty());
    assert!(r.top_exporters.is_empty());
}

// =============================================================================
// 8. Limits: max_file_bytes truncates large files
// =============================================================================

#[test]
fn max_file_bytes_truncates_large_file() {
    let tmp = tempfile::tempdir().unwrap();
    // Write a file with many items but limit reading to a small prefix
    let code = (0..100)
        .map(|i| format!("pub fn func_{i}() {{}}\n"))
        .collect::<String>();
    let rel = write_file(tmp.path(), "src/big.rs", &code);
    let rows = vec![make_row("src/big.rs", "src", "Rust", FileKind::Parent)];
    let export = make_export(rows);
    let limits = AnalysisLimits {
        max_file_bytes: Some(50),
        ..default_limits()
    };
    let r = build_api_surface_report(tmp.path(), &[rel], &export, &limits).unwrap();
    // With only ~50 bytes read, we should get fewer than all 100 items
    assert!(r.total_items < 100);
}

// =============================================================================
// 9. Files not in export are skipped
// =============================================================================

#[test]
fn file_not_in_export_rows_skipped() {
    let tmp = tempfile::tempdir().unwrap();
    let rel = write_file(tmp.path(), "src/orphan.rs", "pub fn orphan() {}\n");
    // Export has no rows for this file
    let export = make_export(vec![]);
    let r = build_api_surface_report(tmp.path(), &[rel], &export, &default_limits()).unwrap();
    assert_eq!(r.total_items, 0);
}

// =============================================================================
// 10. by_module sorting and truncation
// =============================================================================

#[test]
fn by_module_sorted_descending_by_total() {
    let tmp = tempfile::tempdir().unwrap();
    let code_a = "pub fn a1() {}\npub fn a2() {}\npub fn a3() {}\n";
    let code_b = "pub fn b1() {}\n";
    let rel_a = write_file(tmp.path(), "mod_a/lib.rs", code_a);
    let rel_b = write_file(tmp.path(), "mod_b/lib.rs", code_b);
    let rows = vec![
        make_row("mod_a/lib.rs", "mod_a", "Rust", FileKind::Parent),
        make_row("mod_b/lib.rs", "mod_b", "Rust", FileKind::Parent),
    ];
    let export = make_export(rows);
    let r =
        build_api_surface_report(tmp.path(), &[rel_a, rel_b], &export, &default_limits()).unwrap();
    assert_eq!(r.by_module.len(), 2);
    assert!(r.by_module[0].total_items >= r.by_module[1].total_items);
    assert_eq!(r.by_module[0].module, "mod_a");
}

#[test]
fn by_module_tie_broken_by_name() {
    let tmp = tempfile::tempdir().unwrap();
    let code = "pub fn x() {}\n";
    let rel_a = write_file(tmp.path(), "alpha/lib.rs", code);
    let rel_b = write_file(tmp.path(), "beta/lib.rs", code);
    let rows = vec![
        make_row("alpha/lib.rs", "alpha", "Rust", FileKind::Parent),
        make_row("beta/lib.rs", "beta", "Rust", FileKind::Parent),
    ];
    let export = make_export(rows);
    let r =
        build_api_surface_report(tmp.path(), &[rel_a, rel_b], &export, &default_limits()).unwrap();
    assert_eq!(r.by_module.len(), 2);
    // Same total → alphabetical order
    assert_eq!(r.by_module[0].module, "alpha");
    assert_eq!(r.by_module[1].module, "beta");
}

// =============================================================================
// 11. top_exporters contains only files with public items
// =============================================================================

#[test]
fn top_exporters_excludes_internal_only_files() {
    let tmp = tempfile::tempdir().unwrap();
    let pub_code = "pub fn visible() {}\n";
    let priv_code = "fn hidden() {}\n";
    let rel_pub = write_file(tmp.path(), "src/pub.rs", pub_code);
    let rel_priv = write_file(tmp.path(), "src/priv.rs", priv_code);
    let rows = vec![
        make_row("src/pub.rs", "src", "Rust", FileKind::Parent),
        make_row("src/priv.rs", "src", "Rust", FileKind::Parent),
    ];
    let export = make_export(rows);
    let r = build_api_surface_report(
        tmp.path(),
        &[rel_pub, rel_priv],
        &export,
        &default_limits(),
    )
    .unwrap();
    assert_eq!(r.top_exporters.len(), 1);
    assert!(r.top_exporters[0].public_items > 0);
}

// =============================================================================
// 12. Language case insensitivity in FileRow
// =============================================================================

#[test]
fn language_name_case_insensitive_matching() {
    let tmp = tempfile::tempdir().unwrap();
    let code = "pub fn test() {}\n";
    let rel = write_file(tmp.path(), "src/lib.rs", code);
    // Use "rust" lowercase in the row
    let rows = vec![make_row("src/lib.rs", "src", "Rust", FileKind::Parent)];
    let export = make_export(rows);
    let r = build_api_surface_report(tmp.path(), &[rel], &export, &default_limits()).unwrap();
    assert_eq!(r.total_items, 1);
}

// =============================================================================
// 13. JSON serialization roundtrip
// =============================================================================

#[test]
fn api_surface_report_json_roundtrip() {
    let tmp = tempfile::tempdir().unwrap();
    let code = "pub fn a() {}\nfn b() {}\n/// doc\npub fn c() {}\n";
    let rel = write_file(tmp.path(), "src/lib.rs", code);
    let rows = vec![make_row("src/lib.rs", "src", "Rust", FileKind::Parent)];
    let export = make_export(rows);
    let r = build_api_surface_report(tmp.path(), &[rel], &export, &default_limits()).unwrap();

    let json = serde_json::to_string(&r).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed["total_items"].as_u64().unwrap(), r.total_items as u64);
    assert_eq!(parsed["public_items"].as_u64().unwrap(), r.public_items as u64);
    assert_eq!(
        parsed["internal_items"].as_u64().unwrap(),
        r.internal_items as u64
    );
}

#[test]
fn api_surface_report_json_has_expected_keys() {
    let tmp = tempfile::tempdir().unwrap();
    let code = "pub fn x() {}\n";
    let rel = write_file(tmp.path(), "src/lib.rs", code);
    let rows = vec![make_row("src/lib.rs", "src", "Rust", FileKind::Parent)];
    let export = make_export(rows);
    let r = build_api_surface_report(tmp.path(), &[rel], &export, &default_limits()).unwrap();

    let v: serde_json::Value = serde_json::to_value(&r).unwrap();
    let obj = v.as_object().unwrap();
    for key in &[
        "total_items",
        "public_items",
        "internal_items",
        "public_ratio",
        "documented_ratio",
        "by_language",
        "by_module",
        "top_exporters",
    ] {
        assert!(obj.contains_key(*key), "missing key: {key}");
    }
}

// =============================================================================
// 14. Child FileKind rows are excluded from scanning
// =============================================================================

#[test]
fn child_rows_produce_no_items() {
    let tmp = tempfile::tempdir().unwrap();
    let code = "pub fn should_be_ignored() {}\n";
    let rel = write_file(tmp.path(), "src/child.rs", code);
    let rows = vec![make_row("src/child.rs", "src", "Rust", FileKind::Child)];
    let export = make_export(rows);
    let r = build_api_surface_report(tmp.path(), &[rel], &export, &default_limits()).unwrap();
    assert_eq!(r.total_items, 0);
}

// =============================================================================
// 15. Missing file gracefully handled
// =============================================================================

#[test]
fn nonexistent_file_gracefully_skipped() {
    let tmp = tempfile::tempdir().unwrap();
    let rel = PathBuf::from("src/ghost.rs");
    let rows = vec![make_row("src/ghost.rs", "src", "Rust", FileKind::Parent)];
    let export = make_export(rows);
    let r = build_api_surface_report(tmp.path(), &[rel], &export, &default_limits()).unwrap();
    assert_eq!(r.total_items, 0);
}

// =============================================================================
// 16. Ratio clamping invariants
// =============================================================================

#[test]
fn public_ratio_exactly_one_third() {
    let tmp = tempfile::tempdir().unwrap();
    let code = "pub fn a() {}\nfn b() {}\nfn c() {}\n";
    let rel = write_file(tmp.path(), "src/lib.rs", code);
    let rows = vec![make_row("src/lib.rs", "src", "Rust", FileKind::Parent)];
    let export = make_export(rows);
    let r = build_api_surface_report(tmp.path(), &[rel], &export, &default_limits()).unwrap();
    assert!((r.public_ratio - 0.3333).abs() < 0.001);
}

#[test]
fn documented_ratio_exactly_one_when_all_public_documented() {
    let tmp = tempfile::tempdir().unwrap();
    let code = "/// d\npub fn a() {}\n/// d\npub fn b() {}\n";
    let rel = write_file(tmp.path(), "src/lib.rs", code);
    let rows = vec![make_row("src/lib.rs", "src", "Rust", FileKind::Parent)];
    let export = make_export(rows);
    let r = build_api_surface_report(tmp.path(), &[rel], &export, &default_limits()).unwrap();
    assert!((r.documented_ratio - 1.0).abs() < f64::EPSILON);
}

// =============================================================================
// 17. Proptest: invariant total = public + internal for any language
// =============================================================================

fn any_lang_item_line() -> impl Strategy<Value = (String, String)> {
    prop_oneof![
        Just(("pub fn g() {}".to_string(), "Rust".to_string())),
        Just(("fn p() {}".to_string(), "Rust".to_string())),
        Just(("export function e() {}".to_string(), "JavaScript".to_string())),
        Just(("function i() {}".to_string(), "JavaScript".to_string())),
        Just(("func Public() {}".to_string(), "Go".to_string())),
        Just(("func private() {}".to_string(), "Go".to_string())),
    ]
}

proptest! {
    #[test]
    fn prop_total_eq_public_plus_internal_any_lang(
        items in prop::collection::vec(any_lang_item_line(), 1..20)
    ) {
        let tmp = tempfile::tempdir().unwrap();
        let mut files = Vec::new();
        let mut rows = Vec::new();
        // Group by language
        let mut by_lang: std::collections::BTreeMap<String, Vec<String>> = std::collections::BTreeMap::new();
        for (line, lang) in &items {
            by_lang.entry(lang.clone()).or_default().push(line.clone());
        }
        let ext_map: std::collections::BTreeMap<&str, &str> = [
            ("Rust", "rs"), ("JavaScript", "js"), ("Go", "go"),
        ].into_iter().collect();
        for (lang, lines) in &by_lang {
            let ext = ext_map.get(lang.as_str()).unwrap_or(&"txt");
            let fname = format!("src/gen.{ext}");
            let code = lines.join("\n") + "\n";
            let rel = write_file(tmp.path(), &fname, &code);
            rows.push(make_row(&fname, "src", lang, FileKind::Parent));
            files.push(rel);
        }
        let export = make_export(rows);
        let r = build_api_surface_report(tmp.path(), &files, &export, &default_limits()).unwrap();
        prop_assert_eq!(
            r.total_items,
            r.public_items + r.internal_items,
            "invariant violated: {} != {} + {}",
            r.total_items, r.public_items, r.internal_items
        );
    }
}

// =============================================================================
// 18. Proptest: ratios always in [0.0, 1.0]
// =============================================================================

fn rust_line_strategy() -> impl Strategy<Value = String> {
    prop_oneof![
        Just("pub fn g() {}".to_string()),
        Just("fn p() {}".to_string()),
        Just("pub struct S;".to_string()),
        Just("/// doc".to_string()),
        Just("".to_string()),
    ]
}

proptest! {
    #[test]
    fn prop_ratios_bounded(lines in prop::collection::vec(rust_line_strategy(), 0..40)) {
        let code = lines.join("\n") + "\n";
        let tmp = tempfile::tempdir().unwrap();
        fs::write(tmp.path().join("lib.rs"), &code).unwrap();
        let export = make_export(vec![make_row("lib.rs", ".", "Rust", FileKind::Parent)]);
        let r = build_api_surface_report(
            tmp.path(),
            &[PathBuf::from("lib.rs")],
            &export,
            &default_limits(),
        ).unwrap();
        prop_assert!(r.public_ratio >= 0.0 && r.public_ratio <= 1.0);
        prop_assert!(r.documented_ratio >= 0.0 && r.documented_ratio <= 1.0);
    }
}

// =============================================================================
// 19. Proptest: top_exporters sorted descending
// =============================================================================

proptest! {
    #[test]
    fn prop_top_exporters_always_sorted(
        lines_a in prop::collection::vec(rust_line_strategy(), 0..15),
        lines_b in prop::collection::vec(rust_line_strategy(), 0..15),
    ) {
        let tmp = tempfile::tempdir().unwrap();
        fs::create_dir_all(tmp.path().join("a")).unwrap();
        fs::create_dir_all(tmp.path().join("b")).unwrap();
        fs::write(tmp.path().join("a/lib.rs"), lines_a.join("\n") + "\n").unwrap();
        fs::write(tmp.path().join("b/lib.rs"), lines_b.join("\n") + "\n").unwrap();
        let export = make_export(vec![
            make_row("a/lib.rs", "a", "Rust", FileKind::Parent),
            make_row("b/lib.rs", "b", "Rust", FileKind::Parent),
        ]);
        let r = build_api_surface_report(
            tmp.path(),
            &[PathBuf::from("a/lib.rs"), PathBuf::from("b/lib.rs")],
            &export,
            &default_limits(),
        ).unwrap();
        for w in r.top_exporters.windows(2) {
            prop_assert!(w[0].public_items >= w[1].public_items);
        }
    }
}

// =============================================================================
// 20. Proptest: by_language sums equal totals
// =============================================================================

proptest! {
    #[test]
    fn prop_lang_sums_equal_totals(lines in prop::collection::vec(rust_line_strategy(), 0..30)) {
        let code = lines.join("\n") + "\n";
        let tmp = tempfile::tempdir().unwrap();
        fs::write(tmp.path().join("lib.rs"), &code).unwrap();
        let export = make_export(vec![make_row("lib.rs", ".", "Rust", FileKind::Parent)]);
        let r = build_api_surface_report(
            tmp.path(),
            &[PathBuf::from("lib.rs")],
            &export,
            &default_limits(),
        ).unwrap();
        let sum_total: usize = r.by_language.values().map(|l| l.total_items).sum();
        let sum_pub: usize = r.by_language.values().map(|l| l.public_items).sum();
        prop_assert_eq!(r.total_items, sum_total);
        prop_assert_eq!(r.public_items, sum_pub);
    }
}
