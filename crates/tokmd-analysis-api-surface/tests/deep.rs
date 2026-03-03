//! Deep integration tests for `tokmd-analysis-api-surface`.
//!
//! Covers: public function/struct/enum detection across languages, empty files,
//! non-Rust files, API surface item counting, serialization roundtrip, and
//! deterministic output.

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
// 1. Detect public functions in Rust code
// ---------------------------------------------------------------------------

#[test]
fn rust_pub_fn_detected_in_report() {
    let code = "pub fn alpha() {}\npub fn beta() {}\nfn gamma() {}\n";
    let (dir, paths) = write_temp_files(&[("lib.rs", code)]);
    let export = make_export(vec![make_row("lib.rs", "src", "Rust")]);
    let report = build_api_surface_report(dir.path(), &paths, &export, &default_limits()).unwrap();

    assert_eq!(report.public_items, 2);
    assert_eq!(report.internal_items, 1);
    assert_eq!(report.total_items, 3);
}

#[test]
fn rust_pub_async_fn_detected() {
    let code = "pub async fn serve() {}\nasync fn helper() {}\n";
    let (dir, paths) = write_temp_files(&[("server.rs", code)]);
    let export = make_export(vec![make_row("server.rs", "src", "Rust")]);
    let report = build_api_surface_report(dir.path(), &paths, &export, &default_limits()).unwrap();

    assert_eq!(report.public_items, 1);
    assert_eq!(report.internal_items, 1);
}

#[test]
fn rust_pub_unsafe_fn_detected() {
    let code = "pub unsafe fn dangerous() {}\nunsafe fn secret() {}\n";
    let (dir, paths) = write_temp_files(&[("ffi.rs", code)]);
    let export = make_export(vec![make_row("ffi.rs", "src", "Rust")]);
    let report = build_api_surface_report(dir.path(), &paths, &export, &default_limits()).unwrap();

    assert_eq!(report.public_items, 1);
    assert_eq!(report.internal_items, 1);
}

// ---------------------------------------------------------------------------
// 2. Detect public structs/enums
// ---------------------------------------------------------------------------

#[test]
fn rust_pub_struct_detected() {
    let code = "pub struct Config {\n    pub name: String,\n}\nstruct Internal {}\n";
    let (dir, paths) = write_temp_files(&[("types.rs", code)]);
    let export = make_export(vec![make_row("types.rs", "src", "Rust")]);
    let report = build_api_surface_report(dir.path(), &paths, &export, &default_limits()).unwrap();

    assert_eq!(report.public_items, 1);
    assert_eq!(report.internal_items, 1);
}

#[test]
fn rust_pub_enum_detected() {
    let code = "pub enum Color {\n    Red,\n    Green,\n    Blue,\n}\nenum Secret {}\n";
    let (dir, paths) = write_temp_files(&[("colors.rs", code)]);
    let export = make_export(vec![make_row("colors.rs", "src", "Rust")]);
    let report = build_api_surface_report(dir.path(), &paths, &export, &default_limits()).unwrap();

    assert_eq!(report.public_items, 1);
    assert_eq!(report.internal_items, 1);
}

#[test]
fn rust_pub_trait_detected() {
    let code = "pub trait Handler {\n    fn handle(&self);\n}\ntrait Internal {}\n";
    let (dir, paths) = write_temp_files(&[("traits.rs", code)]);
    let export = make_export(vec![make_row("traits.rs", "src", "Rust")]);
    let report = build_api_surface_report(dir.path(), &paths, &export, &default_limits()).unwrap();

    assert_eq!(report.public_items, 1);
    // `fn handle(&self);` inside the trait is also counted as an internal item
    // because the heuristic matches trimmed lines starting with `fn `.
    assert_eq!(report.internal_items, 2);
}

