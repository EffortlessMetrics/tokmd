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
    #[command(alias = "docs-sync")]
    Docs(DocsArgs),
    /// Verify source-of-truth documentation artifact shape and links
    DocArtifacts(DocArtifactsArgs),
    /// Validate the Rust-native proof policy
    ProofPolicy(ProofPolicyArgs),
    /// Resolve proof observation collection thresholds from checked policy and overrides
    ProofObservationThresholds(ProofObservationThresholdsArgs),
    /// Resolve fast proof-run PR policy from checked policy
    ProofRunPrPolicy(ProofRunPrPolicyArgs),
    /// Resolve proof executor PR policy from checked policy and manual overrides
    ProofExecutorPrPolicy(ProofExecutorPrPolicyArgs),
    /// Extract GitHub run ids from a saved workflow run-list JSON artifact
    ProofObservationRunIds(ProofObservationRunIdsArgs),
    /// Discover proof scopes affected by a git diff
    Affected(AffectedArgs),
    /// Print proof command plans without executing them
    Proof(ProofArgs),
    /// Verify generated proof artifacts agree without executing planned commands
    ProofArtifactsCheck(ProofArtifactsCheckArgs),
    /// Verify opted-in executed proof artifacts agree and passed
    ProofExecutionArtifactsCheck(ProofArtifactsCheckArgs),
    /// Verify an opted-in required proof-run summary
    ProofRunArtifactsCheck(ProofRunArtifactsCheckArgs),
    /// Write a compact observation report for an opted-in required proof run
    ProofRunObservation(ProofRunObservationArgs),
    /// Summarize one or more required proof-run observation artifacts
    ProofRunObservationsSummary(ProofRunObservationsSummaryArgs),
    /// Write a compact observation report for opted-in executed proof artifacts
    ProofExecutionObservation(ProofExecutionObservationArgs),
    /// Summarize one or more proof executor observation artifacts
    ProofExecutionObservationsSummary(ProofExecutionObservationsSummaryArgs),
    /// Verify cockpit review-packet schemas and artifact hashes
    ReviewPacketCheck(ReviewPacketCheckArgs),
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
    /// Emit a durable receipt for coverage artifacts
    CoverageReceipt(CoverageReceiptArgs),
    /// Emit a durable receipt for CI job actuals
    CiActuals(CiActualsArgs),
    /// Verify the non-Rust file allowlist (file policy checker)
    CheckFilePolicy(FilePolicyArgs),
    /// Verify the AST-backed Clippy exception ledger
    CheckClippyExceptions(ClippyExceptionsArgs),
    /// Generate the LEM-aware advisory PR Plan
    CiPlan(CiPlanArgs),
    /// Generate Jules run and friction rollup indexes
    JulesIndex(JulesIndexArgs),
    /// Verify the workspace panic-family allowlist (semantic no-panic checker)
    CheckNoPanicFamily(NoPanicArgs),
    /// Propose new no-panic allowlist entries from current findings
    NoPanicPropose(NoPanicProposeArgs),
    /// Verify CI lane whitelist coverage and exception receipts
    CiLaneWhitelist(CiLaneWhitelistArgs),
    /// Auto-fix lint issues (fmt + clippy --fix) then verify
    LintFix(LintFixArgs),
    /// Run Cargo through an opt-in local sccache wrapper
    Sccache(SccacheArgs),
    /// Reclaim target/debug space by trimming Windows PDBs and incremental state
    TrimTarget(TrimTargetArgs),
    /// Emit a small phase-timing receipt for core inventory and optional analysis workflows
    PerfSmoke(PerfSmokeArgs),
    /// Compare heuristic and AST-backed Rust landmarks for explicit files
    AstShadowCompare(AstShadowCompareArgs),
    /// Verify AST shadow artifact shape and summary counts
    AstShadowCheck(AstShadowCheckArgs),
    /// Generate or check committed public Shields badge endpoints
    Badges(BadgesArgs),
    /// Generate or check PR-scoped RIPR evidence artifacts
    RiprPr(RiprPrArgs),
    /// Generate or check RIPR review guidance artifacts
    RiprReviewComments(RiprReviewCommentsArgs),
}

#[derive(Args, Debug, Clone)]
pub struct AstShadowCompareArgs {
    /// Repo-relative Rust source path to compare. Repeat for multiple files.
    #[arg(long = "path", required = true)]
    pub paths: Vec<std::path::PathBuf>,

