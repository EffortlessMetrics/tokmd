use clap::{Args, Parser, Subcommand, ValueEnum};

#[derive(Parser, Debug)]
#[command(name = "xtask")]
#[command(about = "Development tasks for tokmd", long_about = None)]
pub struct XtaskCli {
    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand, Debug, Clone)]
pub enum Commands {
    /// Bump version across the entire workspace
    Bump(BumpArgs),
    /// Publish all crates in dependency order
    Publish(PublishArgs),
    /// Audit the publish surface and optional package/publish dry-run closure checks
    PublishSurface(PublishSurfaceArgs),
    /// Generate PR cockpit metrics for CI
    Cockpit(CockpitArgs),
    /// Manage documentation and verify examples
    Docs(DocsArgs),
    /// Validate the Rust-native proof policy
    ProofPolicy(ProofPolicyArgs),
    /// Discover proof scopes affected by a git diff
    Affected(AffectedArgs),
    /// Print proof command plans without executing them
    Proof(ProofArgs),
    /// Verify generated proof artifacts agree without executing planned commands
    ProofArtifactsCheck(ProofArtifactsCheckArgs),
    /// Verify opted-in executed proof artifacts agree and passed
    ProofExecutionArtifactsCheck(ProofArtifactsCheckArgs),
    /// Write a compact observation report for opted-in executed proof artifacts
    ProofExecutionObservation(ProofExecutionObservationArgs),
    /// Summarize one or more proof executor observation artifacts
    ProofExecutionObservationsSummary(ProofExecutionObservationsSummaryArgs),
    /// Verify all release-facing version surfaces are in sync
    VersionConsistency(VersionConsistencyArgs),
    /// Verify dependency boundaries for analysis microcrates
    BoundariesCheck(BoundariesCheckArgs),
    /// Reject committed crypto fixture blobs outside approved paths
    FixtureBlobsCheck(FixtureBlobsCheckArgs),
    /// Run pre-merge quality gate (fmt, check, clippy, test-compile)
    Gate(GateArgs),
    /// Verify workspace Clippy lint policy and debt ledgers
    CheckLintPolicy(LintPolicyArgs),
    /// Verify the workspace panic-family allowlist (semantic no-panic checker)
    CheckNoPanicFamily(NoPanicArgs),
    /// Propose new no-panic allowlist entries from current findings
    NoPanicPropose(NoPanicProposeArgs),
    /// Auto-fix lint issues (fmt + clippy --fix) then verify
    LintFix(LintFixArgs),
    /// Run Cargo through an opt-in local sccache wrapper
    Sccache(SccacheArgs),
    /// Reclaim target/debug space by trimming Windows PDBs and incremental state
    TrimTarget(TrimTargetArgs),
}

#[derive(Args, Debug, Clone, Default)]
pub struct DocsArgs {
    /// Verify that generated documentation blocks are up to date
    #[arg(long)]
    pub check: bool,

    /// Update documentation blocks in place
    #[arg(long)]
    pub update: bool,
}

#[derive(Args, Debug, Clone, Default)]
pub struct VersionConsistencyArgs {}

#[derive(Args, Debug, Clone, Default)]
pub struct LintPolicyArgs {}

#[derive(Args, Debug, Clone, Default)]
pub struct NoPanicArgs {
    /// Emit a machine-readable JSON report instead of human output
    #[arg(long)]
    pub json: bool,

    /// Treat unallowlisted findings as errors (blocking mode).
    ///
    /// Without `--strict`, the checker validates the allowlist schema, expiry,
    /// and stale entries, and reports finding counts, but unallowlisted
    /// findings are advisory. The strict mode is staged behind workspace lint
    /// inheritance and a panic-family debt burn-down.
    #[arg(long)]
    pub strict: bool,
}

#[derive(Args, Debug, Clone)]
pub struct NoPanicProposeArgs {
    /// Output path for proposed allowlist entries
    #[arg(long, default_value = "target/no-panic-proposed-allowlist.toml")]
    pub output: std::path::PathBuf,
}

