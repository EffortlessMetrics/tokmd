//! Deep tests for `tokmd-analysis-api-surface`.
//!
//! Covers symbol extraction across all supported languages, documentation
//! detection, public/private boundary identification, report-level aggregation
//! invariants, serialization roundtrip, limit behavior, and determinism.

use std::fs;
use std::path::PathBuf;

use tokmd_analysis_api_surface::build_api_surface_report;
use tokmd_analysis_types::{ApiExportItem, ApiSurfaceReport, LangApiSurface, ModuleApiRow};
use tokmd_analysis_util::AnalysisLimits;
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
    let export = make_export(vec![make_row(filename, ".", lang)]);
    build_api_surface_report(dir.path(), &paths, &export, &default_limits()).unwrap()
}

// ═══════════════════════════════════════════════════════════════════
// § Rust symbol extraction – detailed
// ═══════════════════════════════════════════════════════════════════

mod rust_symbols {
    use super::*;

    #[test]
    fn pub_fn_detected_as_public() {
        let r = build_single_file("pub fn greet() {}\n", "lib.rs", "Rust");
        assert_eq!(r.public_items, 1);
        assert_eq!(r.internal_items, 0);
    }

    #[test]
    fn private_fn_detected_as_internal() {
        let r = build_single_file("fn helper() {}\n", "lib.rs", "Rust");
        assert_eq!(r.public_items, 0);
        assert_eq!(r.internal_items, 1);
    }

    #[test]
    fn pub_struct_enum_trait_all_public() {
        let code = "pub struct S;\npub enum E {}\npub trait T {}\n";
        let r = build_single_file(code, "lib.rs", "Rust");
        assert_eq!(r.public_items, 3);
        assert_eq!(r.internal_items, 0);
    }

    #[test]
    fn private_struct_enum_trait_all_internal() {
        let code = "struct S;\nenum E {}\ntrait T {}\n";
        let r = build_single_file(code, "lib.rs", "Rust");
        assert_eq!(r.public_items, 0);
        assert_eq!(r.internal_items, 3);
    }

    #[test]
    fn pub_type_const_static_mod() {
        let code =
            "pub type T = u32;\npub const C: u32 = 1;\npub static S: &str = \"hi\";\npub mod m;\n";
        let r = build_single_file(code, "lib.rs", "Rust");
        assert_eq!(r.public_items, 4);
    }

    #[test]
    fn pub_async_fn() {
        let r = build_single_file("pub async fn fetch() {}\n", "lib.rs", "Rust");
        assert_eq!(r.public_items, 1);
    }

    #[test]
    fn pub_unsafe_fn_and_trait() {
        let code = "pub unsafe fn danger() {}\npub unsafe trait UnsafeTrait {}\n";
        let r = build_single_file(code, "lib.rs", "Rust");
        assert_eq!(r.public_items, 2);
    }

    #[test]
    fn pub_crate_treated_as_public() {
        let r = build_single_file("pub(crate) fn internal() {}\n", "lib.rs", "Rust");
        assert_eq!(r.public_items, 1);
    }

    #[test]
    fn pub_super_treated_as_public() {
        let r = build_single_file("pub(super) fn scoped() {}\n", "lib.rs", "Rust");
        assert_eq!(r.public_items, 1);
    }

    #[test]
    fn pub_in_path_treated_as_public() {
        let r = build_single_file("pub(in crate::foo) fn scoped() {}\n", "lib.rs", "Rust");
        assert_eq!(r.public_items, 1);
    }

    #[test]
    fn documented_pub_fn_tracked() {
        let code = "/// Documentation here\npub fn documented() {}\n";
        let r = build_single_file(code, "lib.rs", "Rust");
        assert_eq!(r.public_items, 1);
        assert_eq!(r.documented_ratio, 1.0);
    }

    #[test]
    fn undocumented_pub_fn_ratio_zero() {
        let r = build_single_file("pub fn undocumented() {}\n", "lib.rs", "Rust");
        assert_eq!(r.public_items, 1);
        assert_eq!(r.documented_ratio, 0.0);
    }

    #[test]
    fn doc_bang_comment_detected() {
        let code = "//! Module doc\npub fn after_doc() {}\n";
        let r = build_single_file(code, "lib.rs", "Rust");
        assert_eq!(r.documented_ratio, 1.0);
    }

    #[test]
    fn doc_attribute_detected() {
        let code = "#[doc = \"documented\"]\npub fn attr_doc() {}\n";
        let r = build_single_file(code, "lib.rs", "Rust");
        assert_eq!(r.documented_ratio, 1.0);
    }

