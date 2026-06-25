//! Feature-gated syntax and AST foundation.
//!
//! AST shadow artifacts remain comparison-only. The syntax parser registry is
//! also feature-gated and produces advisory receipts without changing default
//! `tokmd analyze`, cockpit, context, handoff, FFI, Python, Node, or WASM
//! receipt semantics.

mod capability;
mod facts;
mod panic_seam;
mod python;
mod registry;
mod rust;
mod shadow;
mod typescript;

pub use capability::{
    AST_SHADOW_SCHEMA_VERSION, AstCapability, AstLanguage, AstParserStatus,
    SYNTAX_RECEIPT_SCHEMA_VERSION, capabilities,
};
pub use facts::{
    SyntaxCallSite, SyntaxExport, SyntaxFacts, SyntaxImport, SyntaxReviewSeverity,
    SyntaxReviewSignal, SyntaxRiskSeam, SyntaxSpan, SyntaxSymbol,
};
pub use registry::{
    DEFAULT_MAX_SYNTAX_BYTES, SyntaxParseOptions, SyntaxParseReceipt, SyntaxParseStatus,
    SyntaxParserCapability, normalize_syntax_path, parse_syntax_receipt, syntax_capabilities,
    syntax_capability_for_path,
};
pub use rust::{RustAstError, RustAstShadow, RustLandmark, RustLandmarkKind, parse_rust_landmarks};
pub use shadow::{
    DEFAULT_SHADOW_OUTPUT_DIR, ShadowArtifactError, ShadowArtifactPaths, ShadowArtifactSet,
    ShadowArtifacts, ShadowFileInput, ShadowLandmark, build_shadow_artifacts,
    default_shadow_artifacts, normalize_shadow_path, write_shadow_artifacts,
};

#[cfg(test)]
mod tests {
    use super::{
        AST_SHADOW_SCHEMA_VERSION, AstLanguage, AstParserStatus, SYNTAX_RECEIPT_SCHEMA_VERSION,
        capabilities, default_shadow_artifacts, syntax_capabilities,
    };

    #[test]
    fn rust_capability_is_shadow_only_and_not_default_receipts() {
        let capabilities = capabilities();

        assert_eq!(capabilities.len(), 1);
        assert_eq!(capabilities[0].language, AstLanguage::Rust);
        assert_eq!(
            capabilities[0].parser_status,
            AstParserStatus::ParserBackedShadow
        );
        assert!(capabilities[0].shadow_only);
        assert!(!capabilities[0].changes_default_receipts);
    }

    #[test]
    fn shadow_artifact_contract_is_stable() {
        let artifacts = default_shadow_artifacts();

        assert_eq!(artifacts.output_dir, "target/tokmd-ast-shadow");
        assert_eq!(artifacts.heuristic, "heuristic.json");
        assert_eq!(artifacts.ast, "ast.json");
        assert_eq!(artifacts.diff, "diff.json");
    }

    #[test]
    fn shadow_schema_name_is_ast_scoped() {
        assert_eq!(AST_SHADOW_SCHEMA_VERSION, "tokmd.ast_shadow.v1");
    }

    #[test]
    fn syntax_registry_is_feature_gated_and_does_not_expand_shadow_capabilities() {
        assert_eq!(SYNTAX_RECEIPT_SCHEMA_VERSION, "tokmd.syntax_receipt.v1");
        assert_eq!(syntax_capabilities().len(), 4);
        assert_eq!(capabilities().len(), 1);
        assert_eq!(capabilities()[0].language, AstLanguage::Rust);
    }
}