    /// Output directory for heuristic.json, ast.json, and diff.json.
    #[arg(long, default_value = "target/tokmd-ast-shadow")]
    pub out: std::path::PathBuf,
}

#[derive(Args, Debug, Clone)]
pub struct AstShadowCheckArgs {
    /// Optional repo-relative Rust source path to compare before checking artifacts.
    #[arg(long = "path")]
    pub paths: Vec<std::path::PathBuf>,

    /// Directory containing heuristic.json, ast.json, and diff.json.
    #[arg(long, default_value = "target/tokmd-ast-shadow")]
    pub dir: std::path::PathBuf,

    /// Optional JSON receipt path for the verifier result.
    #[arg(long)]
    pub json: Option<std::path::PathBuf>,
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

#[derive(Args, Debug, Clone)]
pub struct DocArtifactsArgs {
    /// Verify source-of-truth documentation artifact shape and links
    #[arg(long)]
    pub check: bool,

    /// Documentation artifact policy file
    #[arg(long, default_value = "policy/doc-artifacts.toml")]
    pub policy: std::path::PathBuf,

    /// Optional JSON receipt path for the doc-artifacts check result
    #[arg(long)]
    pub json: Option<std::path::PathBuf>,
}

impl Default for DocArtifactsArgs {
    fn default() -> Self {
        Self {
            check: false,
            policy: std::path::PathBuf::from("policy/doc-artifacts.toml"),
            json: None,
        }
    }
}

#[derive(Args, Debug, Clone, Default)]
pub struct BadgesArgs {
    /// Check committed badge endpoints for drift without updating badges/
    #[arg(long)]
    pub check: bool,
}

#[derive(Args, Debug, Clone)]
pub struct RiprPrArgs {
    /// Verify required PR evidence files instead of regenerating them
    #[arg(long)]
    pub check: bool,

    /// Base git revision for diff-aware RIPR evidence
    #[arg(long, default_value = "origin/main")]
    pub base: String,
}

impl Default for RiprPrArgs {
    fn default() -> Self {
        Self {
            check: false,
            base: "origin/main".to_string(),
        }
    }
}

#[derive(Args, Debug, Clone)]
pub struct RiprReviewCommentsArgs {
    /// Verify required review guidance files instead of regenerating them
    #[arg(long)]
    pub check: bool,

    /// Pull-request base revision
    #[arg(long, default_value = "origin/main")]
    pub base: String,

    /// Pull-request head revision
    #[arg(long, default_value = "HEAD")]
    pub head: String,
}

impl Default for RiprReviewCommentsArgs {
    fn default() -> Self {
        Self {
            check: false,
            base: "origin/main".to_string(),
            head: "HEAD".to_string(),
        }
    }
}

#[derive(Args, Debug, Clone, Default)]
pub struct VersionConsistencyArgs {}

#[derive(Args, Debug, Clone, Default)]
pub struct JulesIndexArgs {
    /// Verify generated Jules indexes are up to date without writing files
    #[arg(long)]
    pub check: bool,
}

#[derive(Args, Debug, Clone, Default)]
pub struct LintPolicyArgs {}

#[derive(Args, Debug, Clone)]
pub struct CoverageReceiptArgs {
    /// JSON coverage report generated by cargo-llvm-cov
    #[arg(long, default_value = "coverage.json")]
    pub coverage_json: std::path::PathBuf,

    /// Text coverage summary generated by cargo-llvm-cov
    #[arg(long, default_value = "coverage.txt")]
    pub coverage_text: std::path::PathBuf,

    /// LCOV coverage report generated by cargo-llvm-cov
    #[arg(long, default_value = "lcov.info")]
    pub lcov: std::path::PathBuf,

    /// Output path for the coverage receipt
    #[arg(long, default_value = "target/coverage/coverage-receipt.json")]
    pub output: std::path::PathBuf,

    /// Repository identifier recorded in the receipt
    #[arg(long, default_value = "tokmd")]
    pub repo: String,

    /// Coverage lane name recorded in the receipt
    #[arg(long, default_value = "coverage")]
    pub lane: String,

    /// Coverage flag recorded in the receipt
    #[arg(long, default_value = "rust")]
    pub flag: String,

    /// Workflow name recorded in the receipt
    #[arg(long, default_value = "Coverage")]
    pub workflow: String,

