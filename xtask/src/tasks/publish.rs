//! Publish crates to crates.io in dependency order.
//!
//! Safety guarantees:
//! - Only publishes workspace members (not external dependencies)
//! - Filters out non-publishable crates (publish = false)
//! - Validates exclusions don't break required dependencies
//! - Requires confirmation for actual publishing (unless --yes or CI)

use std::collections::{BTreeMap, BTreeSet, HashSet};
use std::io::{self, IsTerminal, Write};
use std::path::Path;
use std::process::Command;
use std::thread::sleep;
use std::time::Duration;

use anyhow::{Context, Result, anyhow, bail};
use cargo_metadata::{DependencyKind, Metadata, MetadataCommand, Package, PackageId};
use chrono::{DateTime, FixedOffset, Utc};
use petgraph::algo::toposort;
use petgraph::graph::DiGraph;

use crate::cli::PublishArgs;

/// Result of attempting to publish a single crate.
#[derive(Debug)]
pub enum PublishResult {
    Success,
    AlreadyPublished,
    Failed(anyhow::Error),
}

/// Information about why a crate was included in the publish set.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InclusionReason {
    /// Explicitly requested via --crates
    Explicit,
    /// Included as a transitive dependency of an explicit crate
    TransitiveDep(String),
    /// Included because no --crates filter was specified (all publishable)
    Default,
}

/// Information about why a crate was excluded from the publish set.
#[derive(Debug, Clone)]
pub enum ExclusionReason {
    /// Crate has publish = false in Cargo.toml
    NotPublishable,
    /// Crate is xtask (internal tooling)
    IsXtask,
    /// Crate is fuzz (testing infrastructure)
    IsFuzz,
    /// Explicitly excluded via --exclude
    ExplicitExclude,
    /// Not in the requested --crates set or their dependencies
    NotRequested,
}

/// The resolved publish plan.
#[derive(Debug)]
pub struct PublishPlan {
    /// Crates to publish, in topological order.
    pub publish_order: Vec<String>,
    /// Why each crate was included.
    pub inclusion_reasons: BTreeMap<String, InclusionReason>,
    /// Why each crate was excluded.
    pub exclusion_reasons: BTreeMap<String, ExclusionReason>,
    /// The workspace version (from [workspace.package].version).
    pub workspace_version: String,
}

/// Publish all workspace crates in dependency order.
pub fn run(args: PublishArgs) -> Result<()> {
    // Load workspace metadata
    // Use no_deps() for faster metadata loading - we only need workspace members
    // and their manifest-declared dependencies, not the full resolved graph
    let metadata = MetadataCommand::new()
        .no_deps()
        .exec()
        .context("Failed to load cargo metadata")?;

    // Resolve the publish plan (workspace-scoped, validated)
    let plan = resolve_publish_plan(&metadata, &args)?;

    // Handle --plan mode: just print and exit
    if args.plan {
        print_plan(&plan, &args);
        return Ok(());
    }

    // Run pre-publish checks (unless skipped)
    if !args.skip_checks {
        run_pre_publish_checks(&args, &plan.workspace_version)?;
    }

    // Handle --from flag
    let start_idx = if let Some(ref from_crate) = args.from {
        plan.publish_order
            .iter()
            .position(|name| name == from_crate)
            .ok_or_else(|| anyhow!("Crate '{}' not found in publish order", from_crate))?
    } else {
        0
    };

    let crates_to_publish = &plan.publish_order[start_idx..];

    // Print summary and require confirmation for real publishing
    print_pre_publish_summary(&plan, &args, start_idx);

    if !args.dry_run && !args.yes && !confirm_publish()? {
        println!("\nPublish cancelled.");
        return Ok(());
    }

    // Execute publishing
    let (succeeded, failed) = execute_publish(crates_to_publish, &args)?;

    // Print summary
    println!("\n--- Summary ---");
    println!("Succeeded: {}", succeeded.len());
    if !failed.is_empty() {
        println!("Failed: {} ({:?})", failed.len(), failed);
    }

    // Create git tag if requested
    if args.tag && failed.is_empty() && !args.dry_run {
        create_git_tag(&args, &plan.workspace_version)?;
    }

    if !failed.is_empty() {
        bail!("{} crate(s) failed to publish", failed.len());
    }

    Ok(())
}

