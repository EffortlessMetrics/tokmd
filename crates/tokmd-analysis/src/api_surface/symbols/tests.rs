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
    let code = "export interface IFoo {}\nexport type Bar = string;\nexport enum Baz { A, B }\n";
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