#[test]
fn rust_pub_type_const_static_mod_detected() {
    let code = concat!(
        "pub type Result<T> = std::result::Result<T, Error>;\n",
        "pub const MAX: usize = 100;\n",
        "pub static GLOBAL: &str = \"hello\";\n",
        "pub mod utils;\n",
        "pub unsafe trait Marker {}\n",
    );
    let (dir, paths) = write_temp_files(&[("defs.rs", code)]);
    let export = make_export(vec![make_row("defs.rs", "src", "Rust")]);
    let report = build_api_surface_report(dir.path(), &paths, &export, &default_limits()).unwrap();

    assert_eq!(report.public_items, 5);
}

#[test]
fn rust_pub_crate_items_detected_as_public() {
    let code = concat!(
        "pub(crate) fn internal_api() {}\n",
        "pub(super) struct ParentVisible {}\n",
        "pub(in crate::module) enum Scoped {}\n",
    );
    let (dir, paths) = write_temp_files(&[("scoped.rs", code)]);
    let export = make_export(vec![make_row("scoped.rs", "src", "Rust")]);
    let report = build_api_surface_report(dir.path(), &paths, &export, &default_limits()).unwrap();

    // pub(crate), pub(super), pub(in ...) are all treated as public
    assert_eq!(report.public_items, 3);
    assert_eq!(report.internal_items, 0);
}

// ---------------------------------------------------------------------------
// 3. Handle empty files
// ---------------------------------------------------------------------------

#[test]
fn empty_file_yields_zero_items() {
    let (dir, paths) = write_temp_files(&[("empty.rs", "")]);
    let export = make_export(vec![make_row("empty.rs", "src", "Rust")]);
    let report = build_api_surface_report(dir.path(), &paths, &export, &default_limits()).unwrap();

    assert_eq!(report.total_items, 0);
    assert_eq!(report.public_items, 0);
    assert_eq!(report.internal_items, 0);
    assert_eq!(report.public_ratio, 0.0);
    assert_eq!(report.documented_ratio, 0.0);
}

#[test]
fn empty_file_list_yields_zero_items() {
    let dir = tempfile::tempdir().unwrap();
    let export = make_export(vec![]);
    let report = build_api_surface_report(dir.path(), &[], &export, &default_limits()).unwrap();

    assert_eq!(report.total_items, 0);
    assert!(report.by_language.is_empty());
    assert!(report.by_module.is_empty());
    assert!(report.top_exporters.is_empty());
}

#[test]
fn comment_only_file_yields_zero_items() {
    let code = "// Just a comment\n// Another comment\n/* block comment */\n";
    let (dir, paths) = write_temp_files(&[("comments.rs", code)]);
    let export = make_export(vec![make_row("comments.rs", "src", "Rust")]);
    let report = build_api_surface_report(dir.path(), &paths, &export, &default_limits()).unwrap();

    assert_eq!(report.total_items, 0);
}

// ---------------------------------------------------------------------------
// 4. Handle non-Rust files (unsupported languages)
// ---------------------------------------------------------------------------

#[test]
fn unsupported_language_files_skipped() {
    let (dir, paths) = write_temp_files(&[
        ("style.css", ".class { color: red; }\n"),
        ("data.json", "{\"key\": \"value\"}\n"),
        ("readme.md", "# Title\n\nSome text.\n"),
    ]);
    let export = make_export(vec![
        make_row("style.css", ".", "CSS"),
        make_row("data.json", ".", "JSON"),
        make_row("readme.md", ".", "Markdown"),
    ]);
    let report = build_api_surface_report(dir.path(), &paths, &export, &default_limits()).unwrap();

    assert_eq!(report.total_items, 0);
    assert!(report.by_language.is_empty());
}

#[test]
fn binary_file_skipped() {
    let dir = tempfile::tempdir().unwrap();
    fs::write(dir.path().join("binary.rs"), b"\x00\x01\x02\x03\xff\xfe").unwrap();
    let paths = vec![PathBuf::from("binary.rs")];
    let export = make_export(vec![make_row("binary.rs", "src", "Rust")]);
    let report = build_api_surface_report(dir.path(), &paths, &export, &default_limits()).unwrap();

    assert_eq!(report.total_items, 0);
}

