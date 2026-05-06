use super::policy_ast::ProofPolicy;
use anyhow::{Context, Result};
use std::path::Path;

pub fn load_policy(path: &Path) -> Result<ProofPolicy> {
    let content = std::fs::read_to_string(path)
        .with_context(|| format!("failed to read {}", path.display()))?;
    parse_policy_str(&content).with_context(|| format!("failed to parse {}", path.display()))
}

pub fn parse_policy_str(content: &str) -> Result<ProofPolicy> {
    toml::from_str(content).context("proof policy TOML is invalid")
}

#[cfg(test)]
mod tests {
    use super::parse_policy_str;

    #[test]
    fn rejects_unknown_top_level_keys() {
        let err = parse_policy_str(
            r#"
schema = "tokmd.proof_policy.v1"
surprise = true
"#,
        )
        .expect_err("unknown top-level keys should fail");

        assert!(err.to_string().contains("proof policy TOML is invalid"));
    }

    #[test]
    fn rejects_unknown_nested_keys() {
        let err = parse_policy_str(
            r#"
schema = "tokmd.proof_policy.v1"

[[scope]]
name = "core"
kind = "rust"
paths = ["crates/tokmd-core/**"]
proof = ["cargo test -p tokmd-core"]
unexpected = "nope"
"#,
        )
        .expect_err("unknown scope keys should fail");

        assert!(err.to_string().contains("proof policy TOML is invalid"));
    }
}
