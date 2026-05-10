use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use anyhow::Result;
use tokmd_analysis_types::{ApiExportItem, ApiSurfaceReport, LangApiSurface, ModuleApiRow};
use tokmd_types::{ExportData, FileKind, FileRow};

use tokmd_analysis_types::{AnalysisLimits, normalize_path};

#[cfg(test)]
#[path = "tests.rs"]
mod moved_tests;

mod symbols;

const DEFAULT_MAX_FILE_BYTES: u64 = 128 * 1024;
const MAX_TOP_EXPORTERS: usize = 20;
const MAX_BY_MODULE: usize = 50;

// -------
// Main
// -------

/// Build the API surface report by scanning source files for public/internal symbols.
pub(crate) fn build_api_surface_report(
    root: &Path,
    files: &[PathBuf],
    export: &ExportData,
    limits: &AnalysisLimits,
) -> Result<ApiSurfaceReport> {
    // Build lookup from normalized path -> FileRow
    let mut row_map: BTreeMap<String, &FileRow> = BTreeMap::new();
    for row in export.rows.iter().filter(|r| r.kind == FileKind::Parent) {
        row_map.insert(normalize_path(&row.path, root), row);
    }

    let per_file_limit = limits.max_file_bytes.unwrap_or(DEFAULT_MAX_FILE_BYTES) as usize;
    let mut total_bytes = 0u64;

    // Accumulators
    let mut total_items = 0usize;
    let mut public_items = 0usize;
    let mut internal_items = 0usize;
    let mut documented_public = 0usize;

    // Per-language accumulators
    let mut lang_totals: BTreeMap<&str, (usize, usize, usize)> = BTreeMap::new(); // (total, public, internal)

    // Per-module accumulators
    let mut module_totals: BTreeMap<&str, (usize, usize)> = BTreeMap::new(); // (total, public)

    // Top exporters
    let mut exporters: Vec<ApiExportItem> = Vec::new();

    for rel in files {
        if limits.max_bytes.is_some_and(|limit| total_bytes >= limit) {
            break;
        }

        let rel_str = normalize_path(&rel.to_string_lossy(), root);
        let row = match row_map.get(&rel_str) {
            Some(r) => *r,
            None => continue,
        };

        if !symbols::is_api_surface_lang(&row.lang) {
            continue;
        }

        let path = root.join(rel);
        let bytes = match crate::content::io::read_head(&path, per_file_limit) {
            Ok(b) => b,
            Err(_) => continue,
        };
        total_bytes += bytes.len() as u64;

        if !crate::content::io::is_text_like(&bytes) {
            continue;
        }

        let text = String::from_utf8_lossy(&bytes);
        let symbols = symbols::extract_symbols(&row.lang, &text);

        if symbols.is_empty() {
            continue;
        }

        let file_public: usize = symbols.iter().filter(|s| s.is_public).count();
        let file_internal: usize = symbols.iter().filter(|s| !s.is_public).count();
        let file_documented: usize = symbols
            .iter()
            .filter(|s| s.is_public && s.is_documented)
            .count();
        let file_total = symbols.len();

        total_items += file_total;
        public_items += file_public;
        internal_items += file_internal;
        documented_public += file_documented;

        // Per-language
        let entry = lang_totals.entry(row.lang.as_str()).or_insert((0, 0, 0));
        entry.0 += file_total;
        entry.1 += file_public;
        entry.2 += file_internal;

        // Per-module
        let mod_entry = module_totals.entry(row.module.as_str()).or_insert((0, 0));
        mod_entry.0 += file_total;
        mod_entry.1 += file_public;

        // Track top exporters
        if file_public > 0 {
            exporters.push(ApiExportItem {
                path: rel_str,
                lang: row.lang.clone(),
                public_items: file_public,
                total_items: file_total,
            });
        }
    }

    // Build per-language map
    let by_language: BTreeMap<String, LangApiSurface> = lang_totals
        .into_iter()
        .map(|(lang, (total, public, internal))| {
            let public_ratio = if total == 0 {
                0.0
            } else {
                round_f64(public as f64 / total as f64, 4)
            };
            (
                lang.to_owned(),
                LangApiSurface {
                    total_items: total,
                    public_items: public,
                    internal_items: internal,
                    public_ratio,
                },
            )
        })
        .collect();

    // Build per-module vec, sorted by total items descending
    let mut by_module: Vec<ModuleApiRow> = module_totals
        .into_iter()
        .map(|(module, (total, public))| {
            let public_ratio = if total == 0 {
                0.0
            } else {
                round_f64(public as f64 / total as f64, 4)
            };
            ModuleApiRow {
                module: module.to_owned(),
                total_items: total,
                public_items: public,
                public_ratio,
            }
        })
        .collect();
    by_module.sort_by(|a, b| {
        b.total_items
            .cmp(&a.total_items)
            .then_with(|| a.module.cmp(&b.module))
    });
    by_module.truncate(MAX_BY_MODULE);

    // Sort top exporters by public_items descending, then by path
    exporters.sort_by(|a, b| {
        b.public_items
            .cmp(&a.public_items)
            .then_with(|| a.path.cmp(&b.path))
    });
    exporters.truncate(MAX_TOP_EXPORTERS);

    let public_ratio = if total_items == 0 {
        0.0
    } else {
        round_f64(public_items as f64 / total_items as f64, 4)
    };

    let documented_ratio = if public_items == 0 {
        0.0
    } else {
        round_f64(documented_public as f64 / public_items as f64, 4)
    };

    Ok(ApiSurfaceReport {
        total_items,
        public_items,
        internal_items,
        public_ratio,
        documented_ratio,
        by_language,
        by_module,
        top_exporters: exporters,
    })
}

