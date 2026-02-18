use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use anyhow::Result;
use tokmd_analysis_types::{ApiExportItem, ApiSurfaceReport, LangApiSurface, ModuleApiRow};
use tokmd_types::{ExportData, FileKind, FileRow};

use crate::analysis::AnalysisLimits;
use crate::util::normalize_path;

const DEFAULT_MAX_FILE_BYTES: u64 = 128 * 1024;
const MAX_TOP_EXPORTERS: usize = 20;
const MAX_BY_MODULE: usize = 50;

/// Languages supported for API surface analysis.
fn is_api_surface_lang(lang: &str) -> bool {
    matches!(
        lang.to_lowercase().as_str(),
        "rust" | "javascript" | "typescript" | "python" | "go" | "java"
    )
}

/// Represents a single discovered symbol.
#[derive(Debug)]
struct Symbol {
    is_public: bool,
    is_documented: bool,
}

/// Scan a file for public/internal symbols and documentation.
fn extract_symbols(lang: &str, text: &str) -> Vec<Symbol> {
    let lines: Vec<&str> = text.lines().collect();
    match lang.to_lowercase().as_str() {
        "rust" => extract_rust_symbols(&lines),
        "javascript" | "typescript" => extract_js_ts_symbols(&lines),
        "python" => extract_python_symbols(&lines),
        "go" => extract_go_symbols(&lines),
        "java" => extract_java_symbols(&lines),
        _ => Vec::new(),
    }
}

/// Check whether the line preceding a symbol looks like a doc comment.
fn has_doc_comment(lines: &[&str], idx: usize) -> bool {
    if idx == 0 {
        return false;
    }
    let prev = lines[idx - 1].trim();
    // Rust: /// or //! or #[doc
    // JS/TS/Java: /** or //
    // Python: """ or ''' (handled separately)
    // Go: // directly before declaration
    prev.starts_with("///")
        || prev.starts_with("//!")
        || prev.starts_with("/**")
        || prev.starts_with("#[doc")
        || prev.starts_with("/// ")
        || prev.starts_with("// ")
        || prev.starts_with("\"\"\"")
        || prev.starts_with("'''")
}

// -------
// Rust
// -------

fn extract_rust_symbols(lines: &[&str]) -> Vec<Symbol> {
    let mut symbols = Vec::new();

    for (i, line) in lines.iter().enumerate() {
        let trimmed = line.trim();
        // Skip lines inside string literals or comments (simple heuristic)
        if trimmed.starts_with("//") || trimmed.starts_with('*') || trimmed.starts_with("/*") {
            continue;
        }

        let is_public = is_rust_pub_item(trimmed);
        let is_internal = is_rust_internal_item(trimmed);

        if is_public || is_internal {
            symbols.push(Symbol {
                is_public,
                is_documented: has_doc_comment(lines, i),
            });
        }
    }

    symbols
}

fn is_rust_pub_item(trimmed: &str) -> bool {
    // Match pub items, including pub(crate), pub(super), pub(in ...)
    if !trimmed.starts_with("pub ") && !trimmed.starts_with("pub(") {
        return false;
    }

    // Find the part after the pub qualifier
    let after_pub = if trimmed.starts_with("pub(") {
        // Find matching close paren
        if let Some(close) = trimmed.find(')') {
            trimmed[close + 1..].trim_start()
        } else {
            return false;
        }
    } else {
        // "pub " prefix
        &trimmed[4..]
    };

    // Now check for item keywords
    after_pub.starts_with("fn ")
        || after_pub.starts_with("struct ")
        || after_pub.starts_with("enum ")
        || after_pub.starts_with("trait ")
        || after_pub.starts_with("type ")
        || after_pub.starts_with("const ")
        || after_pub.starts_with("static ")
        || after_pub.starts_with("mod ")
        || after_pub.starts_with("async fn ")
        || after_pub.starts_with("unsafe fn ")
        || after_pub.starts_with("unsafe trait ")
}

fn is_rust_internal_item(trimmed: &str) -> bool {
    // Non-pub items at start of line (no leading whitespace for top-level heuristic
    // but we keep it simple: any fn/struct/etc. without pub)
    if trimmed.starts_with("pub ") || trimmed.starts_with("pub(") {
        return false;
    }

    trimmed.starts_with("fn ")
        || trimmed.starts_with("struct ")
        || trimmed.starts_with("enum ")
        || trimmed.starts_with("trait ")
        || trimmed.starts_with("type ")
        || trimmed.starts_with("const ")
        || trimmed.starts_with("static ")
        || trimmed.starts_with("mod ")
        || trimmed.starts_with("async fn ")
        || trimmed.starts_with("unsafe fn ")
        || trimmed.starts_with("unsafe trait ")
}

