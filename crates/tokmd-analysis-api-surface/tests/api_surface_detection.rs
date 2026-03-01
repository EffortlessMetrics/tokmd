//! Deeper detection tests for `tokmd-analysis-api-surface`.
//!
//! Tests edge cases: empty files, files with no public APIs, mixed visibility,
//! comment-heavy files, multiple item types per language, and doc-comment variants.

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
// Empty and whitespace-only files
// ---------------------------------------------------------------------------

#[test]
fn whitespace_only_file_yields_no_symbols() {
    let (dir, paths) = write_temp_files(&[("lib.rs", "   \n\n  \t\n")]);
    let export = make_export(vec![make_row("lib.rs", ".", "Rust")]);
    let report = build_api_surface_report(dir.path(), &paths, &export, &default_limits()).unwrap();
    assert_eq!(report.total_items, 0);
}

#[test]
fn single_newline_file_yields_no_symbols() {
    let (dir, paths) = write_temp_files(&[("lib.rs", "\n")]);
    let export = make_export(vec![make_row("lib.rs", ".", "Rust")]);
    let report = build_api_surface_report(dir.path(), &paths, &export, &default_limits()).unwrap();
    assert_eq!(report.total_items, 0);
}

// ---------------------------------------------------------------------------
// Rust: mixed item types and visibility
// ---------------------------------------------------------------------------

#[test]
fn rust_all_item_types_public() {
    let code = "\
pub fn f() {}
pub struct S;
pub enum E {}
pub trait T {}
pub type Alias = u32;
pub const C: u32 = 0;
pub static ST: u32 = 0;
pub mod m;
pub async fn af() {}
pub unsafe fn uf() {}
pub unsafe trait UT {}
";
    let (dir, paths) = write_temp_files(&[("lib.rs", code)]);
    let export = make_export(vec![make_row("lib.rs", ".", "Rust")]);
    let report = build_api_surface_report(dir.path(), &paths, &export, &default_limits()).unwrap();

    assert_eq!(report.public_items, 11);
    assert_eq!(report.internal_items, 0);
    assert_eq!(report.public_ratio, 1.0);
}

#[test]
fn rust_all_item_types_internal() {
    let code = "\
fn f() {}
struct S;
enum E {}
trait T {}
type Alias = u32;
const C: u32 = 0;
static ST: u32 = 0;
mod m;
async fn af() {}
unsafe fn uf() {}
unsafe trait UT {}
";
    let (dir, paths) = write_temp_files(&[("lib.rs", code)]);
    let export = make_export(vec![make_row("lib.rs", ".", "Rust")]);
    let report = build_api_surface_report(dir.path(), &paths, &export, &default_limits()).unwrap();

    assert_eq!(report.public_items, 0);
    assert_eq!(report.internal_items, 11);
    assert_eq!(report.public_ratio, 0.0);
}

#[test]
fn rust_pub_crate_and_pub_super_counted_as_public() {
    let code = "\
pub(crate) fn crate_fn() {}
pub(super) struct SuperStruct;
pub(in crate::module) enum ScopedEnum {}
";
    let (dir, paths) = write_temp_files(&[("lib.rs", code)]);
    let export = make_export(vec![make_row("lib.rs", ".", "Rust")]);
    let report = build_api_surface_report(dir.path(), &paths, &export, &default_limits()).unwrap();

    assert_eq!(report.public_items, 3);
}

#[test]
fn rust_doc_comment_variants() {
    let code = "\
/// Triple-slash doc
pub fn doc1() {}
//! Inner doc
pub fn doc2() {}
#[doc = \"attr doc\"]
pub fn doc3() {}
pub fn undoc() {}
";
    let (dir, paths) = write_temp_files(&[("lib.rs", code)]);
    let export = make_export(vec![make_row("lib.rs", ".", "Rust")]);
    let report = build_api_surface_report(dir.path(), &paths, &export, &default_limits()).unwrap();

    assert_eq!(report.public_items, 4);
    // doc1, doc2, doc3 are documented; undoc is not
    assert_eq!(report.documented_ratio, 0.75);
}