    /// Commit SHA recorded in the receipt; defaults to GITHUB_SHA or HEAD
    #[arg(long)]
    pub sha: Option<String>,
}

impl Default for CoverageReceiptArgs {
    fn default() -> Self {
        Self {
            coverage_json: std::path::PathBuf::from("coverage.json"),
            coverage_text: std::path::PathBuf::from("coverage.txt"),
            lcov: std::path::PathBuf::from("lcov.info"),
            output: std::path::PathBuf::from("target/coverage/coverage-receipt.json"),
            repo: "tokmd".to_string(),
            lane: "coverage".to_string(),
            flag: "rust".to_string(),
            workflow: "Coverage".to_string(),
            sha: None,
        }
    }
}

#[derive(Args, Debug, Clone)]
pub struct CiActualsArgs {
    /// JSON file containing the GitHub Actions `needs` context
    #[arg(long, default_value = "target/ci/needs.json")]
    pub needs: std::path::PathBuf,

    /// Optional JSON sidecar with measured job durations
    #[arg(long, value_name = "PATH")]
    pub timings: Option<std::path::PathBuf>,

    /// Output path for the CI actuals receipt
    #[arg(long, default_value = "target/ci/ci-actuals.json")]
    pub output: std::path::PathBuf,

    /// Repository identifier recorded in the receipt
    #[arg(long, default_value = "tokmd")]
    pub repo: String,

    /// Workflow name recorded in the receipt
    #[arg(long, default_value = "CI")]
    pub workflow: String,

    /// Commit SHA recorded in the receipt; defaults to GITHUB_SHA or HEAD
    #[arg(long)]
    pub sha: Option<String>,
}

impl Default for CiActualsArgs {
    fn default() -> Self {
        Self {
            needs: std::path::PathBuf::from("target/ci/needs.json"),
            timings: None,
            output: std::path::PathBuf::from("target/ci/ci-actuals.json"),
            repo: "tokmd".to_string(),
            workflow: "CI".to_string(),
            sha: None,
        }
    }
}

#[derive(Args, Debug, Clone)]
pub struct PerfSmokeArgs {
    /// Repository or directory to measure. The emitted receipt records counts,
    /// not the raw path.
    #[arg(long, default_value = ".")]
    pub target_repo: std::path::PathBuf,

    /// Output path for the performance smoke receipt.
    #[arg(long, default_value = "target/perf/perf-smoke.json")]
    pub output: std::path::PathBuf,

    /// Repository identifier recorded in the receipt.
    #[arg(long, default_value = "tokmd")]
    pub repo: String,

    /// Commit SHA recorded in the receipt; defaults to GITHUB_SHA or HEAD.
    #[arg(long)]
    pub sha: Option<String>,

    /// Optional analysis preset to time. Repeat to measure multiple bounded presets.
    #[arg(long = "analysis-preset", value_delimiter = ',')]
    pub analysis_presets: Vec<String>,

    /// Maximum files walked for each timed analysis preset.
    #[arg(long, default_value_t = 500)]
    pub analysis_max_files: usize,

    /// Maximum total bytes read for each timed analysis preset.
    #[arg(long, default_value_t = 52_428_800)]
    pub analysis_max_bytes: u64,

    /// Maximum bytes read from each file for each timed analysis preset.
    #[arg(long, default_value_t = 1_048_576)]
    pub analysis_max_file_bytes: u64,

    /// Maximum git commits scanned for each timed analysis preset.
    #[arg(long, default_value_t = 200)]
    pub analysis_max_commits: usize,

    /// Maximum files scanned per git commit for each timed analysis preset.
    #[arg(long, default_value_t = 200)]
    pub analysis_max_commit_files: usize,
}

impl Default for PerfSmokeArgs {
    fn default() -> Self {
        Self {
            target_repo: std::path::PathBuf::from("."),
            output: std::path::PathBuf::from("target/perf/perf-smoke.json"),
            repo: "tokmd".to_string(),
            sha: None,
            analysis_presets: Vec::new(),
            analysis_max_files: 500,
            analysis_max_bytes: 52_428_800,
            analysis_max_file_bytes: 1_048_576,
            analysis_max_commits: 200,
            analysis_max_commit_files: 200,
        }
    }
}

#[derive(Args, Debug, Clone)]
pub struct FilePolicyArgs {
    /// Path to the non-Rust allowlist TOML
    #[arg(long, default_value = "policy/non-rust-allowlist.toml")]
    pub allowlist: std::path::PathBuf,