fn round_f64(val: f64, decimals: u32) -> f64 {
    let factor = 10f64.powi(decimals as i32);
    (val * factor).round() / factor
}

#[cfg(test)]
mod tests {
    use super::symbols::{extract_symbols, has_doc_comment, is_api_surface_lang};
    use super::*;

    // -------
    // Rust symbol extraction
    // -------

    #[test]
    fn rust_pub_fn() {
        let code = "pub fn foo() {\n}\n";
        let syms = extract_symbols("rust", code);
        assert_eq!(syms.len(), 1);
        assert!(syms[0].is_public);
    }

    #[test]
    fn rust_private_fn() {
        let code = "fn bar() {\n}\n";
        let syms = extract_symbols("rust", code);
        assert_eq!(syms.len(), 1);
        assert!(!syms[0].is_public);
    }

    #[test]
    fn rust_pub_struct_enum_trait() {
        let code = "pub struct Foo;\npub enum Bar {}\npub trait Baz {}\n";
        let syms = extract_symbols("rust", code);
        assert_eq!(syms.len(), 3);
        assert!(syms.iter().all(|s| s.is_public));
    }

    #[test]
    fn rust_pub_crate() {
        let code = "pub(crate) fn internal_fn() {\n}\n";
        let syms = extract_symbols("rust", code);
        assert_eq!(syms.len(), 1);
        // pub(crate) is still considered pub for API surface purposes
        assert!(syms[0].is_public);
    }

    #[test]
    fn rust_internal_items() {
        let code = "struct Private;\nenum InternalEnum {}\ntrait InternalTrait {}\n";
        let syms = extract_symbols("rust", code);
        assert_eq!(syms.len(), 3);
        assert!(syms.iter().all(|s| !s.is_public));
    }

    #[test]
    fn rust_documented_item() {
        let code = "/// Documentation\npub fn documented() {\n}\n";
        let syms = extract_symbols("rust", code);
        assert_eq!(syms.len(), 1);
        assert!(syms[0].is_public);
        assert!(syms[0].is_documented);
    }