#[test]
fn rust_comments_not_counted_as_symbols() {
    let code = "\
// This is a comment
/* block comment */
/// doc comment without item following it
// another comment
";
    let (dir, paths) = write_temp_files(&[("lib.rs", code)]);
    let export = make_export(vec![make_row("lib.rs", ".", "Rust")]);
    let report = build_api_surface_report(dir.path(), &paths, &export, &default_limits()).unwrap();

    assert_eq!(report.total_items, 0);
}

// ---------------------------------------------------------------------------
// JavaScript / TypeScript: edge cases
// ---------------------------------------------------------------------------

#[test]
fn js_all_export_variants() {
    let code = "\
export function f() {}
export async function af() {}
export class C {}
export const x = 1;
export let y = 2;
export default function main() {}
";
    let (dir, paths) = write_temp_files(&[("index.js", code)]);
    let export = make_export(vec![make_row("index.js", ".", "JavaScript")]);
    let report = build_api_surface_report(dir.path(), &paths, &export, &default_limits()).unwrap();

    assert_eq!(report.public_items, 6);
}

#[test]
fn ts_all_export_variants() {
    let code = "\
export interface IFoo {}
export type Bar = string;
export enum Baz { A, B }
export abstract class AC {}
export function tf() {}
";
    let (dir, paths) = write_temp_files(&[("types.ts", code)]);
    let export = make_export(vec![make_row("types.ts", ".", "TypeScript")]);
    let report = build_api_surface_report(dir.path(), &paths, &export, &default_limits()).unwrap();

    assert_eq!(report.public_items, 5);
}

#[test]
fn js_only_internal_no_exports() {
    let code = "\
function internal1() {}
async function internal2() {}
class InternalClass {}
const x = 1;
let y = 2;
interface Local {}
type T = string;
enum E {}
";
    let (dir, paths) = write_temp_files(&[("util.js", code)]);
    let export = make_export(vec![make_row("util.js", ".", "JavaScript")]);
    let report = build_api_surface_report(dir.path(), &paths, &export, &default_limits()).unwrap();

    assert_eq!(report.public_items, 0);
    assert_eq!(report.internal_items, 8);
}

#[test]
fn js_documented_export() {
    let code = "\
/** JSDoc comment */
export function documented() {}
export function undocumented() {}
";
    let (dir, paths) = write_temp_files(&[("index.js", code)]);
    let export = make_export(vec![make_row("index.js", ".", "JavaScript")]);
    let report = build_api_surface_report(dir.path(), &paths, &export, &default_limits()).unwrap();

    assert_eq!(report.public_items, 2);
    assert_eq!(report.documented_ratio, 0.5);
}

// ---------------------------------------------------------------------------
// Python: edge cases
// ---------------------------------------------------------------------------

#[test]
fn python_async_def_public_and_private() {
    let code = "\
async def public_async():
    pass
async def _private_async():
    pass
";
    let (dir, paths) = write_temp_files(&[("mod.py", code)]);
    let export = make_export(vec![make_row("mod.py", ".", "Python")]);
    let report = build_api_surface_report(dir.path(), &paths, &export, &default_limits()).unwrap();

    assert_eq!(report.public_items, 1);
    assert_eq!(report.internal_items, 1);
}

#[test]
fn python_dunder_methods_are_private() {
    let code = "\
def __init__():
    pass
def __repr__():
    pass
";
    let (dir, paths) = write_temp_files(&[("mod.py", code)]);
    let export = make_export(vec![make_row("mod.py", ".", "Python")]);
    let report = build_api_surface_report(dir.path(), &paths, &export, &default_limits()).unwrap();

    // __init__ and __repr__ start with '_', so they're private
    assert_eq!(report.public_items, 0);
    assert_eq!(report.internal_items, 2);
}