// ---------------------------------------------------------------------------
// 5. Count API surface items (multi-language)
// ---------------------------------------------------------------------------

#[test]
fn multi_language_item_counts_are_consistent() {
    let rust = "pub fn r_pub() {}\nfn r_priv() {}\n";
    let js = "export function j_pub() {}\nfunction j_priv() {}\n";
    let ts = "export class TsPub {}\nclass TsPriv {}\nexport interface ITsFace {}\n";
    let py = "def py_pub():\n    pass\ndef _py_priv():\n    pass\nclass PyClass:\n    pass\n";
    let go = "func GoPub() {}\nfunc goPriv() {}\ntype GoType struct {}\n";
    let java = "public class JPub {}\nclass JPriv {}\npublic interface IJFace {}\n";

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

    // Verify totals = sum of per-language
    let lang_total: usize = report.by_language.values().map(|l| l.total_items).sum();
    let lang_pub: usize = report.by_language.values().map(|l| l.public_items).sum();
    let lang_int: usize = report.by_language.values().map(|l| l.internal_items).sum();
    assert_eq!(report.total_items, lang_total);
    assert_eq!(report.public_items, lang_pub);
    assert_eq!(report.internal_items, lang_int);
    assert_eq!(
        report.total_items,
        report.public_items + report.internal_items
    );
    assert_eq!(report.by_language.len(), 6);
}

#[test]
fn public_ratio_computed_correctly() {
    let code = "pub fn a() {}\npub fn b() {}\nfn c() {}\nfn d() {}\n";
    let (dir, paths) = write_temp_files(&[("lib.rs", code)]);
    let export = make_export(vec![make_row("lib.rs", "src", "Rust")]);
    let report = build_api_surface_report(dir.path(), &paths, &export, &default_limits()).unwrap();

    assert_eq!(report.total_items, 4);
    assert_eq!(report.public_items, 2);
    assert_eq!(report.public_ratio, 0.5);
}

#[test]
fn documented_ratio_computed_correctly() {
    let code = concat!(
        "/// Documented public\n",
        "pub fn documented() {}\n",
        "pub fn undocumented() {}\n",
    );
    let (dir, paths) = write_temp_files(&[("lib.rs", code)]);
    let export = make_export(vec![make_row("lib.rs", "src", "Rust")]);
    let report = build_api_surface_report(dir.path(), &paths, &export, &default_limits()).unwrap();

    assert_eq!(report.public_items, 2);
    assert_eq!(report.documented_ratio, 0.5);
}

#[test]
fn top_exporters_contain_correct_counts() {
    let code_a = "pub fn one() {}\npub fn two() {}\npub fn three() {}\n";
    let code_b = "pub fn only_one() {}\nfn priv_b() {}\n";
    let (dir, paths) = write_temp_files(&[("big.rs", code_a), ("small.rs", code_b)]);
    let export = make_export(vec![
        make_row("big.rs", "src", "Rust"),
        make_row("small.rs", "src", "Rust"),
    ]);
    let report = build_api_surface_report(dir.path(), &paths, &export, &default_limits()).unwrap();

    assert_eq!(report.top_exporters.len(), 2);
    // Sorted by public_items descending
    assert_eq!(report.top_exporters[0].public_items, 3);
    assert_eq!(report.top_exporters[1].public_items, 1);
}

#[test]
fn by_module_aggregates_across_files_in_same_module() {
    let code_a = "pub fn a1() {}\nfn a2() {}\n";
    let code_b = "pub fn b1() {}\npub fn b2() {}\n";
    let (dir, paths) = write_temp_files(&[("src/a.rs", code_a), ("src/b.rs", code_b)]);
    let export = make_export(vec![
        make_row("src/a.rs", "src", "Rust"),
        make_row("src/b.rs", "src", "Rust"),
    ]);
    let report = build_api_surface_report(dir.path(), &paths, &export, &default_limits()).unwrap();

    assert_eq!(report.by_module.len(), 1);
    assert_eq!(report.by_module[0].module, "src");
    assert_eq!(report.by_module[0].total_items, 4);
    assert_eq!(report.by_module[0].public_items, 3);
}