/// Resolve the publish plan from workspace metadata.
///
/// This is the critical safety function that ensures we only consider
/// workspace members, not external dependencies.
fn resolve_publish_plan(metadata: &Metadata, args: &PublishArgs) -> Result<PublishPlan> {
    // Step 1: Get workspace members only (SAFETY: this is the critical filter)
    let workspace_member_ids: HashSet<&PackageId> = metadata.workspace_members.iter().collect();
    let workspace_root = metadata.workspace_root.as_std_path();

    // Step 2: Build the set of publishable workspace packages
    let workspace_packages: Vec<&Package> = metadata
        .packages
        .iter()
        .filter(|pkg| workspace_member_ids.contains(&pkg.id))
        .collect();

    // Step 3: Determine which crates are publishable
    let mut publishable: BTreeSet<String> = BTreeSet::new();
    let mut exclusion_reasons: BTreeMap<String, ExclusionReason> = BTreeMap::new();

    for pkg in &workspace_packages {
        let name = pkg.name.as_str();

        // Check publish = false
        if pkg.publish.as_ref().is_some_and(|p| p.is_empty()) {
            exclusion_reasons.insert(name.to_string(), ExclusionReason::NotPublishable);
            continue;
        }

        // Skip xtask and fuzz by convention
        if name == "xtask" {
            exclusion_reasons.insert(name.to_string(), ExclusionReason::IsXtask);
            continue;
        }
        if name == "tokmd-fuzz" || name == "fuzz" {
            exclusion_reasons.insert(name.to_string(), ExclusionReason::IsFuzz);
            continue;
        }

        // Belt-and-suspenders: verify manifest is under workspace root
        let manifest_path = pkg.manifest_path.as_std_path();
        if !manifest_path.starts_with(workspace_root) {
            exclusion_reasons.insert(name.to_string(), ExclusionReason::NotPublishable);
            continue;
        }

        publishable.insert(name.to_string());
    }

    // Step 4: Validate publishable crates don't depend on non-publishable workspace crates
    validate_no_unpublishable_deps(&workspace_packages, &publishable, &exclusion_reasons)?;

    // Step 5: Compute topological order for publishable crates
    let publish_order = compute_publish_order(&workspace_packages, &publishable)?;

    // Step 6: Apply --crates filter (with transitive dependencies)
    let mut inclusion_reasons: BTreeMap<String, InclusionReason> = BTreeMap::new();
    let to_publish: BTreeSet<String> = if let Some(ref crates) = args.crates {
        let requested: HashSet<_> = crates.iter().cloned().collect();

        // Validate requested crates exist and are publishable
        for name in &requested {
            if !publishable.contains(name) {
                if let Some(reason) = exclusion_reasons.get(name) {
                    bail!("Crate '{}' cannot be published: {:?}", name, reason);
                }
                bail!(
                    "Crate '{}' is not a workspace member or does not exist",
                    name
                );
            }
        }

        let mut result = BTreeSet::new();

        // Add requested crates
        for name in &requested {
            result.insert(name.clone());
            inclusion_reasons.insert(name.clone(), InclusionReason::Explicit);
        }

        // Add transitive workspace dependencies
        for name in requested.iter() {
            add_transitive_deps(
                name,
                &workspace_packages,
                &publishable,
                &mut result,
                &mut inclusion_reasons,
            );
        }

        // Mark crates not in result as NotRequested
        for name in &publishable {
            if !result.contains(name) {
                exclusion_reasons.insert(name.clone(), ExclusionReason::NotRequested);
            }
        }

        result
    } else {
        // No filter: publish all publishable crates
        for name in &publishable {
            inclusion_reasons.insert(name.clone(), InclusionReason::Default);
        }
        publishable
    };

    // Step 7: Apply --exclude filter with validation
    let final_set: BTreeSet<String> = if let Some(ref excludes) = args.exclude {
        let exclude_set: HashSet<_> = excludes.iter().collect();

        // Validate exclusions don't break required dependencies
        for name in &to_publish {
            if exclude_set.contains(name) {
                continue;
            }
            let pkg = workspace_packages.iter().find(|p| p.name == *name).unwrap();
            for dep in &pkg.dependencies {
                if !is_publish_dependency(&dep.kind) {
                    continue;
                }
                if exclude_set.contains(&dep.name) && to_publish.contains(&dep.name) {
                    bail!(
                        "Cannot exclude '{}': crate '{}' depends on it",
                        dep.name,
                        name
                    );
                }
            }
        }

        // Apply exclusions
        let mut result = to_publish.clone();
        for name in excludes {
            if result.remove(name) {
                inclusion_reasons.remove(name);
                exclusion_reasons.insert(name.clone(), ExclusionReason::ExplicitExclude);
            }
        }
        result
    } else {
        to_publish
    };

    // Step 8: Filter publish_order to final set
    let filtered_order: Vec<_> = publish_order
        .into_iter()
        .filter(|name| final_set.contains(name))
        .collect();

    // Get workspace version
    let workspace_version = metadata
        .packages
        .iter()
        .find(|p| p.name == "tokmd")
        .map(|p| p.version.to_string())
        .unwrap_or_else(|| "unknown".to_string());

    Ok(PublishPlan {
        publish_order: filtered_order,
        inclusion_reasons,
        exclusion_reasons,
        workspace_version,
    })
}

