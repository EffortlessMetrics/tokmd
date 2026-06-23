use super::capability::{AstLanguage, SYNTAX_RECEIPT_SCHEMA_VERSION};
use super::facts::SyntaxFacts;
use super::python::extract_python_facts;
use super::rust::extract_rust_facts;
use super::typescript::extract_typescript_facts;
use serde_json::{Value, json};
use tree_sitter::{Language, Parser};

pub const DEFAULT_MAX_SYNTAX_BYTES: usize = 1_048_576;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SyntaxParserCapability {
    pub language: AstLanguage,
    pub parser_crate: &'static str,
    pub grammar_symbol: &'static str,
    pub extensions: &'static [&'static str],
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SyntaxParseStatus {
    Complete,
    ParseDegraded,
    ParserFailed,
    SkippedGeneratedOrVendor,
    SkippedTooLarge,
    UnsupportedLanguage,
}

impl SyntaxParseStatus {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Complete => "complete",
            Self::ParseDegraded => "parse_degraded",
            Self::ParserFailed => "parser_failed",
            Self::SkippedGeneratedOrVendor => "skipped_generated_or_vendor",
            Self::SkippedTooLarge => "skipped_too_large",
            Self::UnsupportedLanguage => "unsupported_language",
        }
    }

    #[must_use]
    pub const fn is_advisory(self) -> bool {
        !matches!(self, Self::Complete)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SyntaxParseOptions {
    pub max_bytes: usize,
    pub skip_generated_vendor: bool,
}

impl Default for SyntaxParseOptions {
    fn default() -> Self {
        Self {
            max_bytes: DEFAULT_MAX_SYNTAX_BYTES,
            skip_generated_vendor: true,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SyntaxParseReceipt {
    pub path: String,
    pub language: Option<AstLanguage>,
    pub parser_crate: Option<&'static str>,
    pub grammar_symbol: Option<&'static str>,
    pub status: SyntaxParseStatus,
    pub reason: Option<String>,
    pub source_bytes: usize,
    pub root_kind: Option<String>,
    pub has_error: bool,
    pub facts: SyntaxFacts,
}

impl SyntaxParseReceipt {
    #[must_use]
    pub fn to_value(&self) -> Value {
        json!({
            "schema": SYNTAX_RECEIPT_SCHEMA_VERSION,
            "path": self.path.as_str(),
            "language": self.language.map(AstLanguage::as_str),
            "parser_crate": self.parser_crate,
            "grammar_symbol": self.grammar_symbol,
            "status": self.status.as_str(),
            "advisory": self.status.is_advisory(),
            "reason": self.reason.as_deref(),
            "source_bytes": self.source_bytes,
            "root_kind": self.root_kind.as_deref(),
            "has_error": self.has_error,
            "symbols": self.facts.symbols_value(),
            "imports": self.facts.imports_value(),
            "exports": self.facts.exports_value(),
            "call_sites": self.facts.call_sites_value(),
            "risk_seams": self.facts.risk_seams_value(),
            "review_signals": self.facts.review_signals_value(),
        })
    }
}

pub const RUST_SYNTAX_CAPABILITY: SyntaxParserCapability = SyntaxParserCapability {
    language: AstLanguage::Rust,
    parser_crate: "tree-sitter-rust",
    grammar_symbol: "tree_sitter_rust",
    extensions: &["rs"],
};

pub const TYPESCRIPT_SYNTAX_CAPABILITY: SyntaxParserCapability = SyntaxParserCapability {
    language: AstLanguage::TypeScript,
    parser_crate: "tree-sitter-typescript",
    grammar_symbol: "tree_sitter_typescript",
    extensions: &["ts", "mts", "cts"],
};

pub const TSX_SYNTAX_CAPABILITY: SyntaxParserCapability = SyntaxParserCapability {
    language: AstLanguage::Tsx,
    parser_crate: "tree-sitter-typescript",
    grammar_symbol: "tree_sitter_tsx",
    extensions: &["tsx"],
};

pub const PYTHON_SYNTAX_CAPABILITY: SyntaxParserCapability = SyntaxParserCapability {
    language: AstLanguage::Python,
    parser_crate: "tree-sitter-python",
    grammar_symbol: "tree_sitter_python",
    extensions: &["py", "pyw"],
};

static SYNTAX_CAPABILITIES: &[SyntaxParserCapability] = &[
    RUST_SYNTAX_CAPABILITY,
    TYPESCRIPT_SYNTAX_CAPABILITY,
    TSX_SYNTAX_CAPABILITY,
    PYTHON_SYNTAX_CAPABILITY,
];

#[must_use]
pub fn syntax_capabilities() -> &'static [SyntaxParserCapability] {
    SYNTAX_CAPABILITIES
}

#[must_use]
pub fn syntax_capability_for_path(path: &str) -> Option<&'static SyntaxParserCapability> {
    let extension = syntax_extension(path)?;
    SYNTAX_CAPABILITIES
        .iter()
        .find(|capability| capability.extensions.contains(&extension.as_str()))
}

#[must_use]
pub fn parse_syntax_receipt(
    path: &str,
    source: &str,
    options: SyntaxParseOptions,
) -> SyntaxParseReceipt {
    let normalized_path = normalize_syntax_path(path);
    let source_bytes = source.len();

    if options.skip_generated_vendor
        && let Some(reason) = generated_or_vendor_reason(&normalized_path)
    {
        return advisory_receipt(
            normalized_path,
            None,
            SyntaxParseStatus::SkippedGeneratedOrVendor,
            reason,
            source_bytes,
        );
    }

    let capability = match syntax_capability_for_path(&normalized_path) {
        Some(capability) => capability,
        None => {
            return advisory_receipt(
                normalized_path,
                None,
                SyntaxParseStatus::UnsupportedLanguage,
                "no locked Tree-sitter parser for file extension",
                source_bytes,
            );
        }
    };

    if source_bytes > options.max_bytes {
        return advisory_receipt(
            normalized_path,
            Some(capability),
            SyntaxParseStatus::SkippedTooLarge,
            format!(
                "file has {source_bytes} bytes, above syntax parser limit {}",
                options.max_bytes
            ),
            source_bytes,
        );
    }

    let mut parser = Parser::new();
    let language: Language = match capability.language {
        AstLanguage::Python => tree_sitter_python::LANGUAGE.into(),
        AstLanguage::Rust => tree_sitter_rust::LANGUAGE.into(),
        AstLanguage::TypeScript => tree_sitter_typescript::LANGUAGE_TYPESCRIPT.into(),
        AstLanguage::Tsx => tree_sitter_typescript::LANGUAGE_TSX.into(),
    };

    if let Err(error) = parser.set_language(&language) {
        return advisory_receipt(
            normalized_path,
            Some(capability),
            SyntaxParseStatus::ParserFailed,
            format!("failed to load Tree-sitter parser: {error}"),
            source_bytes,
        );
    }

    let Some(tree) = parser.parse(source, None) else {
        return advisory_receipt(
            normalized_path,
            Some(capability),
            SyntaxParseStatus::ParserFailed,
            "Tree-sitter parser returned no tree",
            source_bytes,
        );
    };

    let root = tree.root_node();
    let has_error = root.has_error();
    let facts = match capability.language {
        AstLanguage::Rust => extract_rust_facts(root, source),
        AstLanguage::TypeScript | AstLanguage::Tsx => extract_typescript_facts(root, source),
        AstLanguage::Python => extract_python_facts(root, source),
    };

    SyntaxParseReceipt {
        path: normalized_path,
        language: Some(capability.language),
        parser_crate: Some(capability.parser_crate),
        grammar_symbol: Some(capability.grammar_symbol),
        status: if has_error {
            SyntaxParseStatus::ParseDegraded
        } else {
            SyntaxParseStatus::Complete
        },
        reason: has_error.then(|| "Tree-sitter parsed with syntax errors".to_owned()),
        source_bytes,
        root_kind: Some(root.kind().to_owned()),
        has_error,
        facts,
    }
}

fn advisory_receipt(
    path: String,
    capability: Option<&SyntaxParserCapability>,
    status: SyntaxParseStatus,
    reason: impl Into<String>,
    source_bytes: usize,
) -> SyntaxParseReceipt {
    SyntaxParseReceipt {
        path,
        language: capability.map(|capability| capability.language),
        parser_crate: capability.map(|capability| capability.parser_crate),
        grammar_symbol: capability.map(|capability| capability.grammar_symbol),
        status,
        reason: Some(reason.into()),
        source_bytes,
        root_kind: None,
        has_error: false,
        facts: SyntaxFacts::default(),
    }
}

#[must_use]
pub fn normalize_syntax_path(path: &str) -> String {
    path.replace('\\', "/")
        .trim_start_matches("./")
        .trim_start_matches('/')
        .to_owned()
}

fn syntax_extension(path: &str) -> Option<String> {
    normalize_syntax_path(path)
        .rsplit_once('.')
        .map(|(_, extension)| extension.to_ascii_lowercase())
}

fn generated_or_vendor_reason(path: &str) -> Option<&'static str> {
    let normalized = normalize_syntax_path(path).to_ascii_lowercase();
    let segments = normalized.split('/').collect::<Vec<_>>();
    if segments
        .iter()
        .any(|segment| matches!(*segment, "vendor" | "node_modules"))
    {
        return Some("generated/vendor policy excluded this file");
    }
    if normalized.contains("/generated/")
        || normalized.contains(".generated.")
        || normalized.contains(".gen.")
    {
        return Some("generated/vendor policy excluded this file");
    }
    None
}

#[cfg(test)]
mod tests {
    use super::{
        SyntaxParseOptions, SyntaxParseStatus, parse_syntax_receipt, syntax_capabilities,
        syntax_capability_for_path,
    };
    use serde_json::json;

    #[test]
    fn registry_locks_supported_parsers_and_extensions() {
        let capabilities = syntax_capabilities();

        assert_eq!(capabilities.len(), 4);
        assert_eq!(
            capabilities
                .iter()
                .map(|capability| (
                    capability.language.as_str(),
                    capability.parser_crate,
                    capability.grammar_symbol,
                    capability.extensions
                ))
                .collect::<Vec<_>>(),
            vec![
                ("rust", "tree-sitter-rust", "tree_sitter_rust", &["rs"][..]),
                (
                    "typescript",
                    "tree-sitter-typescript",
                    "tree_sitter_typescript",
                    &["ts", "mts", "cts"][..]
                ),
                (
                    "tsx",
                    "tree-sitter-typescript",
                    "tree_sitter_tsx",
                    &["tsx"][..]
                ),
                (
                    "python",
                    "tree-sitter-python",
                    "tree_sitter_python",
                    &["py", "pyw"][..]
                ),
            ]
        );
    }

    #[test]
    fn routes_paths_to_locked_capabilities() {
        assert_eq!(
            syntax_capability_for_path("src/main.rs")
                .unwrap()
                .language
                .as_str(),
            "rust"
        );
        assert_eq!(
            syntax_capability_for_path("src/app.TS")
                .unwrap()
                .language
                .as_str(),
            "typescript"
        );
        assert_eq!(
            syntax_capability_for_path("src/app.tsx")
                .unwrap()
                .language
                .as_str(),
            "tsx"
        );
        assert_eq!(
            syntax_capability_for_path("tools/script.py")
                .unwrap()
                .language
                .as_str(),
            "python"
        );
        assert!(syntax_capability_for_path("README.md").is_none());
    }

    #[test]
    fn parses_supported_languages_without_network_or_global_state() {
        let cases = [
            ("src/lib.rs", "fn main() {}\n", "rust", "source_file"),
            (
                "src/app.ts",
                "export function run(value: number) { return value + 1; }\n",
                "typescript",
                "program",
            ),
            (
                "src/App.tsx",
                "export function App() { return <main>Hello</main>; }\n",
                "tsx",
                "program",
            ),
            (
                "tools/run.py",
                "def run(value):\n    return value + 1\n",
                "python",
                "module",
            ),
        ];

        for (path, source, language, root_kind) in cases {
            let receipt = parse_syntax_receipt(path, source, SyntaxParseOptions::default());

            assert_eq!(receipt.status, SyntaxParseStatus::Complete, "{path}");
            assert_eq!(receipt.language.unwrap().as_str(), language);
            assert_eq!(receipt.root_kind.as_deref(), Some(root_kind));
            assert!(!receipt.has_error);
            assert_eq!(receipt.to_value()["status"], "complete");
            assert_eq!(receipt.to_value()["advisory"], false);
        }
    }

    #[test]
    fn extracts_typescript_review_facts_from_native_boundary_fixture() {
        let receipt = parse_syntax_receipt(
            "src/runtime/native_boundary.ts",
            include_str!("../../../../fixtures/syntax/typescript/native_boundary.ts"),
            SyntaxParseOptions::default(),
        );
        let value = receipt.to_value();

        assert_eq!(receipt.status, SyntaxParseStatus::Complete);
        let symbol_names = value["symbols"]
            .as_array()
            .unwrap()
            .iter()
            .filter_map(|entry| entry["name"].as_str())
            .collect::<Vec<_>>();
        for expected in ["bindNative", "loadPlugin", "RuntimeBridge", "main"] {
            assert!(symbol_names.contains(&expected), "{expected}");
        }
        assert!(
            value["imports"]
                .as_array()
                .unwrap()
                .iter()
                .any(|entry| entry["module"] == "bun:ffi" && entry["dynamic"] == false)
        );
        assert!(
            value["imports"]
                .as_array()
                .unwrap()
                .iter()
                .any(|entry| entry["kind"] == "dynamic" && entry["dynamic"] == true)
        );
        assert!(
            value["exports"]
                .as_array()
                .unwrap()
                .iter()
                .any(|entry| entry["name"] == "bindNative")
        );
        assert!(
            value["call_sites"]
                .as_array()
                .unwrap()
                .iter()
                .any(|entry| entry["callee"] == "Bun.serve")
        );
        assert!(
            value["risk_seams"]
                .as_array()
                .unwrap()
                .iter()
                .any(|entry| entry["kind"] == "risky_cast")
        );
        assert!(
            value["risk_seams"]
                .as_array()
                .unwrap()
                .iter()
                .any(|entry| entry["kind"] == "dynamic_import")
        );
        assert!(
            value["risk_seams"]
                .as_array()
                .unwrap()
                .iter()
                .any(|entry| entry["kind"] == "native_boundary_hint")
        );
        assert!(
            value["risk_seams"]
                .as_array()
                .unwrap()
                .iter()
                .any(|entry| entry["kind"] == "entrypoint")
        );
    }

    #[test]
    fn extracts_tsx_exports_and_cast_seams() {
        let receipt = parse_syntax_receipt(
            "src/runtime/component.tsx",
            include_str!("../../../../fixtures/syntax/typescript/component.tsx"),
            SyntaxParseOptions::default(),
        );
        let value = receipt.to_value();

        assert_eq!(receipt.status, SyntaxParseStatus::Complete);
        assert_eq!(receipt.root_kind.as_deref(), Some("program"));
        assert!(
            value["symbols"]
                .as_array()
                .unwrap()
                .iter()
                .any(|entry| entry["name"] == "NativeButton" && entry["public_surface"] == true)
        );
        assert!(
            value["exports"]
                .as_array()
                .unwrap()
                .iter()
                .any(|entry| entry["name"] == "NativeButton")
        );
        assert!(
            value["risk_seams"]
                .as_array()
                .unwrap()
                .iter()
                .any(|entry| entry["kind"] == "risky_cast")
        );
    }

    #[test]
    fn extracts_rust_review_facts_from_panic_seam_fixture() {
        let receipt = parse_syntax_receipt(
            "src/runtime/panic_seams.rs",
            include_str!("../../../../fixtures/syntax/rust/panic_seams.rs"),
            SyntaxParseOptions::default(),
        );
        let value = receipt.to_value();

        assert_eq!(receipt.status, SyntaxParseStatus::Complete);
        assert_eq!(receipt.root_kind.as_deref(), Some("source_file"));
        assert!(
            value["symbols"]
                .as_array()
                .unwrap()
                .iter()
                .any(|entry| entry["name"] == "load_packet" && entry["public_surface"] == true)
        );
        assert!(
            value["exports"]
                .as_array()
                .unwrap()
                .iter()
                .any(|entry| entry["name"] == "ffi_entry")
        );
        assert!(value["imports"].as_array().unwrap().iter().any(|entry| {
            entry["imported"]
                .as_array()
                .unwrap()
                .iter()
                .any(|name| name.as_str().is_some_and(|name| name.contains("TryFrom")))
        }));
        assert!(
            value["call_sites"]
                .as_array()
                .unwrap()
                .iter()
                .any(|entry| entry["callee"] == "panic!")
        );
        for kind in [
            "expect",
            "fallible_conversion_expect",
            "unwrap",
            "capacity_allocation",
            "indexing",
            "panic_macro",
            "assert_macro",
            "guard_evidence",
        ] {
            assert!(
                value["risk_seams"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .any(|entry| entry["kind"] == kind),
                "{kind}"
            );
        }
    }

    #[test]
    fn extracts_python_review_facts_from_native_boundary_fixture() {
        let receipt = parse_syntax_receipt(
            "tools/native_boundary.py",
            include_str!("../../../../fixtures/syntax/python/native_boundary.py"),
            SyntaxParseOptions::default(),
        );
        let value = receipt.to_value();

        assert_eq!(receipt.status, SyntaxParseStatus::Complete);
        assert_eq!(receipt.root_kind.as_deref(), Some("module"));
        assert!(
            value["symbols"]
                .as_array()
                .unwrap()
                .iter()
                .any(|entry| entry["kind"] == "module" && entry["name"] == "<module>")
        );
        assert!(
            value["symbols"]
                .as_array()
                .unwrap()
                .iter()
                .any(|entry| entry["name"] == "NativeBridge" && entry["public_surface"] == true)
        );
        assert!(value["imports"].as_array().unwrap().iter().any(|entry| {
            entry["imported"]
                .as_array()
                .unwrap()
                .iter()
                .any(|name| name == "ctypes")
        }));
        assert!(
            value["call_sites"]
                .as_array()
                .unwrap()
                .iter()
                .any(|entry| entry["callee"] == "subprocess.run")
        );
        for kind in [
            "entrypoint",
            "native_boundary_hint",
            "dynamic_import",
            "dynamic_call",
            "subprocess_call",
            "dynamic_eval",
            "file_io",
            "exception_raise",
            "exception_handler",
            "guard_evidence",
        ] {
            assert!(
                value["risk_seams"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .any(|entry| entry["kind"] == kind),
                "{kind}"
            );
        }
    }

    #[test]
    fn normalizes_review_signals_across_supported_languages() {
        let cases = [
            (
                "src/runtime/native_boundary.ts",
                include_str!("../../../../fixtures/syntax/typescript/native_boundary.ts"),
                vec![
                    "native_boundary",
                    "dynamic_execution",
                    "dynamic_import",
                    "entrypoint",
                    "public_surface",
                    "type_assertion",
                ],
            ),
            (
                "src/runtime/panic_seams.rs",
                include_str!("../../../../fixtures/syntax/rust/panic_seams.rs"),
                vec!["panic_seam", "guard_evidence", "public_surface"],
            ),
            (
                "tools/native_boundary.py",
                include_str!("../../../../fixtures/syntax/python/native_boundary.py"),
                vec![
                    "native_boundary",
                    "process_boundary",
                    "dynamic_execution",
                    "dynamic_import",
                    "io_boundary",
                    "exception_path",
                    "public_surface",
                ],
            ),
        ];

        for (path, source, expected_categories) in cases {
            let receipt = parse_syntax_receipt(path, source, SyntaxParseOptions::default());
            let value = receipt.to_value();
            let review_signals = value["review_signals"].as_array().unwrap();
            assert!(!review_signals.is_empty(), "{path}");
            assert_eq!(
                review_signals[0]["severity"], "high",
                "highest-priority signal should sort first for {path}"
            );
            assert_eq!(
                review_signals[0]["score"], 90,
                "highest-priority score should sort first for {path}"
            );

            for category in expected_categories {
                assert!(
                    review_signals
                        .iter()
                        .any(|entry| entry["category"] == category),
                    "{path} missing {category}: {}",
                    serde_json::to_string_pretty(&review_signals).unwrap()
                );
            }
        }
    }

    #[test]
    fn deprioritizes_test_assertion_noise_for_panic_review_ranking() {
        let mixed = parse_syntax_receipt(
            "src/runtime/mixed_panic_and_tests.rs",
            include_str!("../../../../fixtures/syntax/rust/mixed_panic_and_tests.rs"),
            SyntaxParseOptions::default(),
        );
        let mixed_value = mixed.to_value();
        let mixed_signals = mixed_value["review_signals"].as_array().unwrap();
        assert_eq!(mixed_signals[0]["category"], "panic_seam");
        assert_ne!(mixed_signals[0]["kind"], "assert_macro");
        assert_eq!(mixed_signals[0]["score"], 90);

        let test_assert_index = mixed_signals
            .iter()
            .position(|signal| signal["kind"] == "assert_macro" && signal["test_context"] == true)
            .expect("test assertion signal");
        let production_expect_index = mixed_signals
            .iter()
            .position(|signal| {
                signal["kind"] == "expect"
                    && signal
                        .get("test_context")
                        .is_none_or(|value| value != &json!(true))
            })
            .expect("production expect signal");
        assert!(
            production_expect_index < test_assert_index,
            "production panic seams should rank above test assertion noise"
        );

        let panic_seams = parse_syntax_receipt(
            "src/runtime/panic_seams.rs",
            include_str!("../../../../fixtures/syntax/rust/panic_seams.rs"),
            SyntaxParseOptions::default(),
        );
        let panic_value = panic_seams.to_value();
        let signals = panic_value["review_signals"].as_array().unwrap();
        assert_eq!(signals[0]["severity"], "high");
        assert_eq!(signals[0]["score"], 90);
        assert!(
            signals
                .iter()
                .all(|signal| signal.get("test_context") != Some(&json!(true)))
        );

        let parser = parse_syntax_receipt(
            "crates/tokmd-analysis/src/ast/rust.rs",
            include_str!("rust.rs"),
            SyntaxParseOptions::default(),
        );
        let parser_value = parser.to_value();
        let parser_signals = parser_value["review_signals"].as_array().unwrap();
        assert_ne!(parser_signals[0]["kind"], "assert_macro");
        assert!(
            parser_signals[0]["score"].as_u64().unwrap() >= 60,
            "parser source top rank should stay production-relevant"
        );
    }

    #[test]
    fn malformed_syntax_degrades_explicitly() {
        let receipt = parse_syntax_receipt(
            "src/lib.rs",
            "fn ok() {}\nfn broken(",
            SyntaxParseOptions::default(),
        );

        assert_eq!(receipt.status, SyntaxParseStatus::ParseDegraded);
        assert!(receipt.has_error);
        assert_eq!(receipt.to_value()["status"], "parse_degraded");
        assert_eq!(receipt.to_value()["advisory"], true);
        assert!(receipt.reason.unwrap().contains("syntax errors"));
    }

    #[test]
    fn unsupported_language_is_an_advisory_receipt() {
        let receipt = parse_syntax_receipt("README.md", "# docs\n", SyntaxParseOptions::default());

        assert_eq!(receipt.status, SyntaxParseStatus::UnsupportedLanguage);
        assert!(receipt.language.is_none());
        assert_eq!(receipt.to_value()["schema"], "tokmd.syntax_receipt.v1");
        assert_eq!(receipt.to_value()["status"], "unsupported_language");
        assert_eq!(receipt.to_value()["advisory"], true);
        assert!(
            receipt.to_value()["review_signals"]
                .as_array()
                .unwrap()
                .is_empty()
        );
    }

    #[test]
    fn generated_or_vendor_files_are_skipped_by_policy() {
        for path in [
            "vendor/crate/src/lib.rs",
            "node_modules/pkg/index.ts",
            "src/generated/model.py",
            "src/api.generated.ts",
        ] {
            let receipt =
                parse_syntax_receipt(path, "fn ignored() {}", SyntaxParseOptions::default());

            assert_eq!(
                receipt.status,
                SyntaxParseStatus::SkippedGeneratedOrVendor,
                "{path}"
            );
            assert_eq!(receipt.to_value()["advisory"], true);
            assert!(receipt.reason.unwrap().contains("policy"));
        }
    }

    #[test]
    fn huge_files_are_skipped_with_limit_evidence() {
        let receipt = parse_syntax_receipt(
            "src/lib.rs",
            "fn main() {}\n",
            SyntaxParseOptions {
                max_bytes: 4,
                skip_generated_vendor: false,
            },
        );

        assert_eq!(receipt.status, SyntaxParseStatus::SkippedTooLarge);
        assert_eq!(receipt.language.unwrap().as_str(), "rust");
        assert_eq!(receipt.source_bytes, "fn main() {}\n".len());
        assert!(receipt.facts.symbols.is_empty());
        assert!(
            receipt
                .reason
                .unwrap()
                .contains("above syntax parser limit 4")
        );
    }
}
