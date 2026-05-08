use super::capability::{AstCapability, AstLanguage};

pub const TREE_SITTER_RUST_CRATE: &str = "tree-sitter-rust";
pub const RUST_CAPABILITY: AstCapability =
    AstCapability::planned_shadow(AstLanguage::Rust, TREE_SITTER_RUST_CRATE);
pub static CAPABILITIES: &[AstCapability] = &[RUST_CAPABILITY];