impl Default for NoPanicProposeArgs {
    fn default() -> Self {
        Self {
            output: std::path::PathBuf::from("target/no-panic-proposed-allowlist.toml"),
        }
    }
}

#[derive(Args, Debug, Clone)]
pub struct ProofPolicyArgs {
    /// Validate the proof policy and print a human-readable summary
    #[arg(long)]
    pub check: bool,

    /// Emit a machine-readable validation report
    #[arg(long)]
    pub json: bool,

    /// Policy file to validate
    #[arg(long, default_value = "ci/proof.toml")]
    pub policy: std::path::PathBuf,
}

#[derive(Args, Debug, Clone)]
pub struct AffectedArgs {
    /// Base git revision for changed-file discovery
    #[arg(long, default_value = "origin/main")]
    pub base: String,

    /// Head git revision for changed-file discovery
    #[arg(long, default_value = "HEAD")]
    pub head: String,

    /// Emit a machine-readable affected-scope report
    #[arg(long)]
    pub json: bool,

    /// Policy file to use for scope matching
    #[arg(long, default_value = "ci/proof.toml")]
    pub policy: std::path::PathBuf,
}

#[derive(Args, Debug, Clone)]
pub struct ProofArgs {
    /// Proof profile to plan
    #[arg(long, value_enum, default_value_t = ProofProfile::Affected)]
    pub profile: ProofProfile,

    /// Base git revision for affected profile discovery
    #[arg(long, default_value = "origin/main")]
    pub base: String,

    /// Head git revision for affected profile discovery
    #[arg(long, default_value = "HEAD")]
    pub head: String,

    /// Print the proof plan without executing commands
    #[arg(long)]
    pub plan: bool,

    /// Write a Markdown summary for the generated proof plan
    #[arg(long, value_name = "PATH")]
    pub summary_md: Option<std::path::PathBuf>,

    /// Write a machine-readable planned evidence summary for the generated proof plan
    #[arg(long, value_name = "PATH")]
    pub evidence_json: Option<std::path::PathBuf>,

    /// Write a prototype executor summary for selected non-required evidence commands
    #[arg(long, value_name = "PATH")]
    pub executor_summary: Option<std::path::PathBuf>,

    /// Write the planner-selected executor command manifest
    #[arg(long, value_name = "PATH")]
    pub executor_manifest: Option<std::path::PathBuf>,

    /// Executor summary mode for selected evidence commands
    #[arg(long, value_enum, default_value_t = ProofExecutorMode::Prototype)]
    pub executor_mode: ProofExecutorMode,

    /// Explicitly opt a CI invocation into future planner-selected evidence execution
    #[arg(long)]
    pub allow_ci_evidence_execution: bool,

    /// Explicitly opt a local invocation into planner-selected evidence execution
    #[arg(long)]
    pub allow_local_evidence_execution: bool,

    /// Policy file to use for scope matching
    #[arg(long, default_value = "ci/proof.toml")]
    pub policy: std::path::PathBuf,
}

#[derive(Args, Debug, Clone)]
pub struct ProofArtifactsCheckArgs {
    /// Executor summary artifact to verify
    #[arg(
        long,
        value_name = "PATH",
        default_value = "target/proof/executor-summary.json"
    )]
    pub executor_summary: std::path::PathBuf,

    /// Executor command manifest artifact to verify
    #[arg(
        long,
        value_name = "PATH",
        default_value = "target/proof/executor-manifest.json"
    )]
    pub executor_manifest: std::path::PathBuf,
}

#[derive(Args, Debug, Clone)]
pub struct ProofExecutionObservationArgs {
    /// Executor summary artifact to observe
    #[arg(
        long,
        value_name = "PATH",
        default_value = "target/proof/executor-summary.json"
    )]
    pub executor_summary: std::path::PathBuf,

    /// Executor command manifest artifact to observe
    #[arg(
        long,
        value_name = "PATH",
        default_value = "target/proof/executor-manifest.json"
    )]
    pub executor_manifest: std::path::PathBuf,

    /// Output path for the compact observation report
    #[arg(
        long,
        value_name = "PATH",
        default_value = "target/proof/proof-executor-observation.json"
    )]
    pub output: std::path::PathBuf,
}