    #[test]
    fn rust_undocumented_item() {
        let code = "pub fn undocumented() {\n}\n";
        let syms = extract_symbols("rust", code);
        assert_eq!(syms.len(), 1);
        assert!(syms[0].is_public);
        assert!(!syms[0].is_documented);
    }

    #[test]
    fn rust_pub_mod_const_static() {
        let code = "pub mod mymod;\npub const X: u32 = 1;\npub static Y: &str = \"hi\";\n";
        let syms = extract_symbols("rust", code);
        assert_eq!(syms.len(), 3);
        assert!(syms.iter().all(|s| s.is_public));
    }

    #[test]
    fn rust_pub_type_alias() {
        let code = "pub type MyResult = Result<(), Error>;\n";
        let syms = extract_symbols("rust", code);
        assert_eq!(syms.len(), 1);
        assert!(syms[0].is_public);
    }

    #[test]
    fn rust_async_unsafe() {
        let code = "pub async fn async_pub() {}\npub unsafe fn unsafe_pub() {}\n";
        let syms = extract_symbols("rust", code);
        assert_eq!(syms.len(), 2);
        assert!(syms.iter().all(|s| s.is_public));
    }

    // -------
    // JS/TS symbol extraction
    // -------

    #[test]
    fn js_export_function() {
        let code = "export function foo() {\n}\n";
        let syms = extract_symbols("javascript", code);
        assert_eq!(syms.len(), 1);
        assert!(syms[0].is_public);
    }

    #[test]
    fn js_export_class() {
        let code = "export class MyClass {\n}\n";
        let syms = extract_symbols("typescript", code);
        assert_eq!(syms.len(), 1);
        assert!(syms[0].is_public);
    }

    #[test]
    fn js_export_const_default() {
        let code = "export const X = 1;\nexport default function main() {}\n";
        let syms = extract_symbols("javascript", code);
        assert_eq!(syms.len(), 2);
        assert!(syms.iter().all(|s| s.is_public));
    }

    #[test]
    fn ts_export_interface_type_enum() {
        let code =
            "export interface IFoo {}\nexport type Bar = string;\nexport enum Baz { A, B }\n";
        let syms = extract_symbols("typescript", code);
        assert_eq!(syms.len(), 3);
        assert!(syms.iter().all(|s| s.is_public));
    }

    #[test]
    fn js_internal_function() {
        let code = "function internal() {\n}\n";
        let syms = extract_symbols("javascript", code);
        assert_eq!(syms.len(), 1);
        assert!(!syms[0].is_public);
    }

    // -------
    // Python symbol extraction
    // -------

    #[test]
    fn python_public_def() {
        let code = "def public_func():\n    pass\n";
        let syms = extract_symbols("python", code);
        assert_eq!(syms.len(), 1);
        assert!(syms[0].is_public);
    }

    #[test]
    fn python_private_def() {
        let code = "def _private_func():\n    pass\n";
        let syms = extract_symbols("python", code);
        assert_eq!(syms.len(), 1);
        assert!(!syms[0].is_public);
    }

    #[test]
    fn python_class() {
        let code = "class MyClass:\n    pass\n";
        let syms = extract_symbols("python", code);
        assert_eq!(syms.len(), 1);
        assert!(syms[0].is_public);
    }

    #[test]
    fn python_private_class() {
        let code = "class _InternalClass:\n    pass\n";
        let syms = extract_symbols("python", code);
        assert_eq!(syms.len(), 1);
        assert!(!syms[0].is_public);
    }

    #[test]
    fn python_indented_def_ignored() {
        let code = "class Foo:\n    def method(self):\n        pass\n";
        let syms = extract_symbols("python", code);
        // Only top-level class, not the method
        assert_eq!(syms.len(), 1);
        assert!(syms[0].is_public);
    }

    #[test]
    fn python_docstring_detected() {
        let code = "def documented():\n    \"\"\"This is documented.\"\"\"\n    pass\n";
        let syms = extract_symbols("python", code);
        assert_eq!(syms.len(), 1);
        assert!(syms[0].is_documented);
    }

