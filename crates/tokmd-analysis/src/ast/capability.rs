use super::rust;

pub const AST_SHADOW_SCHEMA_VERSION: &str = "tokmd.ast_shadow.v1";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AstLanguage {
    Rust,
}

impl AstLanguage {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Rust => "rust",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AstParserStatus {
    ParserBackedShadow,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AstCapability {
    pub language: AstLanguage,
    pub parser_crate: &'static str,
    pub parser_status: AstParserStatus,
    pub shadow_only: bool,
    pub changes_default_receipts: bool,
}

impl AstCapability {
    pub const fn parser_backed_shadow(language: AstLanguage, parser_crate: &'static str) -> Self {
        Self {
            language,
            parser_crate,
            parser_status: AstParserStatus::ParserBackedShadow,
            shadow_only: true,
            changes_default_receipts: false,
        }
    }
}

#[must_use]
pub fn capabilities() -> &'static [AstCapability] {
    rust::CAPABILITIES
}

#[cfg(test)]
mod tests {
    use super::{AstCapability, AstLanguage, AstParserStatus};

    #[test]
    fn parser_backed_shadow_constructor_never_changes_default_receipts() {
        let capability = AstCapability::parser_backed_shadow(AstLanguage::Rust, "tree-sitter-rust");

        assert_eq!(capability.language.as_str(), "rust");
        assert_eq!(
            capability.parser_status,
            AstParserStatus::ParserBackedShadow
        );
        assert!(capability.shadow_only);
        assert!(!capability.changes_default_receipts);
    }
}