/// Print the publish plan (for --plan mode).
fn print_plan(plan: &PublishPlan, args: &PublishArgs) {
    println!("=== Publish Plan ===\n");
    println!("Workspace version: {}\n", plan.workspace_version);

    println!("Publish order ({} crates):", plan.publish_order.len());
    for (i, name) in plan.publish_order.iter().enumerate() {
        let reason = plan
            .inclusion_reasons
            .get(name)
            .map(|r| match r {
                InclusionReason::Explicit => " (explicit)".to_string(),
                InclusionReason::TransitiveDep(parent) => format!(" (dep of {})", parent),
                InclusionReason::Default => String::new(),
            })
            .unwrap_or_default();
        println!("  {:2}. {}{}", i + 1, name, reason);
    }

    if !plan.exclusion_reasons.is_empty() && args.verbose {
        println!("\nExcluded crates:");
        for (name, reason) in &plan.exclusion_reasons {
            println!("  - {}: {:?}", name, reason);
        }
    }

    println!("\nFlags:");
    println!("  --dry-run: {}", args.dry_run);
    println!("  --tag: {}", args.tag);
    if args.tag {
        println!(
            "  --tag-format: {} (would create: {})",
            args.tag_format,
            args.tag_format
                .replace("{version}", &plan.workspace_version)
        );
    }
    if let Some(ref from) = args.from {
        println!("  --from: {}", from);
    }

    // Reconstruct the execution command from the current args (minus --plan, plus --yes)
    let exec_cmd = reconstruct_publish_command(args);
    println!("\nTo execute this plan:");
    println!("  {}", exec_cmd);
}

/// Reconstruct the publish command from args, removing --plan and ensuring --yes is present.
fn reconstruct_publish_command(args: &PublishArgs) -> String {
    let mut parts = vec!["cargo xtask publish".to_string()];

    // Scope filters (critical for matching the plan)
    if let Some(ref crates) = args.crates {
        parts.push(format!("--crates {}", crates.join(",")));
    }
    if let Some(ref exclude) = args.exclude {
        parts.push(format!("--exclude {}", exclude.join(",")));
    }
    if let Some(ref from) = args.from {
        parts.push(format!("--from {}", from));
    }

    // Mode flags
    if args.dry_run {
        parts.push("--dry-run".to_string());
    }
    if args.tag {
        parts.push("--tag".to_string());
        if args.tag_format != "v{version}" {
            parts.push(format!("--tag-format \"{}\"", args.tag_format));
        }
    }

    // Skip flags (preserve if user specified them)
    if args.skip_checks {
        parts.push("--skip-checks".to_string());
    }
    if args.skip_tests {
        parts.push("--skip-tests".to_string());
    }
    if args.skip_git_check {
        parts.push("--skip-git-check".to_string());
    }
    if args.skip_changelog_check {
        parts.push("--skip-changelog-check".to_string());
    }
    if args.skip_version_check {
        parts.push("--skip-version-check".to_string());
    }

    // Timing flags (only if non-default)
    if args.interval != 10 {
        parts.push(format!("--interval {}", args.interval));
    }
    if args.retry_delay != 30 {
        parts.push(format!("--retry-delay {}", args.retry_delay));
    }
    if args.rate_limit_timeout != 7200 {
        parts.push(format!("--rate-limit-timeout {}", args.rate_limit_timeout));
    }

    // Always add --yes for non-dry-run (the whole point of this reconstruction)
    if !args.dry_run {
        parts.push("--yes".to_string());
    }

    if args.verbose {
        parts.push("--verbose".to_string());
    }

    parts.join(" ")
}

/// Print pre-publish summary before execution.
fn print_pre_publish_summary(plan: &PublishPlan, args: &PublishArgs, start_idx: usize) {
    let crates_to_publish = &plan.publish_order[start_idx..];
    let mode = if args.dry_run { "[DRY RUN] " } else { "" };

    println!("\n{}Publishing {} crate(s):", mode, crates_to_publish.len());
    for name in crates_to_publish {
        println!("  - {}", name);
    }
    println!();
}

