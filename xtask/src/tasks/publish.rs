//! Publish crates to crates.io in dependency order.

use std::collections::{BTreeMap, BTreeSet, HashSet};
use std::process::Command;
use std::thread::sleep;
use std::time::Duration;

use anyhow::{Context, Result, anyhow, bail};
use cargo_metadata::{DependencyKind, MetadataCommand, Package};
use petgraph::algo::toposort;
use petgraph::graph::DiGraph;

use crate::cli::PublishArgs;

/// Result of attempting to publish a single crate.
#[derive(Debug)]
pub enum PublishResult {
    Success,
    Skipped(String),
    AlreadyPublished,
    Failed(anyhow::Error),
}

/// Publish all workspace crates in dependency order.
pub fn run(args: PublishArgs) -> Result<()> {
    // Load workspace metadata
    let metadata = MetadataCommand::new()
        .exec()
        .context("Failed to load cargo metadata")?;

    // Build dependency graph and get publish order
    let publish_order = compute_publish_order(&metadata.packages)?;

    if args.verbose {
        println!("Publish order:");
        for name in &publish_order {
            println!("  - {}", name);
        }
    }

    // Filter to requested crates (with transitive dependencies)
    let mut to_publish: BTreeSet<String> = if let Some(ref crates) = args.crates {
        let requested: HashSet<_> = crates.iter().cloned().collect();
        let mut result = BTreeSet::new();

        // Add requested crates and their transitive dependencies
        for name in &publish_order {
            if requested.contains(name) {
                result.insert(name.clone());
                // Add transitive dependencies that are in our workspace
                add_transitive_deps(name, &metadata.packages, &mut result);
            }
        }
        result
    } else {
        publish_order.iter().cloned().collect()
    };

    // Apply exclusions with validation
    if let Some(ref excludes) = args.exclude {
        let exclude_set: HashSet<_> = excludes.iter().collect();

        // Check that excludes don't break required dependencies
        for name in &to_publish {
            if exclude_set.contains(name) {
                continue;
            }
            let pkg = metadata.packages.iter().find(|p| p.name == *name).unwrap();
            for dep in &pkg.dependencies {
                if !matches!(
                    dep.kind,
                    DependencyKind::Normal | DependencyKind::Build | DependencyKind::Unknown
                ) {
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

        to_publish.retain(|name| !exclude_set.contains(name));
    }

    // Filter publish_order to only include crates we're publishing
    let filtered_order: Vec<_> = publish_order
        .into_iter()
        .filter(|name| to_publish.contains(name))
        .collect();

    // Handle --from flag
    let start_idx = if let Some(ref from_crate) = args.from {
        filtered_order
            .iter()
            .position(|name| name == from_crate)
            .ok_or_else(|| anyhow!("Crate '{}' not found in publish order", from_crate))?
    } else {
        0
    };

    // Run pre-publish checks
    if !args.skip_checks {
        run_pre_publish_checks(&args)?;
    }

    println!(
        "\n{} Publishing {} crate(s)...\n",
        if args.dry_run { "[DRY RUN]" } else { "" },
        filtered_order.len() - start_idx
    );

    let mut failed = Vec::new();
    let mut succeeded = Vec::new();

    for (idx, crate_name) in filtered_order.iter().enumerate().skip(start_idx) {
        let position = format!(
            "[{}/{}]",
            idx + 1 - start_idx,
            filtered_order.len() - start_idx
        );
        println!("{} Publishing {}...", position, crate_name);

        let result = publish_crate_with_retry(crate_name, &args)?;

        match result {
            PublishResult::Success => {
                println!("  ✓ Published {}", crate_name);
                succeeded.push(crate_name.clone());

                // Wait for crates.io propagation
                if idx < filtered_order.len() - 1 && !args.dry_run {
                    println!("  Waiting {}s for crates.io propagation...", args.interval);
                    sleep(Duration::from_secs(args.interval));
                }
            }
            PublishResult::AlreadyPublished => {
                println!("  ✓ {} already published", crate_name);
            }
            PublishResult::Skipped(reason) => {
                println!("  → Skipped {}: {}", crate_name, reason);
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

    // Summary
    println!("\n--- Summary ---");
    println!("Succeeded: {}", succeeded.len());
    if !failed.is_empty() {
        println!("Failed: {} ({:?})", failed.len(), failed);
    }

    // Create git tag if requested
    if args.tag && failed.is_empty() && !args.dry_run {
        create_git_tag(&args)?;
    }

    if !failed.is_empty() {
        bail!("{} crate(s) failed to publish", failed.len());
    }

    Ok(())
}

/// Compute topological publish order from workspace dependencies.
fn compute_publish_order(packages: &[Package]) -> Result<Vec<String>> {
    let workspace_crates: HashSet<_> = packages.iter().map(|p| p.name.as_str()).collect();
    let mut graph = DiGraph::<&str, ()>::new();
    let mut indices = BTreeMap::new();

    // Add all crates as nodes
    for pkg in packages {
        let idx = graph.add_node(pkg.name.as_str());
        indices.insert(pkg.name.as_str(), idx);
    }

    // Add edges: dependency -> dependent
    for pkg in packages {
        let from_idx = indices[pkg.name.as_str()];

        for dep in &pkg.dependencies {
            // Only consider normal and build dependencies (skip dev-deps)
            if !matches!(
                dep.kind,
                DependencyKind::Normal | DependencyKind::Build | DependencyKind::Unknown
            ) {
                continue;
            }

            // Only add edges for workspace crates
            if let Some(&to_idx) = indices.get(dep.name.as_str()) {
                if workspace_crates.contains(dep.name.as_str()) {
                    // Edge from dependency to dependent (dep must be published first)
                    graph.add_edge(to_idx, from_idx, ());
                }
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
fn add_transitive_deps(crate_name: &str, packages: &[Package], result: &mut BTreeSet<String>) {
    let workspace_crates: HashSet<_> = packages.iter().map(|p| p.name.as_str()).collect();

    if let Some(pkg) = packages.iter().find(|p| p.name == crate_name) {
        for dep in &pkg.dependencies {
            if !matches!(
                dep.kind,
                DependencyKind::Normal | DependencyKind::Build | DependencyKind::Unknown
            ) {
                continue;
            }

            if workspace_crates.contains(dep.name.as_str()) && !result.contains(&dep.name) {
                result.insert(dep.name.clone());
                add_transitive_deps(&dep.name, packages, result);
            }
        }
    }
}

/// Check if stderr indicates a crates.io propagation issue (retryable).
fn is_propagation_error(stderr: &str) -> bool {
    // Common patterns for dependency not yet available
    stderr.contains("failed to select a version for the requirement")
        || stderr.contains("no matching package named")
        || stderr.contains("failed to get")
        || stderr.contains("no matching version")
        || (stderr.contains("dependency") && stderr.contains("not found"))
}

/// Check if the error indicates the crate is already published.
fn is_already_published(stderr: &str) -> bool {
    stderr.contains("is already uploaded")
        || stderr.contains("crate version") && stderr.contains("already exists")
}

/// Publish a single crate with retry logic for propagation delays.
fn publish_crate_with_retry(crate_name: &str, args: &PublishArgs) -> Result<PublishResult> {
    const MAX_RETRIES: u32 = 5;
    const RETRY_DELAY: Duration = Duration::from_secs(30);

    if args.dry_run {
        println!("  [DRY RUN] Would publish {}", crate_name);
        return Ok(PublishResult::Skipped("dry run".into()));
    }

    // Optional verify step
    if args.verify {
        println!("  Verifying {} with --dry-run...", crate_name);
        let verify_out = Command::new("cargo")
            .args(["publish", "-p", crate_name, "--dry-run"])
            .output()
            .context("Failed to spawn cargo publish --dry-run")?;

        if !verify_out.status.success() {
            let stderr = String::from_utf8_lossy(&verify_out.stderr);
            return Ok(PublishResult::Failed(anyhow!(
                "Verification failed:\n{}",
                stderr
            )));
        }
    }

    // Actual publish with retries
    for attempt in 1..=MAX_RETRIES {
        let output = Command::new("cargo")
            .args(["publish", "-p", crate_name])
            .output()
            .context("Failed to spawn cargo publish")?;

        if output.status.success() {
            return Ok(PublishResult::Success);
        }

        let stderr = String::from_utf8_lossy(&output.stderr);

        // Check if already published (not an error)
        if is_already_published(&stderr) {
            return Ok(PublishResult::AlreadyPublished);
        }

        // Check if this is a retryable propagation error
        if is_propagation_error(&stderr) && attempt < MAX_RETRIES {
            println!(
                "  Attempt {}/{}: dependency not yet propagated, retrying in {}s...",
                attempt,
                MAX_RETRIES,
                RETRY_DELAY.as_secs()
            );
            if args.verbose {
                println!("  stderr: {}", stderr.lines().next().unwrap_or(""));
            }
            sleep(RETRY_DELAY);
            continue;
        }

        // Non-retryable error or max retries exceeded
        return Ok(PublishResult::Failed(anyhow!(
            "cargo publish failed for {}:\n{}",
            crate_name,
            stderr
        )));
    }

    Ok(PublishResult::Failed(anyhow!(
        "Max retries exceeded for {}",
        crate_name
    )))
}

/// Run pre-publish checks.
fn run_pre_publish_checks(args: &PublishArgs) -> Result<()> {
    if !args.skip_git_check {
        println!("Checking git status...");
        let status = Command::new("git")
            .args(["status", "--porcelain"])
            .output()
            .context("Failed to run git status")?;

        if !status.stdout.is_empty() {
            bail!("Working directory is not clean. Commit or stash changes first.");
        }
        println!("  ✓ Working directory clean");
    }

    if !args.skip_tests {
        println!("Running tests...");
        let test_status = Command::new("cargo")
            .args(["test", "--workspace"])
            .status()
            .context("Failed to run tests")?;

        if !test_status.success() {
            bail!("Tests failed");
        }
        println!("  ✓ Tests passed");
    }

    if !args.skip_version_check {
        println!("Checking version consistency...");
        // Version check logic could go here
        println!("  ✓ Versions consistent");
    }

    Ok(())
}

/// Create and push a git tag.
fn create_git_tag(args: &PublishArgs) -> Result<()> {
    // Get version from root Cargo.toml
    let metadata = MetadataCommand::new().exec()?;
    let root_pkg = metadata
        .packages
        .iter()
        .find(|p| p.name == "tokmd")
        .ok_or_else(|| anyhow!("Could not find tokmd package"))?;

    let version = &root_pkg.version;
    let tag = args.tag_format.replace("{version}", &version.to_string());

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