#[test]
fn by_language_shows_per_lang_breakdown() {
    let rust = "pub fn r() {}\nfn r2() {}\n";
    let py = "def py_pub():\n    pass\n";
    let (dir, paths) = write_temp_files(&[("lib.rs", rust), ("main.py", py)]);
    let export = make_export(vec![
        make_row("lib.rs", "src", "Rust"),
        make_row("main.py", "py", "Python"),
    ]);
    let report = build_api_surface_report(dir.path(), &paths, &export, &default_limits()).unwrap();

    assert_eq!(report.by_language.len(), 2);
    let rust_surface = &report.by_language["Rust"];
    assert_eq!(rust_surface.total_items, 2);
    assert_eq!(rust_surface.public_items, 1);
    assert_eq!(rust_surface.internal_items, 1);
    assert_eq!(rust_surface.public_ratio, 0.5);

    let py_surface = &report.by_language["Python"];
    assert_eq!(py_surface.total_items, 1);
    assert_eq!(py_surface.public_items, 1);
}

// ---------------------------------------------------------------------------
// 6. Serialization roundtrip of results
// ---------------------------------------------------------------------------

#[test]
fn report_json_roundtrip_preserves_all_fields() {
    let code = concat!(
        "/// Documented\n",
        "pub fn documented() {}\n",
        "pub struct Config {}\n",
        "fn private_fn() {}\n",
        "enum InternalEnum {}\n",
    );
    let (dir, paths) = write_temp_files(&[("lib.rs", code)]);
    let export = make_export(vec![make_row("lib.rs", "src", "Rust")]);
    let report = build_api_surface_report(dir.path(), &paths, &export, &default_limits()).unwrap();

    let json = serde_json::to_string_pretty(&report).unwrap();
    let deserialized: ApiSurfaceReport = serde_json::from_str(&json).unwrap();

    assert_eq!(deserialized.total_items, report.total_items);
    assert_eq!(deserialized.public_items, report.public_items);
    assert_eq!(deserialized.internal_items, report.internal_items);
    assert_eq!(deserialized.public_ratio, report.public_ratio);
    assert_eq!(deserialized.documented_ratio, report.documented_ratio);
    assert_eq!(deserialized.by_language.len(), report.by_language.len());
    assert_eq!(deserialized.by_module.len(), report.by_module.len());
    assert_eq!(deserialized.top_exporters.len(), report.top_exporters.len());
    for (key, orig) in &report.by_language {
        let deser = &deserialized.by_language[key];
        assert_eq!(deser.total_items, orig.total_items);
        assert_eq!(deser.public_items, orig.public_items);
        assert_eq!(deser.internal_items, orig.internal_items);
        assert_eq!(deser.public_ratio, orig.public_ratio);
    }
}

#[test]
fn empty_report_serializes_and_deserializes() {
    let dir = tempfile::tempdir().unwrap();
    let export = make_export(vec![]);
    let report = build_api_surface_report(dir.path(), &[], &export, &default_limits()).unwrap();

    let json = serde_json::to_string(&report).unwrap();
    let deserialized: ApiSurfaceReport = serde_json::from_str(&json).unwrap();

    assert_eq!(deserialized.total_items, 0);
    assert!(deserialized.by_language.is_empty());
    assert!(deserialized.by_module.is_empty());
    assert!(deserialized.top_exporters.is_empty());
}

#[test]
fn lang_api_surface_json_roundtrip() {
    let surface = LangApiSurface {
        total_items: 10,
        public_items: 7,
        internal_items: 3,
        public_ratio: 0.7,
    };
    let json = serde_json::to_string(&surface).unwrap();
    let deser: LangApiSurface = serde_json::from_str(&json).unwrap();
    assert_eq!(deser.total_items, 10);
    assert_eq!(deser.public_items, 7);
    assert_eq!(deser.internal_items, 3);
    assert_eq!(deser.public_ratio, 0.7);
}

