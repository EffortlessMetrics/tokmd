pub const DEFAULT_SHADOW_OUTPUT_DIR: &str = "target/tokmd-ast-shadow";
pub const HEURISTIC_ARTIFACT: &str = "heuristic.json";
pub const AST_ARTIFACT: &str = "ast.json";
pub const DIFF_ARTIFACT: &str = "diff.json";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ShadowArtifactSet {
    pub output_dir: &'static str,
    pub heuristic: &'static str,
    pub ast: &'static str,
    pub diff: &'static str,
}

pub const DEFAULT_SHADOW_ARTIFACTS: ShadowArtifactSet = ShadowArtifactSet {
    output_dir: DEFAULT_SHADOW_OUTPUT_DIR,
    heuristic: HEURISTIC_ARTIFACT,
    ast: AST_ARTIFACT,
    diff: DIFF_ARTIFACT,
};

#[must_use]
pub const fn default_shadow_artifacts() -> &'static ShadowArtifactSet {
    &DEFAULT_SHADOW_ARTIFACTS
}

#[cfg(test)]
mod tests {
    use super::{AST_ARTIFACT, DEFAULT_SHADOW_ARTIFACTS, DIFF_ARTIFACT, HEURISTIC_ARTIFACT};

    #[test]
    fn artifact_names_match_adr_contract() {
        assert_eq!(DEFAULT_SHADOW_ARTIFACTS.heuristic, HEURISTIC_ARTIFACT);
        assert_eq!(DEFAULT_SHADOW_ARTIFACTS.ast, AST_ARTIFACT);
        assert_eq!(DEFAULT_SHADOW_ARTIFACTS.diff, DIFF_ARTIFACT);
    }
}