    #[test]
    fn mixed_pub_and_private() {
        let code = "pub fn a() {}\nfn b() {}\npub struct C;\nstruct D;\n";
        let r = build_single_file(code, "lib.rs", "Rust");
        assert_eq!(r.total_items, 4);
        assert_eq!(r.public_items, 2);
        assert_eq!(r.internal_items, 2);
        assert_eq!(r.public_ratio, 0.5);
    }

    #[test]
    fn comment_only_file_no_symbols() {
        let code = "// Just a comment\n/* block comment */\n";
        let r = build_single_file(code, "lib.rs", "Rust");
        assert_eq!(r.total_items, 0);
    }

    #[test]
    fn unmatched_pub_paren_no_panic() {
        let code = "pub(broken fn foo() {}\n";
        let r = build_single_file(code, "lib.rs", "Rust");
        // Should not panic; unmatched paren handled gracefully
        assert!(r.total_items <= 1);
    }
}

// ═══════════════════════════════════════════════════════════════════
// § JavaScript/TypeScript symbol extraction
// ═══════════════════════════════════════════════════════════════════

mod js_ts_symbols {
    use super::*;

    #[test]
    fn export_function_is_public() {
        let r = build_single_file("export function greet() {}\n", "index.js", "JavaScript");
        assert_eq!(r.public_items, 1);
    }

    #[test]
    fn export_class_is_public() {
        let r = build_single_file("export class MyClass {}\n", "mod.ts", "TypeScript");
        assert_eq!(r.public_items, 1);
    }

    #[test]
    fn export_const_let_are_public() {
        let code = "export const X = 1;\nexport let Y = 2;\n";
        let r = build_single_file(code, "mod.js", "JavaScript");
        assert_eq!(r.public_items, 2);
    }

    #[test]
    fn export_default_is_public() {
        let r = build_single_file(
            "export default function main() {}\n",
            "mod.js",
            "JavaScript",
        );
        assert_eq!(r.public_items, 1);
    }

    #[test]
    fn export_interface_type_enum() {
        let code = "export interface I {}\nexport type T = string;\nexport enum E { A }\n";
        let r = build_single_file(code, "types.ts", "TypeScript");
        assert_eq!(r.public_items, 3);
    }

    #[test]
    fn export_abstract_class() {
        let r = build_single_file("export abstract class Base {}\n", "base.ts", "TypeScript");
        assert_eq!(r.public_items, 1);
    }

    #[test]
    fn export_async_function() {
        let r = build_single_file("export async function fetch() {}\n", "api.ts", "TypeScript");
        assert_eq!(r.public_items, 1);
    }

    #[test]
    fn non_export_function_is_internal() {
        let r = build_single_file("function helper() {}\n", "mod.js", "JavaScript");
        assert_eq!(r.public_items, 0);
        assert_eq!(r.internal_items, 1);
    }

    #[test]
    fn non_export_class_const_let_are_internal() {
        let code = "class Internal {}\nconst X = 1;\nlet Y = 2;\n";
        let r = build_single_file(code, "mod.js", "JavaScript");
        assert_eq!(r.internal_items, 3);
    }

    #[test]
    fn async_function_internal() {
        let r = build_single_file("async function doWork() {}\n", "mod.js", "JavaScript");
        assert_eq!(r.internal_items, 1);
        assert_eq!(r.public_items, 0);
    }

    #[test]
    fn mixed_export_and_internal() {
        let code = "export function pub_fn() {}\nfunction priv_fn() {}\nexport class PubClass {}\nclass PrivClass {}\n";
        let r = build_single_file(code, "mod.ts", "TypeScript");
        assert_eq!(r.public_items, 2);
        assert_eq!(r.internal_items, 2);
        assert_eq!(r.public_ratio, 0.5);
    }

    #[test]
    fn documented_export_with_jsdoc() {
        let code = "/** JSDoc comment */\nexport function documented() {}\n";
        let r = build_single_file(code, "mod.js", "JavaScript");
        assert_eq!(r.public_items, 1);
        assert_eq!(r.documented_ratio, 1.0);
    }
}

// ═══════════════════════════════════════════════════════════════════
// § Python symbol extraction
// ═══════════════════════════════════════════════════════════════════

mod python_symbols {
    use super::*;

    #[test]
    fn public_def() {
        let r = build_single_file("def public_func():\n    pass\n", "mod.py", "Python");
        assert_eq!(r.public_items, 1);
    }