    /// Optional report output directory
    #[arg(long, value_name = "DIR")]
    pub report_dir: Option<std::path::PathBuf>,

    /// Treat findings as errors (default is advisory)
    #[arg(long)]
    pub strict: bool,
}

#[derive(Args, Debug, Clone)]
pub struct ClippyExceptionsArgs {
    /// Clippy exceptions TOML
    #[arg(long, default_value = "policy/clippy-exceptions.toml")]
    pub policy: std::path::PathBuf,

    /// Optional report output directory
    #[arg(long, value_name = "DIR")]
    pub report_dir: Option<std::path::PathBuf>,

    /// Treat findings as errors (default is advisory)
    #[arg(long)]
    pub strict: bool,
}

impl Default for FilePolicyArgs {
    fn default() -> Self {
        Self {
            allowlist: std::path::PathBuf::from("policy/non-rust-allowlist.toml"),
            report_dir: None,
            strict: false,
        }
    }
}

impl Default for ClippyExceptionsArgs {
    fn default() -> Self {
        Self {
            policy: std::path::PathBuf::from("policy/clippy-exceptions.toml"),
            report_dir: None,
            strict: false,
        }
    }
}

#[derive(Args, Debug, Clone)]
pub struct CiPlanArgs {
    /// Base git revision
    #[arg(long, default_value = "origin/main")]
    pub base: String,

    /// Head git revision
    #[arg(long, default_value = "HEAD")]
    pub head: String,

    /// JSON labels payload (object form, bare-array form, or CSV)
    #[arg(long, value_name = "JSON")]
    pub labels_json: Option<String>,

    /// Lane whitelist TOML
    #[arg(long, default_value = "policy/ci-lane-whitelist.toml")]
    pub lanes: std::path::PathBuf,

    /// Risk-pack TOML
    #[arg(long, default_value = "policy/ci-risk-packs.toml")]
    pub risk_packs: std::path::PathBuf,

    /// Output path for the JSON plan; if absent, prints to stdout
    #[arg(long, value_name = "PATH")]
    pub json_out: Option<std::path::PathBuf>,

    /// Optional path to GITHUB_STEP_SUMMARY (the workflow appends to it)
    #[arg(long, value_name = "PATH")]
    pub github_summary: Option<std::path::PathBuf>,

    /// Fail with a non-zero exit when the estimated LEM exceeds the hard
    /// ceiling and no override label is present
    #[arg(long)]
    pub enforce: bool,

    /// Optional directory of past `ci-actuals.json` files used to compute
    /// learned p50/p90/p95 estimates. When absent, static `base_lem` is used.
    #[arg(long, value_name = "DIR")]
    pub actuals_dir: Option<std::path::PathBuf>,
}

impl Default for CiPlanArgs {
    fn default() -> Self {
        Self {
            base: "origin/main".into(),
            head: "HEAD".into(),
            labels_json: None,
            lanes: std::path::PathBuf::from("policy/ci-lane-whitelist.toml"),
            risk_packs: std::path::PathBuf::from("policy/ci-risk-packs.toml"),
            json_out: None,
            github_summary: None,
            enforce: false,
            actuals_dir: None,
        }
    }
}

#[derive(Args, Debug, Clone)]
pub struct CiLaneWhitelistArgs {
    /// Workflows directory (relative to workspace root)
    #[arg(long, default_value = ".github/workflows")]
    pub workflows: std::path::PathBuf,

    /// Lane whitelist TOML
    #[arg(long, default_value = "policy/ci-lane-whitelist.toml")]
    pub whitelist: std::path::PathBuf,

    /// Lane whitelist exceptions TOML
    #[arg(long, default_value = "policy/ci-whitelist-exceptions.toml")]
    pub exceptions: std::path::PathBuf,

    /// Optional report output directory
    #[arg(long, value_name = "DIR")]
    pub report_dir: Option<std::path::PathBuf>,

