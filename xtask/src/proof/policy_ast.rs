use serde::{Deserialize, Serialize};

pub const EXPECTED_SCHEMA: &str = "tokmd.proof_policy.v1";
pub const RETIRED_TOKMD_CONFIG: &str = "tokmd-config";

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ProofPolicy {
    pub schema: String,

    #[serde(default)]
    pub defaults: Defaults,

    #[serde(default)]
    pub tools: Tools,

    #[serde(default)]
    pub executor: Executor,

    #[serde(default)]
    pub scope: Vec<Scope>,

    #[serde(default)]
    pub allow: Allow,

    #[serde(default)]
    pub forbid: Forbid,

    #[serde(default)]
    pub dependency_boundary: Vec<DependencyBoundary>,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Defaults {
    pub mutation_timeout_seconds: Option<u64>,
    pub coverage: Option<String>,
    pub fail_on_unknown_non_rust: Option<bool>,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Tools {
    pub rust: Option<String>,
    pub coverage: Option<String>,
    pub javascript: Option<String>,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Executor {
    pub family: Option<String>,
    pub ci_execution: Option<CiExecution>,
    pub max_dry_run_commands: Option<usize>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum CiExecution {
    ExplicitOptIn,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ScopeKind {
    Rust,
    NonRust,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Scope {
    pub name: String,
    pub kind: ScopeKind,

    #[serde(default)]
    pub paths: Vec<String>,

    #[serde(default)]
    pub packages: Vec<String>,

    #[serde(default)]
    pub proof: Vec<String>,

    #[serde(default)]
    pub mutation: bool,

    #[serde(default)]
    pub coverage: bool,

    #[serde(default)]
    pub reason: Option<String>,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Allow {
    #[serde(default)]
    pub workspace_area: Vec<WorkspaceAreaAllow>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct WorkspaceAreaAllow {
    pub name: String,

    #[serde(default)]
    pub paths: Vec<String>,

    pub reason: String,

    #[serde(default)]
    pub discourage: Vec<String>,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Forbid {
    #[serde(default)]
    pub fixture_blob: Vec<FixtureBlobRule>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct FixtureBlobRule {
    pub name: String,

    #[serde(default)]
    pub extensions: Vec<String>,

    #[serde(default)]
    pub markers: Vec<String>,

    #[serde(default)]
    pub allow: Vec<String>,

    pub reason: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct DependencyBoundary {
    pub name: String,

    #[serde(default)]
    pub packages: Vec<String>,

    #[serde(default)]
    pub forbid: Vec<String>,

    pub reason: String,
}