    #[test]
    fn private_def_underscore_prefix() {
        let r = build_single_file("def _private():\n    pass\n", "mod.py", "Python");
        assert_eq!(r.public_items, 0);
        assert_eq!(r.internal_items, 1);
    }

    #[test]
    fn dunder_private() {
        let r = build_single_file("def __dunder():\n    pass\n", "mod.py", "Python");
        assert_eq!(r.internal_items, 1);
    }

    #[test]
    fn public_class() {
        let r = build_single_file("class MyClass:\n    pass\n", "mod.py", "Python");
        assert_eq!(r.public_items, 1);
    }

    #[test]
    fn private_class() {
        let r = build_single_file("class _Internal:\n    pass\n", "mod.py", "Python");
        assert_eq!(r.internal_items, 1);
    }

    #[test]
    fn async_def_public() {
        let r = build_single_file("async def fetch():\n    pass\n", "mod.py", "Python");
        assert_eq!(r.public_items, 1);
    }

    #[test]
    fn async_def_private() {
        let r = build_single_file("async def _fetch():\n    pass\n", "mod.py", "Python");
        assert_eq!(r.internal_items, 1);
    }

    #[test]
    fn indented_method_not_counted() {
        let code = "class Foo:\n    def method(self):\n        pass\n";
        let r = build_single_file(code, "mod.py", "Python");
        // Only top-level class counted, not indented method
        assert_eq!(r.total_items, 1);
    }

    #[test]
    fn docstring_triple_double_quote() {
        let code = "def documented():\n    \"\"\"Has docstring.\"\"\"\n    pass\n";
        let r = build_single_file(code, "mod.py", "Python");
        assert_eq!(r.documented_ratio, 1.0);
    }

    #[test]
    fn docstring_triple_single_quote() {
        let code = "def documented():\n    '''Has docstring.'''\n    pass\n";
        let r = build_single_file(code, "mod.py", "Python");
        assert_eq!(r.documented_ratio, 1.0);
    }

    #[test]
    fn no_docstring_ratio_zero() {
        let code = "def undocumented():\n    pass\n";
        let r = build_single_file(code, "mod.py", "Python");
        assert_eq!(r.documented_ratio, 0.0);
    }

    #[test]
    fn hash_comments_not_counted_as_symbols() {
        let code = "# comment\ndef func():\n    pass\n";
        let r = build_single_file(code, "mod.py", "Python");
        assert_eq!(r.total_items, 1);
    }
}

// ═══════════════════════════════════════════════════════════════════
// § Go symbol extraction
// ═══════════════════════════════════════════════════════════════════

mod go_symbols {
    use super::*;

    #[test]
    fn uppercase_func_is_public() {
        let r = build_single_file("func PublicFunc() {}\n", "main.go", "Go");
        assert_eq!(r.public_items, 1);
    }

    #[test]
    fn lowercase_func_is_private() {
        let r = build_single_file("func privateFunc() {}\n", "main.go", "Go");
        assert_eq!(r.internal_items, 1);
    }

    #[test]
    fn uppercase_type_is_public() {
        let r = build_single_file("type MyStruct struct {}\n", "main.go", "Go");
        assert_eq!(r.public_items, 1);
    }

    #[test]
    fn lowercase_type_is_private() {
        let r = build_single_file("type myStruct struct {}\n", "main.go", "Go");
        assert_eq!(r.internal_items, 1);
    }

    #[test]
    fn method_receiver_public() {
        let r = build_single_file("func (s *Server) Handle() {}\n", "srv.go", "Go");
        assert_eq!(r.public_items, 1);
    }

    #[test]
    fn method_receiver_private() {
        let r = build_single_file("func (s *Server) handle() {}\n", "srv.go", "Go");
        assert_eq!(r.internal_items, 1);
    }

    #[test]
    fn var_and_const_visibility() {
        let code = "var PublicVar int = 42\nvar privateVar string\nconst MaxRetries = 3\nconst maxBuf = 64\n";
        let r = build_single_file(code, "main.go", "Go");
        assert_eq!(r.public_items, 2); // PublicVar, MaxRetries
        assert_eq!(r.internal_items, 2); // privateVar, maxBuf
    }

    #[test]
    fn interface_type_public() {
        let r = build_single_file("type Handler interface {}\n", "main.go", "Go");
        assert_eq!(r.public_items, 1);
    }