    /// Treat findings as errors (default is advisory)
    #[arg(long)]
    pub strict: bool,
}

impl Default for CiLaneWhitelistArgs {
    fn default() -> Self {
        Self {
            workflows: std::path::PathBuf::from(".github/workflows"),
            whitelist: std::path::PathBuf::from("policy/ci-lane-whitelist.toml"),
            exceptions: std::path::PathBuf::from("policy/ci-whitelist-exceptions.toml"),
            report_dir: None,
            strict: false,
        }
    }
}

#[derive(Args, Debug, Clone, Default)]
pub struct NoPanicArgs {
    /// Emit a machine-readable JSON report instead of human output
    #[arg(long)]
    pub json: bool,

    /// Write the machine-readable report to a JSON artifact
    #[arg(long, value_name = "PATH")]
    pub json_output: Option<std::path::PathBuf>,

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

    /// Write the machine-readable validation report to a JSON artifact
    #[arg(long, value_name = "PATH")]
    pub json_output: Option<std::path::PathBuf>,

    /// Policy file to validate
    #[arg(long, default_value = "ci/proof.toml")]
    pub policy: std::path::PathBuf,
}

#[derive(Args, Debug, Clone)]
pub struct ProofObservationThresholdsArgs {
    /// Machine-readable proof-policy report to read
    #[arg(
        long,
        value_name = "PATH",
        default_value = "target/proof-observations/proof-policy.json"
    )]
    pub proof_policy_json: std::path::PathBuf,

    /// Write resolved thresholds as a GitHub Actions env file
    #[arg(
        long,
        value_name = "PATH",
        default_value = "target/proof-observations/thresholds.env"
    )]
    pub env_output: std::path::PathBuf,

    /// Workflow-dispatch override for the successful proof-executor run window
    #[arg(long, default_value = "")]
    pub run_limit: String,

    /// Workflow-dispatch override for the minimum observation artifact count
    #[arg(long, default_value = "")]
    pub min_observations: String,

    /// Workflow-dispatch override for the minimum executed advisory command count
    #[arg(long, default_value = "")]
    pub min_executed: String,

    /// Workflow-dispatch override for the minimum distinct proof scope count
    #[arg(long, default_value = "")]
    pub min_scopes: String,

    /// Workflow-dispatch override for the minimum produced evidence artifact count
    #[arg(long, default_value = "")]
    pub min_artifacts: String,

    /// Workflow-dispatch override for the minimum recent passing collector run count
    #[arg(long, default_value = "")]
    pub min_passing_collector_runs: String,
}

#[derive(Args, Debug, Clone)]
pub struct ProofRunPrPolicyArgs {
    /// Machine-readable proof-policy report to read
    #[arg(
        long,
        value_name = "PATH",
        default_value = "target/proof-run/proof-policy.json"
    )]
    pub proof_policy_json: std::path::PathBuf,

    /// Write resolved PR proof-run policy as a GitHub Actions output file
    #[arg(
        long,
        value_name = "PATH",
        default_value = "target/proof-run/proof-run-pr.outputs"
    )]
    pub github_output: std::path::PathBuf,
}

#[derive(Args, Debug, Clone)]
pub struct ProofExecutorPrPolicyArgs {
    /// Machine-readable proof-policy report to read
    #[arg(
        long,
        value_name = "PATH",
        default_value = "target/proof/proof-policy.json"
    )]
    pub proof_policy_json: std::path::PathBuf,

    /// Write resolved PR executor policy as a GitHub Actions env file
    #[arg(
        long,
        value_name = "PATH",
        default_value = "target/proof/proof-executor-pr.env"
    )]
    pub env_output: std::path::PathBuf,

    /// Workflow-dispatch override for maximum advisory executor commands
    #[arg(long, default_value = "")]
    pub max_commands: String,
}