// -------
// JS/TS
// -------

fn extract_js_ts_symbols(lines: &[&str]) -> Vec<Symbol> {
    let mut symbols = Vec::new();

    for (i, line) in lines.iter().enumerate() {
        let trimmed = line.trim();

        if trimmed.starts_with("//") || trimmed.starts_with('*') || trimmed.starts_with("/*") {
            continue;
        }

        let is_public = is_js_export(trimmed);
        let is_internal = !is_public && is_js_internal(trimmed);

        if is_public || is_internal {
            symbols.push(Symbol {
                is_public,
                is_documented: has_doc_comment(lines, i),
            });
        }
    }

    symbols
}

fn is_js_export(trimmed: &str) -> bool {
    trimmed.starts_with("export function ")
        || trimmed.starts_with("export async function ")
        || trimmed.starts_with("export class ")
        || trimmed.starts_with("export const ")
        || trimmed.starts_with("export let ")
        || trimmed.starts_with("export default ")
        || trimmed.starts_with("export interface ")
        || trimmed.starts_with("export type ")
        || trimmed.starts_with("export enum ")
        || trimmed.starts_with("export abstract class ")
}

fn is_js_internal(trimmed: &str) -> bool {
    trimmed.starts_with("function ")
        || trimmed.starts_with("async function ")
        || trimmed.starts_with("class ")
        || trimmed.starts_with("const ")
        || trimmed.starts_with("let ")
        || trimmed.starts_with("interface ")
        || trimmed.starts_with("type ")
        || trimmed.starts_with("enum ")
}

// -------
// Python
// -------

fn extract_python_symbols(lines: &[&str]) -> Vec<Symbol> {
    let mut symbols = Vec::new();

    for (i, line) in lines.iter().enumerate() {
        let trimmed = line.trim();

        // Only consider top-level items (no leading whitespace)
        if line.starts_with(' ') || line.starts_with('\t') {
            continue;
        }
        if trimmed.starts_with('#') || trimmed.is_empty() {
            continue;
        }

        let is_symbol = trimmed.starts_with("def ")
            || trimmed.starts_with("async def ")
            || trimmed.starts_with("class ");

        if is_symbol {
            let name = extract_python_name(trimmed);
            let is_public = !name.starts_with('_');
            let documented = has_python_docstring(lines, i);
            symbols.push(Symbol {
                is_public,
                is_documented: documented || has_doc_comment(lines, i),
            });
        }
    }

    symbols
}

fn extract_python_name(trimmed: &str) -> String {
    let rest = if let Some(r) = trimmed.strip_prefix("async def ") {
        r
    } else if let Some(r) = trimmed.strip_prefix("def ") {
        r
    } else if let Some(r) = trimmed.strip_prefix("class ") {
        r
    } else {
        return String::new();
    };

    rest.chars()
        .take_while(|c| c.is_alphanumeric() || *c == '_')
        .collect()
}

/// Check if the line after the def/class has a docstring.
fn has_python_docstring(lines: &[&str], idx: usize) -> bool {
    // Look for a docstring in the lines following the definition
    for line in lines.iter().take((idx + 3).min(lines.len())).skip(idx + 1) {
        let t = line.trim();
        if t.is_empty() {
            continue;
        }
        return t.starts_with("\"\"\"") || t.starts_with("'''") || t.starts_with("r\"\"\"");
    }
    false
}

// -------
// Go
// -------

fn extract_go_symbols(lines: &[&str]) -> Vec<Symbol> {
    let mut symbols = Vec::new();

    for (i, line) in lines.iter().enumerate() {
        let trimmed = line.trim();

        if trimmed.starts_with("//") || trimmed.starts_with("/*") {
            continue;
        }

        if let Some(name) = extract_go_item_name(trimmed) {
            // In Go, items starting with uppercase are public
            let first_char = name.chars().next().unwrap_or('_');
            let is_public = first_char.is_uppercase();
            symbols.push(Symbol {
                is_public,
                is_documented: has_doc_comment(lines, i),
            });
        }
    }

    symbols
}