/// Ask for confirmation before publishing.
fn confirm_publish() -> Result<bool> {
    // Check for CI environment
    if std::env::var("CI").is_ok() {
        println!("CI environment detected, skipping confirmation.");
        return Ok(true);
    }

    // Refuse to prompt if stdin is not a TTY (prevents hangs in scripts/pipes)
    if !io::stdin().is_terminal() {
        bail!("stdin is not a terminal. Use --yes to skip confirmation in non-interactive mode.");
    }

    print!("Proceed with publishing? [y/N] ");
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;

    Ok(input.trim().eq_ignore_ascii_case("y") || input.trim().eq_ignore_ascii_case("yes"))
}

/// Execute the publish for a list of crates.
fn execute_publish(crates: &[String], args: &PublishArgs) -> Result<(Vec<String>, Vec<String>)> {
    let mut succeeded = Vec::new();
    let mut failed = Vec::new();

    for (idx, crate_name) in crates.iter().enumerate() {
        let position = format!("[{}/{}]", idx + 1, crates.len());
        println!("{} Publishing {}...", position, crate_name);

        let result = publish_crate_with_retry(crate_name, args)?;

        match result {
            PublishResult::Success => {
                println!("  ✓ Published {}", crate_name);
                succeeded.push(crate_name.clone());

                // Wait for crates.io propagation (unless last or dry-run)
                if idx < crates.len() - 1 && !args.dry_run {
                    println!("  Waiting {}s for crates.io propagation...", args.interval);
                    sleep(Duration::from_secs(args.interval));
                }
            }
            PublishResult::AlreadyPublished => {
                println!("  ✓ {} already published", crate_name);
                succeeded.push(crate_name.clone());
            }
            PublishResult::Failed(e) => {
                println!("  ✗ Failed to publish {}: {}", crate_name, e);
                failed.push(crate_name.clone());

                if !args.continue_on_error {
                    bail!(
                        "Publishing failed. Resume with: cargo xtask publish --from {}",
                        crate_name
                    );
                }
            }
        }
    }

    Ok((succeeded, failed))
}

/// Check if a dependency kind should be considered for publish ordering.
fn is_publish_dependency(kind: &DependencyKind) -> bool {
    matches!(
        kind,
        DependencyKind::Normal
            | DependencyKind::Build
            | DependencyKind::Development
            | DependencyKind::Unknown
    )
}

/// Validate that publishable crates don't depend on non-publishable workspace crates.
///
/// This catches the "silent broken publish" case where:
/// - Crate A is publishable
/// - Crate A depends on workspace crate B
/// - Crate B has publish = false (or is otherwise excluded)
fn validate_no_unpublishable_deps(
    packages: &[&Package],
    publishable: &BTreeSet<String>,
    exclusion_reasons: &BTreeMap<String, ExclusionReason>,
) -> Result<()> {
    let workspace_names: HashSet<_> = packages.iter().map(|p| p.name.as_str()).collect();
    let mut errors = Vec::new();

    for pkg in packages {
        if !publishable.contains(pkg.name.as_str()) {
            continue;
        }

        for dep in &pkg.dependencies {
            if !is_publish_dependency(&dep.kind) {
                continue;
            }

            // Only check workspace dependencies
            if !workspace_names.contains(dep.name.as_str()) {
                continue;
            }

            // If the dependency is a workspace crate but not publishable, that's an error
            if !publishable.contains(&dep.name) {
                let reason = exclusion_reasons
                    .get(&dep.name)
                    .map(|r| format!("{:?}", r))
                    .unwrap_or_else(|| "unknown".to_string());
                errors.push(format!(
                    "'{}' depends on non-publishable workspace crate '{}' ({})",
                    pkg.name, dep.name, reason
                ));
            }
        }
    }

    if !errors.is_empty() {
        bail!(
            "Cannot publish: workspace dependency violation(s):\n  - {}",
            errors.join("\n  - ")
        );
    }

    Ok(())
}