    #[test]
    fn documented_go_func() {
        let code = "// PublicFunc does something.\nfunc PublicFunc() {}\n";
        let r = build_single_file(code, "main.go", "Go");
        assert_eq!(r.documented_ratio, 1.0);
    }
}

// ═══════════════════════════════════════════════════════════════════
// § Java symbol extraction
// ═══════════════════════════════════════════════════════════════════

mod java_symbols {
    use super::*;

    #[test]
    fn public_class() {
        let r = build_single_file("public class App {}\n", "App.java", "Java");
        assert_eq!(r.public_items, 1);
    }

    #[test]
    fn public_interface() {
        let r = build_single_file("public interface Service {}\n", "Service.java", "Java");
        assert_eq!(r.public_items, 1);
    }

    #[test]
    fn public_enum() {
        let r = build_single_file("public enum Color { RED, GREEN }\n", "Color.java", "Java");
        assert_eq!(r.public_items, 1);
    }

    #[test]
    fn public_abstract_class() {
        let r = build_single_file("public abstract class Base {}\n", "Base.java", "Java");
        assert_eq!(r.public_items, 1);
    }

    #[test]
    fn public_final_class() {
        let r = build_single_file(
            "public final class Immutable {}\n",
            "Immutable.java",
            "Java",
        );
        assert_eq!(r.public_items, 1);
    }

    #[test]
    fn public_record() {
        let r = build_single_file(
            "public record Point(int x, int y) {}\n",
            "Point.java",
            "Java",
        );
        assert_eq!(r.public_items, 1);
    }

    #[test]
    fn public_sealed_class() {
        let r = build_single_file("public sealed class Shape {}\n", "Shape.java", "Java");
        assert_eq!(r.public_items, 1);
    }

    #[test]
    fn public_static_method() {
        let r = build_single_file(
            "public static void main(String[] args) {}\n",
            "App.java",
            "Java",
        );
        assert_eq!(r.public_items, 1);
    }

    #[test]
    fn package_private_class() {
        let r = build_single_file("class Internal {}\n", "Internal.java", "Java");
        assert_eq!(r.internal_items, 1);
    }

    #[test]
    fn private_method() {
        let r = build_single_file("private void helper() {}\n", "App.java", "Java");
        assert_eq!(r.internal_items, 1);
    }

    #[test]
    fn protected_method() {
        let r = build_single_file("protected void helper() {}\n", "App.java", "Java");
        assert_eq!(r.internal_items, 1);
    }

    #[test]
    fn javadoc_detected() {
        let code = "/** Javadoc */\npublic class Documented {}\n";
        let r = build_single_file(code, "App.java", "Java");
        assert_eq!(r.documented_ratio, 1.0);
    }

    #[test]
    fn internal_record() {
        let r = build_single_file("record Internal(String s) {}\n", "App.java", "Java");
        assert_eq!(r.internal_items, 1);
    }
}

// ═══════════════════════════════════════════════════════════════════
// § Edge cases
// ═══════════════════════════════════════════════════════════════════

mod edge_cases {
    use super::*;

    #[test]
    fn empty_file_yields_empty_report() {
        let r = build_single_file("", "lib.rs", "Rust");
        assert_eq!(r.total_items, 0);
        assert_eq!(r.public_ratio, 0.0);
        assert_eq!(r.documented_ratio, 0.0);
    }

    #[test]
    fn whitespace_only_file() {
        let r = build_single_file("   \n  \n\t\n", "lib.rs", "Rust");
        assert_eq!(r.total_items, 0);
    }

    #[test]
    fn unsupported_language_skipped() {
        let r = build_single_file("some code here\n", "file.md", "Markdown");
        assert_eq!(r.total_items, 0);
        assert!(r.by_language.is_empty());
    }

    #[test]
    fn nonexistent_file_gracefully_skipped() {
        let dir = tempfile::tempdir().unwrap();
        let paths = vec![PathBuf::from("nonexistent.rs")];
        let export = make_export(vec![make_row("nonexistent.rs", ".", "Rust")]);
        let r = build_api_surface_report(dir.path(), &paths, &export, &default_limits()).unwrap();
        assert_eq!(r.total_items, 0);
    }

    #[test]
    fn child_kind_rows_excluded() {
        let (dir, paths) = write_temp_files(&[("lib.rs", "pub fn visible() {}\n")]);
        let mut row = make_row("lib.rs", ".", "Rust");
        row.kind = FileKind::Child;
        let export = make_export(vec![row]);
        let r = build_api_surface_report(dir.path(), &paths, &export, &default_limits()).unwrap();
        assert_eq!(r.total_items, 0);
    }