#[derive(Args, Debug, Clone)]
pub struct ProofExecutionObservationsSummaryArgs {
    /// Proof executor observation artifact to include. Repeat for multiple runs.
    #[arg(long = "observation", value_name = "PATH")]
    pub observations: Vec<std::path::PathBuf>,

    /// Directory tree to scan for proof-executor-observation.json artifacts.
    #[arg(long = "observations-dir", value_name = "DIR")]
    pub observation_dirs: Vec<std::path::PathBuf>,

    /// Minimum number of observation artifacts required in the collection.
    #[arg(long, default_value_t = 0)]
    pub min_observations: usize,

    /// Minimum number of executed commands required in the collection.
    #[arg(long, default_value_t = 0)]
    pub min_executed: usize,

    /// Minimum number of distinct scope rows required in the collection.
    #[arg(long, default_value_t = 0)]
    pub min_scopes: usize,

    /// Minimum number of produced artifact paths required in the collection.
    #[arg(long, default_value_t = 0)]
    pub min_artifacts: usize,

    /// Output path for the collection summary. Prints JSON to stdout when omitted.
    #[arg(long, value_name = "PATH")]
    pub output: Option<std::path::PathBuf>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum ProofProfile {
    Fast,
    Affected,
    Release,
    Deep,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum ProofExecutorMode {
    Prototype,
    DryRun,
    Execute,
}

#[derive(Args, Debug, Clone, Default)]
pub struct PublishArgs {
    /// Show publish plan without executing anything (no crates.io interaction)
    #[arg(long)]
    pub plan: bool,

    /// Run in dry-run mode (runs `cargo package --list` per crate for local packaging validation)
    #[arg(long, short = 'n')]
    pub dry_run: bool,

    /// Deprecated alias for --dry-run
    #[arg(long, hide = true)]
    pub verify: bool,

    /// Seconds to wait between publishes for crates.io propagation
    #[arg(long, default_value = "10")]
    pub interval: u64,

    /// Seconds to wait between retries for dependency propagation
    #[arg(long, default_value = "30")]
    pub retry_delay: u64,

    /// Maximum duration (in seconds) for each publish attempt
    #[arg(long, default_value = "300")]
    pub timeout: u64,

    /// Continue on failure instead of aborting
    #[arg(long)]
    pub continue_on_error: bool,

    /// Resume publishing from this crate (skips crates before this one)
    #[arg(long)]
    pub from: Option<String>,

    /// Verbose output
    #[arg(long, short = 'v')]
    pub verbose: bool,

    /// Skip all pre-publish checks
    #[arg(long)]
    pub skip_checks: bool,

    /// Skip running tests
    #[arg(long)]
    pub skip_tests: bool,

    /// Skip git status check
    #[arg(long)]
    pub skip_git_check: bool,

    /// Skip CHANGELOG verification
    #[arg(long)]
    pub skip_changelog_check: bool,

    /// Skip version consistency check
    #[arg(long)]
    pub skip_version_check: bool,

    /// Specific crates to publish (comma-separated). Transitive workspace dependencies are included.
    #[arg(long, value_delimiter = ',')]
    pub crates: Option<Vec<String>>,

    /// Exclude specific crates from publishing (comma-separated). Fails if exclusion would break dependencies.
    #[arg(long, value_delimiter = ',')]
    pub exclude: Option<Vec<String>>,

    /// Create and push git tag after successful publish (e.g., v1.3.0)
    #[arg(long)]
    pub tag: bool,

    /// Custom tag format (use {version} placeholder, e.g., "release-{version}")
    #[arg(long, default_value = "v{version}")]
    pub tag_format: String,

    /// Maximum total seconds to wait for rate limit cooldowns (default 7200)
    #[arg(long, default_value = "7200")]
    pub rate_limit_timeout: u64,

    /// Skip confirmation prompt (required for non-dry-run without TTY)
    #[arg(long, short = 'y')]
    pub yes: bool,
}

#[derive(Args, Debug, Clone, Default)]
pub struct PublishSurfaceArgs {
    /// Emit machine-readable JSON report
    #[arg(long)]
    pub json: bool,

    /// Run cargo package --list for published surface crates
    #[arg(long)]
    pub verify_publish: bool,
}

#[derive(Args, Debug, Clone)]
pub struct BumpArgs {
    /// New version to set (semver format: MAJOR.MINOR.PATCH)
    #[arg(required = true)]
    pub version: String,

    /// Show what would be changed without making changes
    #[arg(long, short = 'n')]
    pub dry_run: bool,

    /// Bump schema versions (format: NAME=VERSION, e.g., SCHEMA_VERSION=3)
    ///
    /// Known schema constants:
    ///   - SCHEMA_VERSION (crates/tokmd-types/src/lib.rs) - core receipts
    ///   - ANALYSIS_SCHEMA_VERSION (crates/tokmd-analysis-types/src/lib.rs)
    ///   - COCKPIT_SCHEMA_VERSION (crates/tokmd-types/src/cockpit.rs)
    ///   - TOOL_SCHEMA_VERSION (crates/tokmd/src/tool_schema.rs)
    ///   - CONTEXT_SCHEMA_VERSION (crates/tokmd-types/src/lib.rs)
    ///   - CONTEXT_BUNDLE_SCHEMA_VERSION (crates/tokmd-types/src/lib.rs)
    ///   - HANDOFF_SCHEMA_VERSION (crates/tokmd-types/src/lib.rs)
    #[arg(long, value_delimiter = ',')]
    pub schema: Option<Vec<String>>,
}

#[derive(Args, Debug, Clone, Default)]
pub struct BoundariesCheckArgs {}

#[derive(Args, Debug, Clone, Default)]
pub struct FixtureBlobsCheckArgs {}

#[derive(Args, Debug, Clone, Default)]
pub struct GateArgs {
    /// Run in check-only mode (no file modifications)
    #[arg(long)]
    pub check: bool,
}

#[derive(Args, Debug, Clone, Default)]
pub struct LintFixArgs {
    /// Verify lint without modifying files
    #[arg(long)]
    pub check: bool,

    /// Skip clippy --fix step
    #[arg(long)]
    pub no_clippy: bool,
}

#[derive(Args, Debug, Clone, Default)]
pub struct TrimTargetArgs {
    /// Report reclaimable target/debug space without deleting files
    #[arg(long)]
    pub check: bool,

    /// Keep PDB files
    #[arg(long)]
    pub keep_pdb: bool,

    /// Keep incremental compilation directories
    #[arg(long)]
    pub keep_incremental: bool,
}

#[derive(Args, Debug, Clone, Default)]
pub struct SccacheArgs {
    /// Verify that sccache is installed and print the repo-native entry points
    #[arg(long)]
    pub check: bool,

    /// Show sccache cache statistics
    #[arg(long)]
    pub stats: bool,

    /// Stop the local sccache server
    #[arg(long)]
    pub stop: bool,

    /// Preserve the caller's incremental setting instead of defaulting to 0
    #[arg(long)]
    pub keep_incremental: bool,

    /// Normalize paths under this base dir for cross-worktree cache reuse
    #[arg(long = "basedir", value_name = "PATH")]
    pub basedirs: Vec<std::path::PathBuf>,

    /// Cargo subcommand and arguments to run under sccache
    #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
    pub cargo_args: Vec<String>,
}

#[derive(Args, Debug, Clone, Default)]
pub struct CockpitArgs {
    /// Base reference to compare from (default: main)
    #[arg(long, default_value = "main")]
    pub base: String,

    /// Head reference to compare to (default: HEAD)
    #[arg(long, default_value = "HEAD")]
    pub head: String,

    /// PR number for GitHub comment posting
    #[arg(long)]
    pub pr_number: Option<u64>,

    /// Output format: json, md, sections
    #[arg(long, default_value = "json")]
    pub format: String,

    /// Post cockpit as PR comment via gh CLI
    #[arg(long)]
    pub post_comment: bool,
}