fn extract_go_item_name(trimmed: &str) -> Option<String> {
    // func Name or func (receiver) Name
    if let Some(rest) = trimmed.strip_prefix("func ") {
        let rest = if rest.starts_with('(') {
            // Method receiver: skip to closing paren
            if let Some(close) = rest.find(')') {
                rest[close + 1..].trim_start()
            } else {
                return None;
            }
        } else {
            rest
        };
        let name: String = rest
            .chars()
            .take_while(|c| c.is_alphanumeric() || *c == '_')
            .collect();
        if !name.is_empty() {
            return Some(name);
        }
    }

    // type Name struct/interface
    if let Some(rest) = trimmed.strip_prefix("type ") {
        let name: String = rest
            .chars()
            .take_while(|c| c.is_alphanumeric() || *c == '_')
            .collect();
        if !name.is_empty() {
            return Some(name);
        }
    }

    // var Name or const Name (top-level)
    for prefix in &["var ", "const "] {
        if let Some(rest) = trimmed.strip_prefix(prefix) {
            let name: String = rest
                .chars()
                .take_while(|c| c.is_alphanumeric() || *c == '_')
                .collect();
            if !name.is_empty() {
                return Some(name);
            }
        }
    }

    None
}

// -------
// Java
// -------

fn extract_java_symbols(lines: &[&str]) -> Vec<Symbol> {
    let mut symbols = Vec::new();

    for (i, line) in lines.iter().enumerate() {
        let trimmed = line.trim();

        if trimmed.starts_with("//") || trimmed.starts_with('*') || trimmed.starts_with("/*") {
            continue;
        }

        let is_public = is_java_public(trimmed);
        let is_internal = !is_public && is_java_internal(trimmed);

        if is_public || is_internal {
            symbols.push(Symbol {
                is_public,
                is_documented: has_doc_comment(lines, i),
            });
        }
    }

    symbols
}

fn is_java_public(trimmed: &str) -> bool {
    trimmed.starts_with("public class ")
        || trimmed.starts_with("public interface ")
        || trimmed.starts_with("public enum ")
        || trimmed.starts_with("public static ")
        || trimmed.starts_with("public abstract class ")
        || trimmed.starts_with("public final class ")
        || trimmed.starts_with("public record ")
        || trimmed.starts_with("public sealed ")
        // public return-type method(
        || (trimmed.starts_with("public ")
            && (trimmed.contains('(') || trimmed.contains(" class ") || trimmed.contains(" interface ")))
}

fn is_java_internal(trimmed: &str) -> bool {
    // private/protected/package-private items
    trimmed.starts_with("private ")
        || trimmed.starts_with("protected ")
        || trimmed.starts_with("class ")
        || trimmed.starts_with("interface ")
        || trimmed.starts_with("enum ")
        || trimmed.starts_with("abstract class ")
        || trimmed.starts_with("final class ")
        || trimmed.starts_with("static ")
        || trimmed.starts_with("record ")
}

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
    let mut lang_totals: BTreeMap<String, (usize, usize, usize)> = BTreeMap::new(); // (total, public, internal)

    // Per-module accumulators
    let mut module_totals: BTreeMap<String, (usize, usize)> = BTreeMap::new(); // (total, public)

    // Top exporters
    let mut exporters: Vec<ApiExportItem> = Vec::new();

    for rel in files {
        if limits.max_bytes.is_some_and(|limit| total_bytes >= limit) {
            break;
        }

        let rel_str = rel.to_string_lossy().replace('\\', "/");
        let row = match row_map.get(&rel_str) {
            Some(r) => *r,
            None => continue,
        };

        if !is_api_surface_lang(&row.lang) {
            continue;
        }

        let path = root.join(rel);
        let bytes = match tokmd_content::read_head(&path, per_file_limit) {
            Ok(b) => b,
            Err(_) => continue,
        };
        total_bytes += bytes.len() as u64;

        if !tokmd_content::is_text_like(&bytes) {
            continue;
        }

        let text = String::from_utf8_lossy(&bytes);
        let symbols = extract_symbols(&row.lang, &text);

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
        let lang_key = row.lang.clone();
        let entry = lang_totals.entry(lang_key).or_insert((0, 0, 0));
        entry.0 += file_total;
        entry.1 += file_public;
        entry.2 += file_internal;

        // Per-module
        let mod_entry = module_totals.entry(row.module.clone()).or_insert((0, 0));
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
                lang,
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
                module,
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
    fn unsupported_langs() {
        assert!(!is_api_surface_lang("Markdown"));
        assert!(!is_api_surface_lang("JSON"));
        assert!(!is_api_surface_lang("CSS"));
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
}