    #[test]
    fn no_export_rows_yields_empty() {
        let (dir, paths) = write_temp_files(&[("lib.rs", "pub fn f() {}\n")]);
        let export = make_export(vec![]);
        let r = build_api_surface_report(dir.path(), &paths, &export, &default_limits()).unwrap();
        assert_eq!(r.total_items, 0);
    }

    #[test]
    fn empty_file_list_yields_empty() {
        let dir = tempfile::tempdir().unwrap();
        let export = make_export(vec![]);
        let r = build_api_surface_report(dir.path(), &[], &export, &default_limits()).unwrap();
        assert_eq!(r.total_items, 0);
        assert!(r.by_language.is_empty());
        assert!(r.by_module.is_empty());
        assert!(r.top_exporters.is_empty());
    }

    #[test]
    fn all_languages_empty_input_no_symbols() {
        for lang in &["Rust", "JavaScript", "TypeScript", "Python", "Go", "Java"] {
            let ext = match *lang {
                "Rust" => "rs",
                "JavaScript" => "js",
                "TypeScript" => "ts",
                "Python" => "py",
                "Go" => "go",
                "Java" => "java",
                _ => "txt",
            };
            let filename = format!("empty.{ext}");
            let r = build_single_file("", &filename, lang);
            assert_eq!(r.total_items, 0, "empty {lang} file should yield 0 items");
        }
    }
}

// ═══════════════════════════════════════════════════════════════════
// § Report-level aggregation invariants
// ═══════════════════════════════════════════════════════════════════

mod aggregation_invariants {
    use super::*;

    fn verify_report_invariants(r: &ApiSurfaceReport) {
        // total = public + internal
        assert_eq!(
            r.total_items,
            r.public_items + r.internal_items,
            "total should equal public + internal"
        );
        // public_ratio consistency
        if r.total_items > 0 {
            let expected_ratio = r.public_items as f64 / r.total_items as f64;
            assert!(
                (r.public_ratio - expected_ratio).abs() < 0.001,
                "public_ratio mismatch"
            );
        } else {
            assert_eq!(r.public_ratio, 0.0);
        }
        // by_language sums match totals
        let lang_total: usize = r.by_language.values().map(|l| l.total_items).sum();
        let lang_public: usize = r.by_language.values().map(|l| l.public_items).sum();
        let lang_internal: usize = r.by_language.values().map(|l| l.internal_items).sum();
        assert_eq!(r.total_items, lang_total, "lang totals should match");
        assert_eq!(r.public_items, lang_public, "lang public should match");
        assert_eq!(
            r.internal_items, lang_internal,
            "lang internal should match"
        );
        // per-language ratio consistency
        for (lang, surface) in &r.by_language {
            assert_eq!(
                surface.total_items,
                surface.public_items + surface.internal_items,
                "lang {lang} total should equal public + internal"
            );
        }
    }

    #[test]
    fn rust_only_invariants() {
        let code = "pub fn a() {}\nfn b() {}\npub struct C;\nstruct D;\n";
        let r = build_single_file(code, "lib.rs", "Rust");
        verify_report_invariants(&r);
    }

    #[test]
    fn python_only_invariants() {
        let code = "def public():\n    pass\ndef _private():\n    pass\nclass MyClass:\n    pass\n";
        let r = build_single_file(code, "mod.py", "Python");
        verify_report_invariants(&r);
    }

    #[test]
    fn multi_language_invariants() {
        let rust = "pub fn r() {}\nfn ri() {}\n";
        let js = "export function j() {}\nfunction ji() {}\n";
        let py = "def p():\n    pass\ndef _pi():\n    pass\n";
        let go = "func G() {}\nfunc g() {}\n";
        let java = "public class J {}\nclass ji {}\n";
        let ts = "export interface TI {}\ninterface ti {}\n";

        let (dir, paths) = write_temp_files(&[
            ("lib.rs", rust),
            ("index.js", js),
            ("mod.py", py),
            ("main.go", go),
            ("App.java", java),
            ("types.ts", ts),
        ]);
        let export = make_export(vec![
            make_row("lib.rs", "rust_mod", "Rust"),
            make_row("index.js", "js_mod", "JavaScript"),
            make_row("mod.py", "py_mod", "Python"),
            make_row("main.go", "go_mod", "Go"),
            make_row("App.java", "java_mod", "Java"),
            make_row("types.ts", "ts_mod", "TypeScript"),
        ]);
        let r = build_api_surface_report(dir.path(), &paths, &export, &default_limits()).unwrap();
        verify_report_invariants(&r);
        assert_eq!(r.by_language.len(), 6);
    }