#[test]
fn python_triple_quote_variants() {
    let code = "\
def with_double_triple():
    \"\"\"Double triple-quote docstring.\"\"\"
    pass
def with_single_triple():
    '''Single triple-quote docstring.'''
    pass
def without_doc():
    pass
";
    let (dir, paths) = write_temp_files(&[("mod.py", code)]);
    let export = make_export(vec![make_row("mod.py", ".", "Python")]);
    let report = build_api_surface_report(dir.path(), &paths, &export, &default_limits()).unwrap();

    assert_eq!(report.public_items, 3);
    // 2 documented, 1 not
    let expected_ratio = 0.6667;
    assert!(
        (report.documented_ratio - expected_ratio).abs() < 0.001,
        "expected ~{expected_ratio}, got {}",
        report.documented_ratio
    );
}

#[test]
fn python_comments_and_blank_lines_only() {
    let code = "\
# comment 1
# comment 2

# another comment
";
    let (dir, paths) = write_temp_files(&[("mod.py", code)]);
    let export = make_export(vec![make_row("mod.py", ".", "Python")]);
    let report = build_api_surface_report(dir.path(), &paths, &export, &default_limits()).unwrap();

    assert_eq!(report.total_items, 0);
}

// ---------------------------------------------------------------------------
// Go: edge cases
// ---------------------------------------------------------------------------

#[test]
fn go_var_and_const_visibility() {
    let code = "\
var PublicVar int = 1
var privateVar int = 2
const PublicConst = 42
const privateConst = 99
";
    let (dir, paths) = write_temp_files(&[("main.go", code)]);
    let export = make_export(vec![make_row("main.go", ".", "Go")]);
    let report = build_api_surface_report(dir.path(), &paths, &export, &default_limits()).unwrap();

    assert_eq!(report.public_items, 2);
    assert_eq!(report.internal_items, 2);
}

#[test]
fn go_type_interface() {
    let code = "\
type PublicInterface interface {}
type privateInterface interface {}
type PublicStruct struct {}
type privateStruct struct {}
";
    let (dir, paths) = write_temp_files(&[("types.go", code)]);
    let export = make_export(vec![make_row("types.go", ".", "Go")]);
    let report = build_api_surface_report(dir.path(), &paths, &export, &default_limits()).unwrap();

    assert_eq!(report.public_items, 2);
    assert_eq!(report.internal_items, 2);
}

#[test]
fn go_documented_func() {
    let code = "\
// PublicFunc does something.
func PublicFunc() {}
func UndocumentedFunc() {}
";
    let (dir, paths) = write_temp_files(&[("main.go", code)]);
    let export = make_export(vec![make_row("main.go", ".", "Go")]);
    let report = build_api_surface_report(dir.path(), &paths, &export, &default_limits()).unwrap();

    assert_eq!(report.public_items, 2);
    assert_eq!(report.documented_ratio, 0.5);
}

#[test]
fn go_comment_only_file() {
    let code = "\
// Package comment
// More comments
/* Block comment */
";
    let (dir, paths) = write_temp_files(&[("doc.go", code)]);
    let export = make_export(vec![make_row("doc.go", ".", "Go")]);
    let report = build_api_surface_report(dir.path(), &paths, &export, &default_limits()).unwrap();

    assert_eq!(report.total_items, 0);
}

// ---------------------------------------------------------------------------
// Java: edge cases
// ---------------------------------------------------------------------------

#[test]
fn java_all_visibility_modifiers() {
    let code = "\
public class PubClass {}
protected class ProtClass {}
private class PrivClass {}
class PkgClass {}
public interface PubIface {}
public enum PubEnum { A }
public abstract class AbsClass {}
public final class FinClass {}
public record PubRecord() {}
public sealed class SealedClass {}
";
    let (dir, paths) = write_temp_files(&[("Mixed.java", code)]);
    let export = make_export(vec![make_row("Mixed.java", ".", "Java")]);
    let report = build_api_surface_report(dir.path(), &paths, &export, &default_limits()).unwrap();

    assert_eq!(report.public_items, 7);
    assert_eq!(report.internal_items, 3);
}

