//! Feature-gated AST foundation.
//!
//! This module is intentionally shadow-only. It records the initial capability
//! and artifact contract from ADR-0008 without changing default receipt
//! semantics.

mod capability;
mod rust;
mod shadow;

pub use capability::{
    AST_SHADOW_SCHEMA_VERSION, AstCapability, AstLanguage, AstParserStatus, capabilities,
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
        AST_SHADOW_SCHEMA_VERSION, AstLanguage, AstParserStatus, capabilities,
        default_shadow_artifacts,
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
}