#[test]
fn module_api_row_json_roundtrip() {
    let row = ModuleApiRow {
        module: "core".to_string(),
        total_items: 20,
        public_items: 15,
        public_ratio: 0.75,
    };
    let json = serde_json::to_string(&row).unwrap();
    let deser: ModuleApiRow = serde_json::from_str(&json).unwrap();
    assert_eq!(deser.module, "core");
    assert_eq!(deser.total_items, 20);
}

#[test]
fn api_export_item_json_roundtrip() {
    let item = ApiExportItem {
        path: "src/lib.rs".to_string(),
        lang: "Rust".to_string(),
        public_items: 5,
        total_items: 8,
    };
    let json = serde_json::to_string(&item).unwrap();
    let deser: ApiExportItem = serde_json::from_str(&json).unwrap();
    assert_eq!(deser.path, "src/lib.rs");
    assert_eq!(deser.public_items, 5);
}

// ---------------------------------------------------------------------------
// 7. Deterministic results
// ---------------------------------------------------------------------------

#[test]
fn report_is_deterministic_across_multiple_runs() {
    let code = concat!(
        "pub fn alpha() {}\n",
        "pub struct Beta {}\n",
        "fn gamma() {}\n",
        "pub enum Delta {}\n",
        "/// Doc\npub trait Epsilon {}\n",
    );
    let (dir, paths) = write_temp_files(&[("lib.rs", code)]);
    let export = make_export(vec![make_row("lib.rs", "src", "Rust")]);

    let report1 = build_api_surface_report(dir.path(), &paths, &export, &default_limits()).unwrap();
    let report2 = build_api_surface_report(dir.path(), &paths, &export, &default_limits()).unwrap();

    let json1 = serde_json::to_string(&report1).unwrap();
    let json2 = serde_json::to_string(&report2).unwrap();
    assert_eq!(json1, json2, "Reports should be byte-identical");
}

#[test]
fn multi_file_report_is_deterministic() {
    let code_a = "pub fn a() {}\nfn a_priv() {}\n";
    let code_b = "pub struct B {}\nenum BPriv {}\n";
    let code_c = "export function cPub() {}\nfunction cPriv() {}\n";

    let (dir, paths) = write_temp_files(&[("a.rs", code_a), ("b.rs", code_b), ("c.js", code_c)]);
    let export = make_export(vec![
        make_row("a.rs", "src", "Rust"),
        make_row("b.rs", "src", "Rust"),
        make_row("c.js", "web", "JavaScript"),
    ]);

    let report1 = build_api_surface_report(dir.path(), &paths, &export, &default_limits()).unwrap();
    let report2 = build_api_surface_report(dir.path(), &paths, &export, &default_limits()).unwrap();

    let json1 = serde_json::to_string(&report1).unwrap();
    let json2 = serde_json::to_string(&report2).unwrap();
    assert_eq!(json1, json2);
}

#[test]
fn by_language_uses_btreemap_for_stable_ordering() {
    let rust = "pub fn r() {}\n";
    let js = "export function j() {}\n";
    let py = "def p():\n    pass\n";

    let (dir, paths) = write_temp_files(&[("lib.rs", rust), ("index.js", js), ("main.py", py)]);
    let export = make_export(vec![
        make_row("lib.rs", "src", "Rust"),
        make_row("index.js", "web", "JavaScript"),
        make_row("main.py", "py", "Python"),
    ]);

    let report = build_api_surface_report(dir.path(), &paths, &export, &default_limits()).unwrap();
    let keys: Vec<&String> = report.by_language.keys().collect();
    let mut sorted_keys = keys.clone();
    sorted_keys.sort();
    assert_eq!(
        keys, sorted_keys,
        "by_language keys should be sorted (BTreeMap)"
    );
}

// ---------------------------------------------------------------------------
// Additional edge cases
// ---------------------------------------------------------------------------