    #[test]
    fn empty_report_invariants() {
        let dir = tempfile::tempdir().unwrap();
        let export = make_export(vec![]);
        let r = build_api_surface_report(dir.path(), &[], &export, &default_limits()).unwrap();
        verify_report_invariants(&r);
    }
}

// ═══════════════════════════════════════════════════════════════════
// § Sorting and capping
// ═══════════════════════════════════════════════════════════════════

mod sorting_and_capping {
    use super::*;

    #[test]
    fn top_exporters_sorted_by_public_items_desc() {
        let few = "pub fn a() {}\n";
        let many = "pub fn x() {}\npub fn y() {}\npub fn z() {}\n";
        let (dir, paths) = write_temp_files(&[("few.rs", few), ("many.rs", many)]);
        let export = make_export(vec![
            make_row("few.rs", ".", "Rust"),
            make_row("many.rs", ".", "Rust"),
        ]);
        let r = build_api_surface_report(dir.path(), &paths, &export, &default_limits()).unwrap();
        assert_eq!(r.top_exporters[0].path, "many.rs");
        assert_eq!(r.top_exporters[0].public_items, 3);
    }

    #[test]
    fn top_exporters_tiebreak_by_path() {
        let code = "pub fn f() {}\n";
        let (dir, paths) = write_temp_files(&[("b.rs", code), ("a.rs", code)]);
        let export = make_export(vec![
            make_row("b.rs", ".", "Rust"),
            make_row("a.rs", ".", "Rust"),
        ]);
        let r = build_api_surface_report(dir.path(), &paths, &export, &default_limits()).unwrap();
        assert_eq!(r.top_exporters[0].path, "a.rs");
        assert_eq!(r.top_exporters[1].path, "b.rs");
    }

    #[test]
    fn top_exporters_capped_at_20() {
        let code = "pub fn f() {}\n";
        let dir = tempfile::tempdir().unwrap();
        let mut paths = Vec::new();
        let mut rows = Vec::new();
        for i in 0..25 {
            let name = format!("mod{i}.rs");
            fs::write(dir.path().join(&name), code).unwrap();
            paths.push(PathBuf::from(&name));
            rows.push(make_row(&name, ".", "Rust"));
        }
        let export = make_export(rows);
        let r = build_api_surface_report(dir.path(), &paths, &export, &default_limits()).unwrap();
        assert!(r.top_exporters.len() <= 20);
    }

    #[test]
    fn by_module_sorted_by_total_desc() {
        let one = "pub fn a() {}\n";
        let three = "pub fn x() {}\npub fn y() {}\npub fn z() {}\n";
        let (dir, paths) = write_temp_files(&[("a/lib.rs", one), ("b/lib.rs", three)]);
        let export = make_export(vec![
            make_row("a/lib.rs", "mod_a", "Rust"),
            make_row("b/lib.rs", "mod_b", "Rust"),
        ]);
        let r = build_api_surface_report(dir.path(), &paths, &export, &default_limits()).unwrap();
        assert_eq!(r.by_module[0].module, "mod_b");
        assert_eq!(r.by_module[1].module, "mod_a");
    }

    #[test]
    fn by_module_tiebreak_by_name() {
        let code = "pub fn f() {}\n";
        let (dir, paths) = write_temp_files(&[("b/lib.rs", code), ("a/lib.rs", code)]);
        let export = make_export(vec![
            make_row("b/lib.rs", "mod_b", "Rust"),
            make_row("a/lib.rs", "mod_a", "Rust"),
        ]);
        let r = build_api_surface_report(dir.path(), &paths, &export, &default_limits()).unwrap();
        assert_eq!(r.by_module[0].module, "mod_a");
        assert_eq!(r.by_module[1].module, "mod_b");
    }

    #[test]
    fn by_module_capped_at_50() {
        let code = "pub fn f() {}\n";
        let dir = tempfile::tempdir().unwrap();
        let mut paths = Vec::new();
        let mut rows = Vec::new();
        for i in 0..55 {
            let name = format!("mod{i}/lib.rs");
            let module = format!("mod{i}");
            let full = dir.path().join(&name);
            fs::create_dir_all(full.parent().unwrap()).unwrap();
            fs::write(&full, code).unwrap();
            paths.push(PathBuf::from(&name));
            rows.push(make_row(&name, &module, "Rust"));
        }
        let export = make_export(rows);
        let r = build_api_surface_report(dir.path(), &paths, &export, &default_limits()).unwrap();
        assert!(r.by_module.len() <= 50);
    }

