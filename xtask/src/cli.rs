use clap::{Args, Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(name = "xtask")]
#[command(about = "Development tasks for tokmd", long_about = None)]
pub struct XtaskCli {
    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand, Debug, Clone)]
pub enum Commands {
    /// Publish all crates in dependency order
    Publish(PublishArgs),
}

#[derive(Args, Debug, Clone, Default)]
pub struct PublishArgs {
    /// Show publish plan without executing anything (no crates.io interaction)
    #[arg(long)]
    pub plan: bool,

    /// Run in dry-run mode (runs `cargo publish --dry-run` per crate, validates packaging)
    #[arg(long, short = 'n')]
    pub dry_run: bool,

    /// Run cargo publish --dry-run for each crate before actual publish (deprecated: use --dry-run)
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

    /// Skip confirmation prompt (required for non-dry-run without TTY)
    #[arg(long, short = 'y')]
    pub yes: bool,
}