#[derive(Args, Debug, Clone)]
pub struct ProofObservationRunIdsArgs {
    /// GitHub Actions run-list JSON to read
    #[arg(
        long,
        value_name = "PATH",
        default_value = "target/proof-observations/runs.json"
    )]
    pub runs_json: std::path::PathBuf,

    /// Write one run id per line for artifact download loops
    #[arg(
        long,
        value_name = "PATH",
        default_value = "target/proof-observations/run-ids.txt"
    )]
    pub output: std::path::PathBuf,
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

    /// Write the machine-readable affected-scope report to a JSON artifact
    #[arg(long, value_name = "PATH")]
    pub json_output: Option<std::path::PathBuf>,

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

    /// Execute required proof-plan commands and write a proof-run summary
    #[arg(long)]
    pub run_required: bool,

    /// Explicitly opt a CI invocation into required proof command execution
    #[arg(long)]
    pub allow_ci_required_execution: bool,

    /// Explicitly opt a local invocation into required proof command execution
    #[arg(long)]
    pub allow_local_required_execution: bool,

    /// Write the required proof-run execution summary
    #[arg(
        long,
        value_name = "PATH",
        default_value = "target/proof/proof-run-summary.json"
    )]
    pub proof_run_summary: std::path::PathBuf,

    /// Write a Markdown summary for the generated proof plan
    #[arg(long, value_name = "PATH")]
    pub summary_md: Option<std::path::PathBuf>,

    /// Write the generated proof plan JSON report to this path
    #[arg(long, value_name = "PATH")]
    pub plan_json: Option<std::path::PathBuf>,

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

    /// Override the policy-selected maximum number of advisory executor commands.
    #[arg(long)]
    pub executor_max_commands: Option<usize>,

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
pub struct ProofRunArtifactsCheckArgs {
    /// Required proof-run summary artifact to verify
    #[arg(
        long,
        value_name = "PATH",
        default_value = "target/proof/proof-run-summary.json"
    )]
    pub proof_run_summary: std::path::PathBuf,
}

#[derive(Args, Debug, Clone)]
pub struct ProofRunObservationArgs {
    /// Required proof-run summary artifact to observe
    #[arg(
        long,
        value_name = "PATH",
        default_value = "target/proof-run/proof-run-summary.json"
    )]
    pub proof_run_summary: std::path::PathBuf,

    /// Output path for the compact proof-run observation report
    #[arg(
        long,
        value_name = "PATH",
        default_value = "target/proof-run/proof-run-observation.json"
    )]
    pub output: std::path::PathBuf,
}

#[derive(Args, Debug, Clone)]
pub struct ProofRunObservationsSummaryArgs {
    /// Proof-run observation artifact to include. Repeat for multiple runs.
    #[arg(long = "observation", value_name = "PATH")]
    pub observations: Vec<std::path::PathBuf>,

    /// Directory tree to scan for proof-run-observation.json artifacts.
    #[arg(long = "observations-dir", value_name = "DIR")]
    pub observation_dirs: Vec<std::path::PathBuf>,

    /// JSON list of successful source workflow runs used as the observation window.
    #[arg(long, value_name = "PATH")]
    pub source_runs_json: Option<std::path::PathBuf>,

    /// Output path for the collection summary. Prints JSON to stdout when omitted.
    #[arg(long, value_name = "PATH")]
    pub output: Option<std::path::PathBuf>,

    /// Output path for a human-readable Markdown collection summary.
    #[arg(long, value_name = "PATH")]
    pub summary_md: Option<std::path::PathBuf>,
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

    /// Minimum number of recent passing manual collector runs required.
    #[arg(long, default_value_t = 0)]
    pub min_passing_collector_runs: usize,

    /// Output path for the collection summary. Prints JSON to stdout when omitted.
    #[arg(long, value_name = "PATH")]
    pub output: Option<std::path::PathBuf>,

    /// Output path for a human-readable Markdown collection summary.
    #[arg(long, value_name = "PATH")]
    pub summary_md: Option<std::path::PathBuf>,

    /// GitHub Actions run-list JSON for successful manual collector runs.
    #[arg(long, value_name = "PATH")]
    pub collector_runs_json: Option<std::path::PathBuf>,

    /// GitHub Actions run-list JSON for the successful proof-executor window.
    #[arg(long, value_name = "PATH")]
    pub source_runs_json: Option<std::path::PathBuf>,

    /// Output path for a promotion-readiness receipt.
    #[arg(long, value_name = "PATH")]
    pub promotion_readiness: Option<std::path::PathBuf>,
}

#[derive(Args, Debug, Clone)]
pub struct ReviewPacketCheckArgs {
    /// Cockpit review packet directory, usually .tokmd/review
    #[arg(long)]
    pub dir: std::path::PathBuf,

    /// Write a machine-readable verification receipt to this path.
    #[arg(long, value_name = "PATH")]
    pub json: Option<std::path::PathBuf>,
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
    ///   - CONTEXT_SCHEMA_VERSION (crates/tokmd-types/src/context.rs)
    ///   - CONTEXT_BUNDLE_SCHEMA_VERSION (crates/tokmd-types/src/context.rs)
    ///   - HANDOFF_SCHEMA_VERSION (crates/tokmd-types/src/context.rs)
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