#[test]
fn java_public_method_with_return_type() {
    let code = "\
public String getName() {}
public void doStuff() {}
private int helper() {}
";
    let (dir, paths) = write_temp_files(&[("Svc.java", code)]);
    let export = make_export(vec![make_row("Svc.java", ".", "Java")]);
    let report = build_api_surface_report(dir.path(), &paths, &export, &default_limits()).unwrap();

    assert_eq!(report.public_items, 2);
    assert_eq!(report.internal_items, 1);
}

#[test]
fn java_javadoc_detected() {
    let code = "\
/** Javadoc for class */
public class Documented {}
public class Undocumented {}
";
    let (dir, paths) = write_temp_files(&[("Doc.java", code)]);
    let export = make_export(vec![make_row("Doc.java", ".", "Java")]);
    let report = build_api_surface_report(dir.path(), &paths, &export, &default_limits()).unwrap();

    assert_eq!(report.public_items, 2);
    assert_eq!(report.documented_ratio, 0.5);
}

// ---------------------------------------------------------------------------
// Unsupported languages
// ---------------------------------------------------------------------------

#[test]
fn c_files_not_analyzed() {
    let code = "void main() {}\n";
    let (dir, paths) = write_temp_files(&[("main.c", code)]);
    let export = make_export(vec![make_row("main.c", ".", "C")]);
    let report = build_api_surface_report(dir.path(), &paths, &export, &default_limits()).unwrap();
    assert_eq!(report.total_items, 0);
}

#[test]
fn ruby_files_not_analyzed() {
    let code = "class Foo; end\n";
    let (dir, paths) = write_temp_files(&[("app.rb", code)]);
    let export = make_export(vec![make_row("app.rb", ".", "Ruby")]);
    let report = build_api_surface_report(dir.path(), &paths, &export, &default_limits()).unwrap();
    assert_eq!(report.total_items, 0);
}

// ---------------------------------------------------------------------------
// Mixed visibility across multiple files in same language
// ---------------------------------------------------------------------------

#[test]
fn multiple_rust_files_aggregate_correctly() {
    let code_a = "pub fn a1() {}\npub fn a2() {}\nfn a3() {}\n";
    let code_b = "pub fn b1() {}\nfn b2() {}\nfn b3() {}\nfn b4() {}\n";
    let (dir, paths) = write_temp_files(&[("a.rs", code_a), ("b.rs", code_b)]);
    let export = make_export(vec![
        make_row("a.rs", "mod_a", "Rust"),
        make_row("b.rs", "mod_b", "Rust"),
    ]);
    let report = build_api_surface_report(dir.path(), &paths, &export, &default_limits()).unwrap();

    assert_eq!(report.total_items, 7);
    assert_eq!(report.public_items, 3);
    assert_eq!(report.internal_items, 4);

    // Per-language: single "Rust" entry
    assert_eq!(report.by_language.len(), 1);
    let rust_lang = &report.by_language["Rust"];
    assert_eq!(rust_lang.total_items, 7);
    assert_eq!(rust_lang.public_items, 3);
}

// ---------------------------------------------------------------------------
// Limits: max_file_bytes truncates large files
// ---------------------------------------------------------------------------

#[test]
fn max_file_bytes_limits_individual_file_scan() {
    // Create a file where the pub fn is past the byte limit
    let prefix = "// padding\n".repeat(100);
    let code = format!("{prefix}pub fn late_fn() {{}}\n");
    let (dir, paths) = write_temp_files(&[("big.rs", &code)]);
    let export = make_export(vec![make_row("big.rs", ".", "Rust")]);

    let limits = AnalysisLimits {
        max_file_bytes: Some(50), // only read first 50 bytes
        ..Default::default()
    };
    let report = build_api_surface_report(dir.path(), &paths, &export, &limits).unwrap();

    // The pub fn should be beyond the 50-byte window
    assert_eq!(report.public_items, 0);
}