/// Compute topological publish order from workspace dependencies.
fn compute_publish_order(
    packages: &[&Package],
    publishable: &BTreeSet<String>,
) -> Result<Vec<String>> {
    let mut graph = DiGraph::<&str, ()>::new();
    let mut indices = BTreeMap::new();

    // Add publishable crates as nodes
    for pkg in packages {
        if publishable.contains(pkg.name.as_str()) {
            let idx = graph.add_node(pkg.name.as_str());
            indices.insert(pkg.name.as_str(), idx);
        }
    }

    // Add edges: dependency -> dependent (dependency must be published first)
    for pkg in packages {
        if !publishable.contains(pkg.name.as_str()) {
            continue;
        }
        let from_idx = indices[pkg.name.as_str()];

        for dep in &pkg.dependencies {
            if !is_publish_dependency(&dep.kind) {
                continue;
            }

            // Only add edges for publishable workspace crates
            if let Some(&to_idx) = indices.get(dep.name.as_str()) {
                graph.add_edge(to_idx, from_idx, ());
            }
        }
    }

    // Topological sort
    let sorted = toposort(&graph, None).map_err(|cycle| {
        let node = graph[cycle.node_id()];
        anyhow!("Dependency cycle detected involving: {}", node)
    })?;

    Ok(sorted
        .into_iter()
        .map(|idx| graph[idx].to_string())
        .collect())
}

/// Add transitive workspace dependencies to the set.
fn add_transitive_deps(
    crate_name: &str,
    packages: &[&Package],
    publishable: &BTreeSet<String>,
    result: &mut BTreeSet<String>,
    inclusion_reasons: &mut BTreeMap<String, InclusionReason>,
) {
    if let Some(pkg) = packages.iter().find(|p| p.name == crate_name) {
        for dep in &pkg.dependencies {
            if !is_publish_dependency(&dep.kind) {
                continue;
            }

            if publishable.contains(&dep.name) && !result.contains(&dep.name) {
                result.insert(dep.name.clone());
                // Only set reason if not already set (preserve explicit over transitive)
                inclusion_reasons
                    .entry(dep.name.clone())
                    .or_insert_with(|| InclusionReason::TransitiveDep(crate_name.to_string()));
                add_transitive_deps(&dep.name, packages, publishable, result, inclusion_reasons);
            }
        }
    }
}

/// Classify publish errors for retry logic.
#[derive(Debug)]
enum PublishErrorKind {
    /// Dependency not yet visible on crates.io - retryable
    PropagationDelay,
    /// Crate version already exists - treat as success
    AlreadyPublished,
    /// Authentication error - fail fast
    AuthError,
    /// Invalid manifest or missing files - fail fast
    ManifestError,
    /// Network error - potentially retryable
    NetworkError,
    /// Rate limited by crates.io (HTTP 429) - retryable after cooldown
    RateLimited,
    /// Unknown error - fail
    Unknown,
}

/// Classify the stderr output from cargo publish.
fn classify_publish_error(stderr: &str) -> PublishErrorKind {
    let lower = stderr.to_lowercase();

    // Already published - not an error
    if lower.contains("is already uploaded")
        || (lower.contains("crate version") && lower.contains("already exists"))
    {
        return PublishErrorKind::AlreadyPublished;
    }

    // Rate limit (429) - retryable after cooldown
    if lower.contains("status 429")
        || lower.contains("429 too many requests")
        || lower.contains("too many requests")
        || lower.contains("rate limit")
    {
        return PublishErrorKind::RateLimited;
    }

    // Auth errors - fail fast, no retry
    if lower.contains("token") && (lower.contains("invalid") || lower.contains("expired"))
        || lower.contains("not logged in")
        || lower.contains("authentication")
        || lower.contains("unauthorized")
        || lower.contains("403")
    {
        return PublishErrorKind::AuthError;
    }

    // Manifest/packaging errors - fail fast, no retry
    if lower.contains("invalid manifest")
        || lower.contains("missing") && lower.contains("field")
        || lower.contains("could not find")
        || lower.contains("failed to package")
        || lower.contains("license")
        || lower.contains("readme")
    {
        return PublishErrorKind::ManifestError;
    }

    // Propagation errors - retryable
    if lower.contains("failed to select a version for the requirement")
        || lower.contains("no matching package named")
        || lower.contains("failed to get")
        || lower.contains("no matching version")
        || (lower.contains("dependency") && lower.contains("not found"))
    {
        return PublishErrorKind::PropagationDelay;
    }

    // Network errors - potentially retryable
    if lower.contains("network")
        || lower.contains("connection")
        || lower.contains("timeout")
        || lower.contains("timed out")
    {
        return PublishErrorKind::NetworkError;
    }

    PublishErrorKind::Unknown
}