    // -------
    // Go symbol extraction
    // -------

    #[test]
    fn go_public_func() {
        let code = "func PublicFunc() {\n}\n";
        let syms = extract_symbols("go", code);
        assert_eq!(syms.len(), 1);
        assert!(syms[0].is_public);
    }

    #[test]
    fn go_private_func() {
        let code = "func privateFunc() {\n}\n";
        let syms = extract_symbols("go", code);
        assert_eq!(syms.len(), 1);
        assert!(!syms[0].is_public);
    }

    #[test]
    fn go_public_type() {
        let code = "type MyStruct struct {\n}\n";
        let syms = extract_symbols("go", code);
        assert_eq!(syms.len(), 1);
        assert!(syms[0].is_public);
    }

    #[test]
    fn go_method_receiver() {
        let code = "func (s *Server) Handle() {\n}\n";
        let syms = extract_symbols("go", code);
        assert_eq!(syms.len(), 1);
        assert!(syms[0].is_public);
    }

    #[test]
    fn go_private_method() {
        let code = "func (s *Server) handle() {\n}\n";
        let syms = extract_symbols("go", code);
        assert_eq!(syms.len(), 1);
        assert!(!syms[0].is_public);
    }

    // -------
    // Java symbol extraction
    // -------

    #[test]
    fn java_public_class() {
        let code = "public class MyClass {\n}\n";
        let syms = extract_symbols("java", code);
        assert_eq!(syms.len(), 1);
        assert!(syms[0].is_public);
    }

    #[test]
    fn java_public_interface() {
        let code = "public interface MyInterface {\n}\n";
        let syms = extract_symbols("java", code);
        assert_eq!(syms.len(), 1);
        assert!(syms[0].is_public);
    }

    #[test]
    fn java_public_enum() {
        let code = "public enum Color {\n    RED, GREEN, BLUE\n}\n";
        let syms = extract_symbols("java", code);
        assert_eq!(syms.len(), 1);
        assert!(syms[0].is_public);
    }

    #[test]
    fn java_public_static_method() {
        let code = "public static void main(String[] args) {\n}\n";
        let syms = extract_symbols("java", code);
        assert_eq!(syms.len(), 1);
        assert!(syms[0].is_public);
    }

    #[test]
    fn java_package_private_class() {
        let code = "class InternalClass {\n}\n";
        let syms = extract_symbols("java", code);
        assert_eq!(syms.len(), 1);
        assert!(!syms[0].is_public);
    }

    #[test]
    fn java_private_member() {
        let code = "private void helper() {\n}\n";
        let syms = extract_symbols("java", code);
        assert_eq!(syms.len(), 1);
        assert!(!syms[0].is_public);
    }

    #[test]
    fn java_documented() {
        let code = "/** Javadoc */\npublic class Documented {\n}\n";
        let syms = extract_symbols("java", code);
        assert_eq!(syms.len(), 1);
        assert!(syms[0].is_documented);
    }

    // -------
    // Unsupported language
    // -------

    #[test]
    fn unsupported_lang_returns_empty() {
        let code = "some code here\n";
        let syms = extract_symbols("markdown", code);
        assert!(syms.is_empty());
    }

    #[test]
    fn empty_input_returns_empty() {
        for lang in &["rust", "javascript", "typescript", "python", "go", "java"] {
            let syms = extract_symbols(lang, "");
            assert!(
                syms.is_empty(),
                "empty input for {lang} should yield no symbols"
            );
        }
    }

    // -------
    // is_api_surface_lang
    // -------

    #[test]
    fn supported_langs() {
        assert!(is_api_surface_lang("Rust"));
        assert!(is_api_surface_lang("JavaScript"));
        assert!(is_api_surface_lang("TypeScript"));
        assert!(is_api_surface_lang("Python"));
        assert!(is_api_surface_lang("Go"));
        assert!(is_api_surface_lang("Java"));
    }