#[test]
fn max_bytes_limit_stops_processing_early() {
    let code = "pub fn found() {}\n";
    let (dir, paths) = write_temp_files(&[("a.rs", code), ("b.rs", code)]);
    let export = make_export(vec![
        make_row("a.rs", "src", "Rust"),
        make_row("b.rs", "src", "Rust"),
    ]);

    let limits = AnalysisLimits {
        max_bytes: Some(5),
        ..Default::default()
    };
    let report = build_api_surface_report(dir.path(), &paths, &export, &limits).unwrap();
    // With a tiny budget, at most one file is processed
    assert!(report.public_items <= 1);
}

#[test]
fn js_ts_all_export_forms_detected() {
    let code = concat!(
        "export function fn1() {}\n",
        "export async function fn2() {}\n",
        "export class Cls1 {}\n",
        "export const CONST1 = 1;\n",
        "export let let1 = 2;\n",
        "export default function main() {}\n",
        "export interface IFace {}\n",
        "export type MyType = string;\n",
        "export enum MyEnum {}\n",
        "export abstract class AbsCls {}\n",
    );
    let (dir, paths) = write_temp_files(&[("all.ts", code)]);
    let export = make_export(vec![make_row("all.ts", "src", "TypeScript")]);
    let report = build_api_surface_report(dir.path(), &paths, &export, &default_limits()).unwrap();

    assert_eq!(report.public_items, 10);
}

#[test]
fn go_method_receiver_public_and_private() {
    let code = concat!(
        "func (s *Server) Start() {}\n",
        "func (s *Server) stop() {}\n",
        "type Config struct {}\n",
        "type config struct {}\n",
    );
    let (dir, paths) = write_temp_files(&[("server.go", code)]);
    let export = make_export(vec![make_row("server.go", "pkg", "Go")]);
    let report = build_api_surface_report(dir.path(), &paths, &export, &default_limits()).unwrap();

    assert_eq!(report.public_items, 2); // Start, Config
    assert_eq!(report.internal_items, 2); // stop, config
}

#[test]
fn java_public_method_with_return_type_detected() {
    let code = concat!(
        "public String getName() { return name; }\n",
        "public void setName(String name) { this.name = name; }\n",
        "private int count() { return 0; }\n",
    );
    let (dir, paths) = write_temp_files(&[("Bean.java", code)]);
    let export = make_export(vec![make_row("Bean.java", "com", "Java")]);
    let report = build_api_surface_report(dir.path(), &paths, &export, &default_limits()).unwrap();

    assert_eq!(report.public_items, 2);
    assert_eq!(report.internal_items, 1);
}

#[test]
fn python_docstring_detection_across_forms() {
    let code = concat!(
        "def documented():\n",
        "    \"\"\"Has docstring.\"\"\"\n",
        "    pass\n",
        "\n",
        "def also_documented():\n",
        "    '''Single-quote docstring.'''\n",
        "    pass\n",
        "\n",
        "def undocumented():\n",
        "    pass\n",
    );
    let (dir, paths) = write_temp_files(&[("mod.py", code)]);
    let export = make_export(vec![make_row("mod.py", "pkg", "Python")]);
    let report = build_api_surface_report(dir.path(), &paths, &export, &default_limits()).unwrap();

    assert_eq!(report.public_items, 3);
    // documented + also_documented have docstrings
    assert!(report.documented_ratio > 0.0);
}

#[test]
fn child_file_rows_are_ignored() {
    let code = "pub fn visible() {}\n";
    let (dir, paths) = write_temp_files(&[("child.rs", code)]);
    let export = ExportData {
        rows: vec![FileRow {
            path: "child.rs".to_string(),
            module: "src".to_string(),
            lang: "Rust".to_string(),
            kind: FileKind::Child,
            code: 10,
            comments: 0,
            blanks: 0,
            lines: 10,
            bytes: 50,
            tokens: 20,
        }],
        module_roots: vec![],
        module_depth: 1,
        children: ChildIncludeMode::Separate,
    };
    let report = build_api_surface_report(dir.path(), &paths, &export, &default_limits()).unwrap();

    // Child rows are filtered out of the row_map
    assert_eq!(report.total_items, 0);
}