    #[test]
    fn no_public_items_not_in_top_exporters() {
        let r = build_single_file("fn internal() {}\n", "lib.rs", "Rust");
        assert!(r.top_exporters.is_empty());
    }
}

// ═══════════════════════════════════════════════════════════════════
// § Limit behavior
// ═══════════════════════════════════════════════════════════════════

mod limits {
    use super::*;

    #[test]
    fn max_bytes_stops_scanning() {
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
        let r = build_api_surface_report(dir.path(), &paths, &export, &limits).unwrap();
        assert!(
            r.total_items <= 1,
            "expected at most 1 item after budget, got {}",
            r.total_items
        );
    }

    #[test]
    fn max_file_bytes_truncates_large_file() {
        let mut code = String::new();
        for _ in 0..200 {
            code.push_str("// padding\n");
        }
        code.push_str("pub fn hidden() {}\n");
        let (dir, paths) = write_temp_files(&[("big.rs", &code)]);
        let export = make_export(vec![make_row("big.rs", ".", "Rust")]);
        let limits = AnalysisLimits {
            max_file_bytes: Some(100),
            ..Default::default()
        };
        let r = build_api_surface_report(dir.path(), &paths, &export, &limits).unwrap();
        assert_eq!(
            r.public_items, 0,
            "symbol beyond truncation point not found"
        );
    }
}

// ═══════════════════════════════════════════════════════════════════
// § Serialization roundtrip
// ═══════════════════════════════════════════════════════════════════

mod serialization {
    use super::*;

    #[test]
    fn report_json_roundtrip() {
        let code = "pub fn a() {}\nfn b() {}\n/// Doc\npub fn c() {}\n";
        let r = build_single_file(code, "lib.rs", "Rust");
        let json = serde_json::to_string(&r).unwrap();
        let deserialized: ApiSurfaceReport = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.total_items, r.total_items);
        assert_eq!(deserialized.public_items, r.public_items);
        assert_eq!(deserialized.internal_items, r.internal_items);
        assert_eq!(deserialized.public_ratio, r.public_ratio);
        assert_eq!(deserialized.documented_ratio, r.documented_ratio);
        assert_eq!(deserialized.by_language.len(), r.by_language.len());
        assert_eq!(deserialized.by_module.len(), r.by_module.len());
        assert_eq!(deserialized.top_exporters.len(), r.top_exporters.len());
    }

    #[test]
    fn empty_report_json_roundtrip() {
        let dir = tempfile::tempdir().unwrap();
        let export = make_export(vec![]);
        let r = build_api_surface_report(dir.path(), &[], &export, &default_limits()).unwrap();
        let json = serde_json::to_string(&r).unwrap();
        let deserialized: ApiSurfaceReport = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.total_items, 0);
    }

    #[test]
    fn json_contains_expected_fields() {
        let r = build_single_file("pub fn f() {}\n", "lib.rs", "Rust");
        let json = serde_json::to_string(&r).unwrap();
        for field in &[
            "total_items",
            "public_items",
            "internal_items",
            "public_ratio",
            "documented_ratio",
            "by_language",
            "by_module",
            "top_exporters",
        ] {
            assert!(json.contains(field), "JSON should contain '{field}'");
        }
    }

    #[test]
    fn lang_api_surface_roundtrip() {
        let surface = LangApiSurface {
            total_items: 5,
            public_items: 3,
            internal_items: 2,
            public_ratio: 0.6,
        };
        let json = serde_json::to_string(&surface).unwrap();
        let deserialized: LangApiSurface = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.total_items, 5);
        assert_eq!(deserialized.public_ratio, 0.6);
    }

    #[test]
    fn module_api_row_roundtrip() {
        let row = ModuleApiRow {
            module: "src".to_string(),
            total_items: 10,
            public_items: 7,
            public_ratio: 0.7,
        };
        let json = serde_json::to_string(&row).unwrap();
        let deserialized: ModuleApiRow = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.module, "src");
        assert_eq!(deserialized.public_ratio, 0.7);
    }

    #[test]
    fn api_export_item_roundtrip() {
        let item = ApiExportItem {
            path: "lib.rs".to_string(),
            lang: "Rust".to_string(),
            public_items: 3,
            total_items: 5,
        };
        let json = serde_json::to_string(&item).unwrap();
        let deserialized: ApiExportItem = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.path, "lib.rs");
        assert_eq!(deserialized.public_items, 3);
    }
}