/// Parse the rate limit retry-after timestamp from crates.io error output.
///
/// Looks for "try again after <RFC2822 timestamp>" in the stderr text.
/// Returns the parsed timestamp, or `None` if not found/parseable.
fn parse_rate_limit_timestamp(stderr: &str) -> Option<DateTime<FixedOffset>> {
    // Look for "try again after " (case-insensitive) followed by an RFC2822 timestamp.
    // Example: "Please try again after Tue, 24 Feb 2026 16:57:08 GMT"
    let lower = stderr.to_lowercase();
    let marker = "try again after ";
    let pos = lower.find(marker)?;
    let after = &stderr[pos + marker.len()..];

    // The timestamp ends at " or ", a quote, or a newline
    let end = after
        .find(" or ")
        .or_else(|| after.find(|c: char| c == '"' || c == '\n' || c == '\r'))
        .unwrap_or(after.len());
    let timestamp_str = after[..end].trim();

    DateTime::parse_from_rfc2822(timestamp_str).ok()
}

/// Publish a single crate with retry logic for propagation delays.
fn publish_crate_with_retry(crate_name: &str, args: &PublishArgs) -> Result<PublishResult> {
    const MAX_RETRIES: u32 = 5;
    const MAX_RATE_LIMIT_WAITS: u32 = 6;
    const RATE_LIMIT_FALLBACK_SECS: u64 = 300;

    let retry_delay = Duration::from_secs(args.retry_delay);
    let rate_limit_timeout = Duration::from_secs(args.rate_limit_timeout);

    // Dry-run mode: validate packaging locally.
    //
    // We use `cargo package --list` instead of `cargo publish --dry-run`
    // because lockstep workspace releases reference versions that may not yet
    // exist on crates.io during preparation.
    if args.dry_run {
        println!("  [DRY RUN] Validating {}...", crate_name);
        let output = Command::new("cargo")
            .args(["package", "-p", crate_name, "--list", "--locked"])
            .output()
            .context("Failed to spawn cargo package")?;

        if output.status.success() {
            return Ok(PublishResult::Success);
        }

        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        let details = if stderr.trim().is_empty() {
            stdout.to_string()
        } else {
            stderr.to_string()
        };

        return Ok(PublishResult::Failed(anyhow!(
            "Dry-run packaging validation failed:\n{}",
            details
        )));
    }

    // Actual publish with retries.
    // Rate limit waits are tracked separately and don't count against MAX_RETRIES.
    let mut attempt: u32 = 0;
    let mut rate_limit_waits: u32 = 0;
    let mut total_rate_limit_wait = Duration::ZERO;

    loop {
        attempt += 1;

        let output = Command::new("cargo")
            .args(["publish", "-p", crate_name, "--locked"])
            .output()
            .context("Failed to spawn cargo publish")?;

        if output.status.success() {
            return Ok(PublishResult::Success);
        }

        let stderr = String::from_utf8_lossy(&output.stderr);
        let error_kind = classify_publish_error(&stderr);

        match error_kind {
            PublishErrorKind::AlreadyPublished => {
                return Ok(PublishResult::AlreadyPublished);
            }
            PublishErrorKind::AuthError => {
                return Ok(PublishResult::Failed(anyhow!(
                    "Authentication error (check `cargo login`):\n{}",
                    stderr
                )));
            }
            PublishErrorKind::ManifestError => {
                return Ok(PublishResult::Failed(anyhow!(
                    "Manifest or packaging error:\n{}",
                    stderr
                )));
            }
            PublishErrorKind::RateLimited => {
                rate_limit_waits += 1;
                if rate_limit_waits > MAX_RATE_LIMIT_WAITS {
                    return Ok(PublishResult::Failed(anyhow!(
                        "Rate limited {} times for {}, giving up:\n{}",
                        rate_limit_waits,
                        crate_name,
                        stderr
                    )));
                }

                // Parse the retry-after timestamp, fall back to default
                let wait_secs = if let Some(retry_after) = parse_rate_limit_timestamp(&stderr) {
                    let now = Utc::now();
                    let until = retry_after.with_timezone(&Utc);
                    let delta = until.signed_duration_since(now);
                    if delta.num_seconds() > 0 {
                        println!(
                            "  Rate limited by crates.io. Waiting until {} ({}s)...",
                            retry_after,
                            delta.num_seconds()
                        );
                        delta.num_seconds() as u64
                    } else {
                        // Timestamp is in the past, wait a small amount
                        println!(
                            "  Rate limited by crates.io (retry-after already passed). Waiting 10s..."
                        );
                        10
                    }
                } else {
                    println!(
                        "  Rate limited by crates.io (could not parse retry-after). Waiting {}s...",
                        RATE_LIMIT_FALLBACK_SECS
                    );
                    RATE_LIMIT_FALLBACK_SECS
                };

                let wait_duration = Duration::from_secs(wait_secs);

                // Check against total rate limit timeout budget
                if total_rate_limit_wait + wait_duration > rate_limit_timeout {
                    return Ok(PublishResult::Failed(anyhow!(
                        "Rate limit wait would exceed --rate-limit-timeout ({}s) for {}:\n{}",
                        rate_limit_timeout.as_secs(),
                        crate_name,
                        stderr
                    )));
                }

                total_rate_limit_wait += wait_duration;
                sleep(wait_duration);
                // Don't increment attempt - rate limit waits are tracked separately
                attempt -= 1;
                continue;
            }
            PublishErrorKind::PropagationDelay | PublishErrorKind::NetworkError => {
                if attempt < MAX_RETRIES {
                    let kind_desc = match error_kind {
                        PublishErrorKind::PropagationDelay => "dependency not yet propagated",
                        PublishErrorKind::NetworkError => "network error",
                        _ => "transient error",
                    };
                    println!(
                        "  Attempt {}/{}: {}, retrying in {}s...",
                        attempt,
                        MAX_RETRIES,
                        kind_desc,
                        retry_delay.as_secs()
                    );
                    if args.verbose {
                        println!("  stderr: {}", stderr.lines().next().unwrap_or(""));
                    }
                    sleep(retry_delay);
                    continue;
                }
            }
            PublishErrorKind::Unknown => {
                // Unknown errors don't get retried
            }
        }

        // Max retries exceeded or non-retryable error
        return Ok(PublishResult::Failed(anyhow!(
            "cargo publish failed for {}:\n{}",
            crate_name,
            stderr
        )));
    }
}