// ---------------------------------------------------------------------------
// Module sorting tie-breaking by name
// ---------------------------------------------------------------------------

#[test]
fn modules_with_equal_items_sorted_by_name() {
    let code = "pub fn f() {}\n";
    let (dir, paths) =
        write_temp_files(&[("a/lib.rs", code), ("b/lib.rs", code), ("c/lib.rs", code)]);
    let export = make_export(vec![
        make_row("a/lib.rs", "alpha", "Rust"),
        make_row("b/lib.rs", "beta", "Rust"),
        make_row("c/lib.rs", "gamma", "Rust"),
    ]);
    let report = build_api_surface_report(dir.path(), &paths, &export, &default_limits()).unwrap();

    assert_eq!(report.by_module.len(), 3);
    // Equal total_items → sorted by module name ascending
    assert_eq!(report.by_module[0].module, "alpha");
    assert_eq!(report.by_module[1].module, "beta");
    assert_eq!(report.by_module[2].module, "gamma");
}

// ---------------------------------------------------------------------------
// Top exporters tie-breaking by path
// ---------------------------------------------------------------------------

#[test]
fn top_exporters_equal_public_sorted_by_path() {
    let code = "pub fn f() {}\n";
    let (dir, paths) = write_temp_files(&[("z.rs", code), ("a.rs", code)]);
    let export = make_export(vec![
        make_row("z.rs", ".", "Rust"),
        make_row("a.rs", ".", "Rust"),
    ]);
    let report = build_api_surface_report(dir.path(), &paths, &export, &default_limits()).unwrap();

    assert_eq!(report.top_exporters.len(), 2);
    // Equal public_items → sorted by path ascending
    assert_eq!(report.top_exporters[0].path, "a.rs");
    assert_eq!(report.top_exporters[1].path, "z.rs");
}

// ---------------------------------------------------------------------------
// Totals consistency: total = public + internal always
// ---------------------------------------------------------------------------

#[test]
fn totals_consistency_across_all_languages() {
    let rust = "pub fn r() {}\nfn ri() {}\n";
    let js = "export function j() {}\nfunction ji() {}\n";
    let ts = "export interface TI {}\ninterface ti {}\n";
    let py = "def pub():\n    pass\ndef _priv():\n    pass\n";
    let go = "func Pub() {}\nfunc priv_() {}\n";
    let java = "public class J {}\nclass Ji {}\n";

    let (dir, paths) = write_temp_files(&[
        ("lib.rs", rust),
        ("index.js", js),
        ("types.ts", ts),
        ("mod.py", py),
        ("main.go", go),
        ("App.java", java),
    ]);
    let export = make_export(vec![
        make_row("lib.rs", "rust", "Rust"),
        make_row("index.js", "js", "JavaScript"),
        make_row("types.ts", "ts", "TypeScript"),
        make_row("mod.py", "py", "Python"),
        make_row("main.go", "go", "Go"),
        make_row("App.java", "java", "Java"),
    ]);
    let report = build_api_surface_report(dir.path(), &paths, &export, &default_limits()).unwrap();

    assert_eq!(
        report.total_items,
        report.public_items + report.internal_items,
        "total must equal public + internal"
    );

    // Per-language consistency
    for (lang, surface) in &report.by_language {
        assert_eq!(
            surface.total_items,
            surface.public_items + surface.internal_items,
            "lang {lang}: total must equal public + internal"
        );
    }
}

// ---------------------------------------------------------------------------
// Non-existent file gracefully skipped
// ---------------------------------------------------------------------------

#[test]
fn nonexistent_file_is_skipped() {
    let dir = tempfile::tempdir().unwrap();
    let paths = vec![PathBuf::from("missing.rs")];
    let export = make_export(vec![make_row("missing.rs", ".", "Rust")]);
    let report = build_api_surface_report(dir.path(), &paths, &export, &default_limits()).unwrap();

    assert_eq!(report.total_items, 0);
}