// ═══════════════════════════════════════════════════════════════════
// § Determinism
// ═══════════════════════════════════════════════════════════════════

mod determinism {
    use super::*;

    #[test]
    fn ten_runs_identical_report() {
        let code = "pub fn a() {}\nfn b() {}\npub struct C;\n/// Doc\npub fn d() {}\n";
        let (dir, paths) = write_temp_files(&[("lib.rs", code)]);
        let export = make_export(vec![make_row("lib.rs", ".", "Rust")]);

        let first =
            build_api_surface_report(dir.path(), &paths, &export, &default_limits()).unwrap();
        for _ in 0..9 {
            let run =
                build_api_surface_report(dir.path(), &paths, &export, &default_limits()).unwrap();
            assert_eq!(first.total_items, run.total_items);
            assert_eq!(first.public_items, run.public_items);
            assert_eq!(first.internal_items, run.internal_items);
            assert_eq!(first.public_ratio, run.public_ratio);
            assert_eq!(first.documented_ratio, run.documented_ratio);
            assert_eq!(first.by_language.len(), run.by_language.len());
            assert_eq!(first.by_module.len(), run.by_module.len());
            assert_eq!(first.top_exporters.len(), run.top_exporters.len());
        }
    }

    #[test]
    fn multi_language_determinism() {
        let rust = "pub fn r() {}\nfn ri() {}\n";
        let py = "def p():\n    pass\ndef _pi():\n    pass\n";
        let js = "export function j() {}\nfunction ji() {}\n";
        let (dir, paths) = write_temp_files(&[("lib.rs", rust), ("mod.py", py), ("index.js", js)]);
        let export = make_export(vec![
            make_row("lib.rs", "rust_mod", "Rust"),
            make_row("mod.py", "py_mod", "Python"),
            make_row("index.js", "js_mod", "JavaScript"),
        ]);

        let first =
            build_api_surface_report(dir.path(), &paths, &export, &default_limits()).unwrap();
        let second =
            build_api_surface_report(dir.path(), &paths, &export, &default_limits()).unwrap();

        let json1 = serde_json::to_string(&first).unwrap();
        let json2 = serde_json::to_string(&second).unwrap();
        assert_eq!(json1, json2, "JSON serialization should be identical");
    }
}

// ═══════════════════════════════════════════════════════════════════
// § Multi-module aggregation
// ═══════════════════════════════════════════════════════════════════

mod multi_module {
    use super::*;

    #[test]
    fn same_module_accumulates() {
        let a = "pub fn a() {}\nfn b() {}\n";
        let c = "pub fn c() {}\npub fn d() {}\n";
        let (dir, paths) = write_temp_files(&[("src/a.rs", a), ("src/b.rs", c)]);
        let export = make_export(vec![
            make_row("src/a.rs", "src", "Rust"),
            make_row("src/b.rs", "src", "Rust"),
        ]);
        let r = build_api_surface_report(dir.path(), &paths, &export, &default_limits()).unwrap();
        assert_eq!(r.by_module.len(), 1);
        assert_eq!(r.by_module[0].module, "src");
        assert_eq!(r.by_module[0].total_items, 4);
        assert_eq!(r.by_module[0].public_items, 3);
    }

    #[test]
    fn different_modules_separate() {
        let a = "pub fn a() {}\n";
        let b = "pub fn b() {}\n";
        let (dir, paths) = write_temp_files(&[("src/a.rs", a), ("lib/b.rs", b)]);
        let export = make_export(vec![
            make_row("src/a.rs", "mod_src", "Rust"),
            make_row("lib/b.rs", "mod_lib", "Rust"),
        ]);
        let r = build_api_surface_report(dir.path(), &paths, &export, &default_limits()).unwrap();
        assert_eq!(r.by_module.len(), 2);
    }

    #[test]
    fn module_public_ratio_correct() {
        let code = "pub fn a() {}\nfn b() {}\n";
        let (dir, paths) = write_temp_files(&[("lib.rs", code)]);
        let export = make_export(vec![make_row("lib.rs", "root", "Rust")]);
        let r = build_api_surface_report(dir.path(), &paths, &export, &default_limits()).unwrap();
        assert_eq!(r.by_module[0].public_ratio, 0.5);
    }
}