/// Run pre-publish checks.
fn run_pre_publish_checks(args: &PublishArgs, workspace_version: &str) -> Result<()> {
    println!("Running pre-publish checks...\n");

    // Git status check
    if !args.skip_git_check {
        print!("  Checking git status... ");
        io::stdout().flush()?;

        let status = Command::new("git")
            .args(["status", "--porcelain"])
            .output()
            .context("Failed to run git status")?;

        if !status.stdout.is_empty() {
            println!("✗");
            bail!("Working directory is not clean. Commit or stash changes first.");
        }
        println!("✓");
    }

    // Version consistency check
    if !args.skip_version_check {
        print!("  Checking version consistency... ");
        io::stdout().flush()?;

        let metadata = MetadataCommand::new()
            .no_deps()
            .exec()
            .context("Failed to load cargo metadata")?;

        let workspace_member_ids: HashSet<_> = metadata.workspace_members.iter().collect();

        let mut inconsistent = Vec::new();
        for pkg in &metadata.packages {
            if !workspace_member_ids.contains(&pkg.id) {
                continue;
            }
            // Skip non-publishable crates
            if pkg.publish.as_ref().is_some_and(|p| p.is_empty()) {
                continue;
            }
            if pkg.name == "xtask" || pkg.name == "tokmd-fuzz" || pkg.name == "fuzz" {
                continue;
            }

            let pkg_version = pkg.version.to_string();
            if pkg_version != workspace_version {
                inconsistent.push(format!("{} ({})", pkg.name, pkg_version));
            }
        }

        if !inconsistent.is_empty() {
            println!("✗");
            bail!(
                "Version mismatch! Expected {}, but found:\n  {}",
                workspace_version,
                inconsistent.join("\n  ")
            );
        }
        println!("✓ (all crates at {})", workspace_version);
    }

    // Changelog check
    if !args.skip_changelog_check {
        print!("  Checking CHANGELOG.md contains {}... ", workspace_version);
        io::stdout().flush()?;

        let changelog_path = Path::new("CHANGELOG.md");
        if !changelog_path.exists() {
            println!("✗");
            bail!("CHANGELOG.md not found");
        }

        let changelog =
            std::fs::read_to_string(changelog_path).context("Failed to read CHANGELOG.md")?;

        // Look for version header like [1.3.0] or ## 1.3.0
        let version_patterns = [
            format!("[{}]", workspace_version),
            format!("## {}", workspace_version),
        ];

        let has_version = version_patterns
            .iter()
            .any(|pattern| changelog.contains(pattern));

        if !has_version {
            println!("✗");
            bail!(
                "CHANGELOG.md does not contain version {}. Add a changelog entry first.",
                workspace_version
            );
        }
        println!("✓");
    }

    // Tests
    if !args.skip_tests {
        println!("  Running tests...");
        let test_status = Command::new("cargo")
            .args([
                "test",
                "--workspace",
                "--all-features",
                "--exclude",
                "tokmd-fuzz",
                "--locked",
            ])
            .status()
            .context("Failed to run tests")?;

        if !test_status.success() {
            bail!("Tests failed");
        }
        println!("  ✓ Tests passed");
    }

    println!();
    Ok(())
}