    #[test]
    fn supported_langs_case_insensitive() {
        assert!(is_api_surface_lang("RUST"));
        assert!(is_api_surface_lang("javascript"));
        assert!(is_api_surface_lang("gO"));
    }

    #[test]
    fn unsupported_langs() {
        assert!(!is_api_surface_lang("Markdown"));
        assert!(!is_api_surface_lang("JSON"));
        assert!(!is_api_surface_lang("CSS"));
    }

    // -------
    // has_doc_comment edge cases
    // -------

    #[test]
    fn has_doc_comment_at_index_zero_is_false() {
        let lines = vec!["pub fn foo() {}"];
        assert!(!has_doc_comment(&lines, 0));
    }

    #[test]
    fn has_doc_comment_with_doc_attribute() {
        let lines = vec!["#[doc = \"documented\"]", "pub fn foo() {}"];
        assert!(has_doc_comment(&lines, 1));
    }

    // -------
    // Go var/const
    // -------

    #[test]
    fn go_var_public() {
        let code = "var PublicVar int = 42\n";
        let syms = extract_symbols("go", code);
        assert_eq!(syms.len(), 1);
        assert!(syms[0].is_public);
    }

    #[test]
    fn go_const_private() {
        let code = "const maxBuffer = 1024\n";
        let syms = extract_symbols("go", code);
        assert_eq!(syms.len(), 1);
        assert!(!syms[0].is_public);
    }

    // -------
    // Python async def
    // -------

    #[test]
    fn python_async_def() {
        let code = "async def fetch():\n    pass\n";
        let syms = extract_symbols("python", code);
        assert_eq!(syms.len(), 1);
        assert!(syms[0].is_public);
    }

    #[test]
    fn python_async_def_private() {
        let code = "async def _fetch():\n    pass\n";
        let syms = extract_symbols("python", code);
        assert_eq!(syms.len(), 1);
        assert!(!syms[0].is_public);
    }

    // -------
    // Java additional forms
    // -------

    #[test]
    fn java_public_record() {
        let code = "public record Point(int x, int y) {}\n";
        let syms = extract_symbols("java", code);
        assert_eq!(syms.len(), 1);
        assert!(syms[0].is_public);
    }

    #[test]
    fn java_protected_member() {
        let code = "protected void helper() {}\n";
        let syms = extract_symbols("java", code);
        assert_eq!(syms.len(), 1);
        assert!(!syms[0].is_public);
    }

    // -------
    // JS/TS export enum
    // -------

    #[test]
    fn ts_export_enum() {
        let code = "export enum Direction { Up, Down }\n";
        let syms = extract_symbols("typescript", code);
        assert_eq!(syms.len(), 1);
        assert!(syms[0].is_public);
    }

    #[test]
    fn js_async_function_internal() {
        let code = "async function doWork() {}\n";
        let syms = extract_symbols("javascript", code);
        assert_eq!(syms.len(), 1);
        assert!(!syms[0].is_public);
    }

    // -------
    // Rust pub with unmatched paren
    // -------

    #[test]
    fn rust_pub_unmatched_paren_no_panic() {
        let code = "pub(broken fn foo() {}\n";
        let syms = extract_symbols("rust", code);
        // Unmatched paren should not match as pub item
        assert!(syms.is_empty() || !syms[0].is_public);
    }

    // -------
    // round_f64
    // -------

    #[test]
    fn test_round() {
        assert_eq!(round_f64(0.12345, 4), 0.1235);
        assert_eq!(round_f64(0.5, 0), 1.0);
        assert_eq!(round_f64(1.0, 4), 1.0);
    }

    #[test]
    fn test_round_zero() {
        assert_eq!(round_f64(0.0, 4), 0.0);
    }

    #[test]
    fn test_round_small_fraction() {
        assert_eq!(round_f64(0.3333, 2), 0.33);
        assert_eq!(round_f64(0.6667, 2), 0.67);
    }
}