/// Create and push a git tag.
fn create_git_tag(args: &PublishArgs, version: &str) -> Result<()> {
    let tag = args.tag_format.replace("{version}", version);

    // Check if tag already exists
    let tag_check = Command::new("git")
        .args(["tag", "-l", &tag])
        .output()
        .context("Failed to check existing tags")?;

    if !tag_check.stdout.is_empty() {
        println!("Tag {} already exists, skipping tag creation.", tag);
        return Ok(());
    }

    println!("Creating git tag: {}", tag);

    let status = Command::new("git")
        .args(["tag", "-a", &tag, "-m", &format!("Release {}", tag)])
        .status()
        .context("Failed to create git tag")?;

    if !status.success() {
        bail!("Failed to create git tag");
    }

    println!("Pushing tag to origin...");
    let push_status = Command::new("git")
        .args(["push", "origin", &tag])
        .status()
        .context("Failed to push git tag")?;

    if !push_status.success() {
        bail!("Failed to push git tag");
    }

    println!("  ✓ Tag {} created and pushed", tag);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Datelike;

    #[test]
    fn test_classify_already_published() {
        assert!(matches!(
            classify_publish_error("crate version `1.0.0` is already uploaded"),
            PublishErrorKind::AlreadyPublished
        ));
        assert!(matches!(
            classify_publish_error("the crate version 1.0.0 already exists"),
            PublishErrorKind::AlreadyPublished
        ));
    }

    #[test]
    fn test_classify_auth_error() {
        assert!(matches!(
            classify_publish_error("token is invalid"),
            PublishErrorKind::AuthError
        ));
        assert!(matches!(
            classify_publish_error("error: not logged in"),
            PublishErrorKind::AuthError
        ));
    }

    #[test]
    fn test_classify_propagation_error() {
        assert!(matches!(
            classify_publish_error("failed to select a version for the requirement `foo`"),
            PublishErrorKind::PropagationDelay
        ));
        assert!(matches!(
            classify_publish_error("no matching package named `bar`"),
            PublishErrorKind::PropagationDelay
        ));
    }

    #[test]
    fn test_classify_manifest_error() {
        assert!(matches!(
            classify_publish_error("invalid manifest: missing field `description`"),
            PublishErrorKind::ManifestError
        ));
    }

    #[test]
    fn test_is_publish_dependency() {
        assert!(is_publish_dependency(&DependencyKind::Normal));
        assert!(is_publish_dependency(&DependencyKind::Build));
        assert!(is_publish_dependency(&DependencyKind::Development));
    }

    #[test]
    fn test_classify_rate_limit() {
        // HTTP 429 status code in error message
        assert!(matches!(
            classify_publish_error(
                "the remote server responded with an error (status 429 Too Many Requests): \
                 You have published too many new crates"
            ),
            PublishErrorKind::RateLimited
        ));

        // "too many" without explicit 429
        assert!(matches!(
            classify_publish_error(
                "You have published too many requests in a short period of time"
            ),
            PublishErrorKind::RateLimited
        ));

        // Just the 429 status
        assert!(matches!(
            classify_publish_error("error: 429 rate limit exceeded"),
            PublishErrorKind::RateLimited
        ));

        // unrelated "too many" should not match rate limiting
        assert!(matches!(
            classify_publish_error("open files: too many open files"),
            PublishErrorKind::Unknown
        ));
    }

    #[test]
    fn test_parse_rate_limit_timestamp() {
        // Full crates.io error message
        let stderr = "the remote server responded with an error (status 429 Too Many Requests): \
                       You have published too many new crates in a short period of time. \
                       Please try again after Tue, 24 Feb 2026 16:57:08 GMT or email help@crates.io";
        let ts = parse_rate_limit_timestamp(stderr);
        assert!(ts.is_some(), "should parse RFC2822 timestamp");
        let ts = ts.unwrap();
        assert_eq!(ts.year(), 2026);
        assert_eq!(ts.month(), 2);
        assert_eq!(ts.day(), 24);

        // No timestamp present
        assert!(parse_rate_limit_timestamp("some random error").is_none());

        // Marker present but invalid timestamp
        assert!(parse_rate_limit_timestamp("try again after not-a-real-timestamp").is_none());
    }
}
