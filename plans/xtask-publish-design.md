# xtask Publish Command Design Document

## Overview

This document outlines the design for converting the PowerShell script [`scripts/publish-all.ps1`](scripts/publish-all.ps1:1) into a proper Rust `xtask` command. The xtask pattern is a cargo convention where development tasks are implemented as Rust binaries within the workspace, invoked via `cargo xtask <command>`.

## Current State Analysis

### Existing PowerShell Script

The [`scripts/publish-all.ps1`](scripts/publish-all.ps1:1) script currently:

1. **Publishes 8 crates in dependency order:**
   - tokmd-types
   - tokmd-config
   - tokmd-model
   - tokmd-format
   - tokmd-scan
   - tokmd-tokeignore
   - tokmd-core
   - tokmd

2. **Features:**
   - `-DryRun` flag for dry-run mode
   - 10-second delay between publishes (skipped in dry-run) for crates.io propagation
   - Aborts on any failure
   - Color-coded console output

3. **Limitations:**
   - PowerShell-only (not cross-platform)
   - Hard-coded crate list
   - Limited error handling
   - No extensibility

### Existing Alternatives

The [`Justfile`](Justfile:1) already contains publish helpers using `cargo-workspaces`:

```justfile
publish-dry: setup-publish
    cargo ws publish --from-git --dry-run

publish: setup-publish
    cargo ws publish --from-git --publish-interval 10 --yes
```

However, the PowerShell script provides a simpler, dependency-free approach with explicit control over the publish order.

## Architecture Design

### 1. Crate Structure and Location

**Recommended Location:** `xtask/` (top-level directory, sibling to `crates/`)

**Rationale:**
- Following the standard xtask convention
- Clear separation from production crates
- Easy to invoke via `cargo xtask <command>`

**Directory Structure:**

```
tokmd/
├── xtask/
│   ├── Cargo.toml
│   └── src/
│       ├── main.rs
│       ├── cli.rs
│       └── tasks/
│           ├── mod.rs
│           └── publish.rs
```

### 2. Workspace Integration

The xtask crate should be **added as a workspace member** but **excluded from default builds**:

**Workspace Cargo.toml changes:**

```toml
[workspace]
resolver = "2"
members = [
    "crates/tokmd",
    "crates/tokmd-analysis",
    "crates/tokmd-analysis-format",
    "crates/tokmd-analysis-types",
    "crates/tokmd-config",
    "crates/tokmd-content",
    "crates/tokmd-core",
    "crates/tokmd-format",
    "crates/tokmd-fun",
    "crates/tokmd-gate",
    "crates/tokmd-git",
    "crates/tokmd-model",
    "crates/tokmd-redact",
    "crates/tokmd-scan",
    "crates/tokmd-tokeignore",
    "crates/tokmd-types",
    "crates/tokmd-walk",
    "fuzz",
    "xtask",
]
default-members = [
    "crates/tokmd",
    "crates/tokmd-analysis",
    "crates/tokmd-analysis-format",
    "crates/tokmd-analysis-types",
    "crates/tokmd-config",
    "crates/tokmd-content",
    "crates/tokmd-core",
    "crates/tokmd-format",
    "crates/tokmd-fun",
    "crates/tokmd-gate",
    "crates/tokmd-git",
    "crates/tokmd-model",
    "crates/tokmd-redact",
    "crates/tokmd-scan",
    "crates/tokmd-tokeignore",
    "crates/tokmd-types",
    "crates/tokmd-walk",
]
# xtask is a workspace member but excluded from default-members
```

**Cargo Alias Configuration:**

Create `.cargo/config.toml` to enable the `cargo xtask` alias:

```toml
# .cargo/config.toml
[alias]
xtask = "run -p xtask --"
```

This makes `cargo xtask publish` work as documented.

### 3. CLI Interface Design

**Command Pattern:** `cargo xtask publish [OPTIONS]`

**CLI Structure (using clap derive):**

```rust
#[derive(Parser, Debug)]
#[command(name = "xtask")]
#[command(about = "Development tasks for tokmd", long_about = None)]
struct XtaskCli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Publish all crates in dependency order
    Publish(PublishArgs),
}

#[derive(Args, Debug)]
struct PublishArgs {
    /// Run in dry-run mode (no actual publishing)
    #[arg(long, short = 'n')]
    dry_run: bool,

    /// Run cargo publish --dry-run for each crate before actual publish
    #[arg(long)]
    verify: bool,

    /// Seconds to wait between publishes for crates.io propagation
    #[arg(long, default_value = "10")]
    interval: u64,

    /// Maximum duration (in seconds) for each publish attempt
    #[arg(long, default_value = "300")]
    timeout: u64,

    /// Continue on failure instead of aborting
    #[arg(long)]
    continue_on_error: bool,

    /// Resume publishing from this crate (skips crates before this one)
    #[arg(long)]
    from: Option<String>,

    /// Verbose output
    #[arg(long, short = 'v')]
    verbose: bool,

    /// Skip all pre-publish checks
    #[arg(long)]
    skip_checks: bool,

    /// Skip running tests
    #[arg(long)]
    skip_tests: bool,

    /// Skip git status check
    #[arg(long)]
    skip_git_check: bool,

    /// Skip CHANGELOG verification
    #[arg(long)]
    skip_changelog_check: bool,

    /// Skip version consistency check
    #[arg(long)]
    skip_version_check: bool,

    /// Specific crates to publish (comma-separated). If not specified, all crates are published.
    #[arg(long, value_delimiter = ',')]
    crates: Option<Vec<String>>,

    /// Exclude specific crates from publishing (comma-separated)
    #[arg(long, value_delimiter = ',')]
    exclude: Option<Vec<String>>,

    /// Create and push git tag after successful publish (e.g., v1.3.0)
    #[arg(long)]
    tag: bool,

    /// Custom tag format (use {version} placeholder, e.g., "release-{version}")
    #[arg(long, default_value = "v{version}")]
    tag_format: String,
}
```

**Usage Examples:**

```bash
# Dry run to verify everything is ready (includes pre-publish checks)
cargo xtask publish --dry-run

# Full publish with default 10-second interval
cargo xtask publish

# Full publish with pre-publish verification (dry-run each crate first)
cargo xtask publish --verify

# Full publish with custom interval
cargo xtask publish --interval 15

# Verbose mode for debugging
cargo xtask publish --verbose

# Continue even if a crate fails
cargo xtask publish --continue-on-error

# Resume from a specific crate after a failure
cargo xtask publish --from tokmd-config

# Skip all pre-publish checks (not recommended)
cargo xtask publish --skip-checks

# Skip only the git status check
cargo xtask publish --skip-git-check

# Publish only specific crates (maintains dependency order)
cargo xtask publish --crates tokmd-types,tokmd-config

# Publish all crates except specific ones
cargo xtask publish --exclude tokmd-fuzz,tokmd-examples

# Publish and create git tag
cargo xtask publish --tag

# Publish with custom tag format
cargo xtask publish --tag --tag-format "release-{version}"

# Publish a single crate for testing
cargo xtask publish --crates tokmd-types --dry-run
```

### 4. Implementation Approach

#### 4.1 Main Entry Point ([`xtask/src/main.rs`](xtask/src/main.rs))

```rust
use anyhow::Result;
use clap::Parser;

mod cli;
mod tasks;

use cli::XtaskCli;

fn main() -> Result<()> {
    let cli = XtaskCli::parse();

    match cli.command {
        cli::Commands::Publish(args) => tasks::publish::run(args),
    }
}
```

#### 4.2 CLI Definition ([`xtask/src/cli.rs`](xtask/src/cli.rs))

Contains the clap derive structs as shown in section 3.

#### 4.3 Publish Task Implementation ([`xtask/src/tasks/publish.rs`](xtask/src/tasks/publish.rs))

**Key Components:**

1. **Pre-publish Checks:**
   - Run all tests: `cargo test --all-features`
   - Check git status for uncommitted changes
   - Verify CHANGELOG.md has an entry for the new version
   - Each check can be individually skipped via flags

2. **Crate Registry:**
   - Define the publishable crates in dependency order
   - Could be a static array or loaded from a config file

3. **Publish Function:**
   - Iterate through crates in order
   - Execute `cargo publish -p <crate>` with appropriate flags
   - Handle success/failure
   - Implement delay between publishes

4. **Error Handling:**
   - Use `anyhow` for error propagation
   - Provide clear error messages
   - Support `--continue-on-error` flag

5. **Progress Reporting:**
   - Color-coded console output (using `indicatif` for progress bars)
   - Clear status messages for each crate

**Pseudocode:**

```rust
pub fn run(args: PublishArgs) -> Result<()> {
    // Run pre-publish checks
    if !args.skip_checks {
        run_pre_publish_checks(&args)?;
    }

    // Resolve publish order dynamically
    let crates = resolve_publish_order(&args)?;
    let total = crates.len();

    if total == 0 {
        return Err(anyhow!("No crates to publish."));
    }

    if args.verbose {
        println!("Publish order: {}", crates.join(", "));
    }

    // Handle --from flag for resume capability
    let start_index = if let Some(ref from_crate) = args.from {
        crates.iter()
            .position(|c| c == from_crate)
            .ok_or_else(|| anyhow!("Crate '{}' not found in publish order", from_crate))?
    } else {
        0
    };

    let progress = ProgressBar::new((total - start_index) as u64);

    for (index, crate_name) in crates.iter().enumerate().skip(start_index) {
        progress.set_message(format!("Publishing {} ({}/{})", crate_name, index + 1, total));

        // Run pre-publish verification if requested
        if args.verify && !args.dry_run {
            verify_crate(crate_name, &args)?;
        }

        // Publish with retry logic for crates.io propagation
        match publish_crate_with_retry(crate_name, &args)? {
            PublishResult::Success => {
                if !args.dry_run && index < total - 1 {
                    sleep(Duration::from_secs(args.interval));
                }
            }
            PublishResult::Failed(err) => {
                if !args.continue_on_error {
                    return Err(err);
                }
                eprintln!("⚠️  Failed to publish {}: {}", crate_name, err);
            }
        }
    }

    progress.finish_with_message("All crates published successfully!");

    // Create and push git tag if requested
    if args.tag && !args.dry_run {
        create_and_push_git_tag(&args)?;
    }

    Ok(())
}

enum PublishResult {
    Success,
    Failed(anyhow::Error),
}

fn verify_crate(crate_name: &str, args: &PublishArgs) -> Result<()> {
    println!("🔍 Verifying {} (dry-run)...", crate_name);
    
    let status = Command::new("cargo")
        .args(["publish", "-p", crate_name, "--dry-run"])
        .status()?;

    if !status.success() {
        return Err(anyhow!("Dry-run verification failed for {}", crate_name));
    }
    
    println!("✅ {} verified successfully", crate_name);
    Ok(())
}

fn publish_crate_with_retry(crate_name: &str, args: &PublishArgs) -> Result<PublishResult> {
    let max_retries = 3;
    let retry_delay = Duration::from_secs(args.interval);
    
    for attempt in 1..=max_retries {
        println!("📦 Publishing {} (attempt {}/{})...", crate_name, attempt, max_retries);
        
        let result = Command::new("cargo")
            .args(["publish", "-p", crate_name])
            .status();
        
        match result {
            Ok(status) if status.success() => {
                println!("✅ {} published successfully", crate_name);
                return Ok(PublishResult::Success);
            }
            Ok(_) => {
                let err = anyhow!("cargo publish failed for {}", crate_name);
                return Ok(PublishResult::Failed(err));
            }
            Err(e) => {
                // Check if it's a "dependency not found" error that might resolve with retry
                let error_str = e.to_string().to_lowercase();
                let is_dependency_error = error_str.contains("dependency")
                    || error_str.contains("not found")
                    || error_str.contains("version");
                
                if is_dependency_error && attempt < max_retries {
                    println!("⏳ Waiting {}s for crates.io propagation...", args.interval);
                    sleep(retry_delay);
                    continue;
                }
                
                return Ok(PublishResult::Failed(anyhow!(e)));
            }
        }
    }
    
    Ok(PublishResult::Failed(anyhow!("Max retries exceeded for {}", crate_name)))
}

fn create_and_push_git_tag(args: &PublishArgs) -> Result<()> {
    let version = get_workspace_version()?;
    let tag_name = args.tag_format.replace("{version}", &version);
    
    println!("🏷️  Creating git tag: {}", tag_name);
    
    // Create tag
    let status = Command::new("git")
        .args(["tag", "-a", &tag_name, "-m", &format!("Release {}", version)])
        .status()?;
    
    if !status.success() {
        return Err(anyhow!("Failed to create git tag {}", tag_name));
    }
    
    // Push tag
    println!("📤 Pushing tag to remote...");
    let status = Command::new("git")
        .args(["push", "origin", &tag_name])
        .status()?;
    
    if !status.success() {
        return Err(anyhow!("Failed to push git tag {}", tag_name));
    }
    
    println!("✅ Tag {} created and pushed successfully", tag_name);
    Ok(())
}

fn run_pre_publish_checks(args: &PublishArgs) -> Result<()> {
    // Check 1: Run tests
    if !args.skip_tests {
        println!("🧪 Running tests...");
        run_cargo_test()?;
    }

    // Check 2: Git status
    if !args.skip_git_check {
        println!("🔍 Checking git status...");
        check_git_status()?;
    }

    // Check 3: CHANGELOG
    if !args.skip_changelog_check {
        println!("📝 Verifying CHANGELOG...");
        verify_changelog()?;
    }

    // Check 4: Version consistency
    if !args.skip_version_check {
        println!("🔢 Checking version consistency...");
        check_version_consistency()?;
    }

    // Check 5: Registry token
    println!("🔑 Checking registry token...");
    check_registry_token()?;

    println!("✅ All pre-publish checks passed!");
    Ok(())
}
```

#### 4.3.1 Pre-publish Check Implementation

**Test Check:**
```rust
fn run_cargo_test() -> Result<()> {
    let status = Command::new("cargo")
        .args([
            "test",
            "--workspace",
            "--all-features",
            "--exclude",
            "tokmd-fuzz",
        ])
        .status()?;

    if !status.success() {
        return Err(anyhow!("Tests failed. Aborting publish."));
    }
    Ok(())
}
```

**Git Status Check:**
```rust
fn check_git_status() -> Result<()> {
    let output = Command::new("git")
        .args(["status", "--porcelain"])
        .output()?;

    if !output.stdout.is_empty() {
        let changes = String::from_utf8_lossy(&output.stdout);
        return Err(anyhow!(
            "Uncommitted changes detected:\n{}\nCommit or stash changes before publishing.",
            changes
        ));
    }
    Ok(())
}
```

**CHANGELOG Verification:**
```rust
fn verify_changelog() -> Result<()> {
    // Read workspace version from [workspace.package] in root Cargo.toml
    let version = get_workspace_version()?;

    // Check CHANGELOG.md for version entry
    let changelog = std::fs::read_to_string("CHANGELOG.md")?;

    if !changelog.contains(&format!("## [{}]", version)) {
        return Err(anyhow!(
            "CHANGELOG.md does not contain an entry for version {}.\n\
             Please update CHANGELOG.md before publishing.",
            version
        ));
    }
    Ok(())
}

fn get_workspace_version() -> Result<String> {
    let cargo_toml = std::fs::read_to_string("Cargo.toml")?;
    let value: toml::Value = toml::from_str(&cargo_toml)?;
    
    value.get("workspace")
        .and_then(|w| w.get("package"))
        .and_then(|p| p.get("version"))
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .ok_or_else(|| anyhow!("Could not find [workspace.package.version] in Cargo.toml"))
}
```

**Version Consistency Check:**
```rust
fn check_version_consistency() -> Result<()> {
    let metadata = MetadataCommand::new()
        .no_deps()
        .exec()?;

    let workspace_version = get_workspace_version()?;
    let mut mismatches = Vec::new();

    for pkg_id in metadata.workspace_members() {
        let pkg = &metadata[pkg_id];
        if pkg.publish.map_or(true, |v| !v.is_empty()) {
            let pkg_version = pkg.version.to_string();
            if pkg_version != workspace_version {
                mismatches.push((pkg.name.clone(), pkg_version));
            }
        }
    }

    if !mismatches.is_empty() {
        let msg = mismatches
            .into_iter()
            .map(|(name, ver)| format!("  - {} = {}", name, ver))
            .collect::<Vec<_>>()
            .join("\n");
        return Err(anyhow!(
            "Version mismatch detected. Workspace version is {}.\n\
             The following crates have different versions:\n{}\n\
             Please update crate versions to match.",
            workspace_version, msg
        ));
    }

    Ok(())
}
```

**Registry Token Check:**
```rust
fn check_registry_token() -> Result<()> {
    if std::env::var("CARGO_REGISTRY_TOKEN").is_err() {
        return Err(anyhow!(
            "CARGO_REGISTRY_TOKEN environment variable is not set.\n\
             Please set it before publishing:\n\
               export CARGO_REGISTRY_TOKEN=your_token_here  # Linux/macOS\n\
               set CARGO_REGISTRY_TOKEN=your_token_here     # Windows"
        ));
    }
    Ok(())
}
```

#### 4.4 Dependency Order Management

**Dynamic Dependency Resolution (Recommended)**

The xtask will use `cargo_metadata` to dynamically resolve the publish order by:

1. **Parsing the workspace Cargo.toml** to get all workspace members
2. **Building a dependency graph** of all crates
3. **Topologically sorting** to determine publish order (dependencies before dependents)
4. **Filtering** to only include crates marked for publication
5. **Applying `--crates` and `--exclude` filters** for selective publishing

**Implementation:**

```rust
use cargo_metadata::{Metadata, MetadataCommand};
use petgraph::algo::toposort;
use petgraph::graph::{DiGraph, NodeIndex};

struct CrateInfo {
    name: String,
    path: String,
    publishable: bool,
    is_workspace: bool,
}

fn resolve_publish_order(args: &PublishArgs) -> Result<Vec<String>> {
    // Load workspace metadata (with deps for dependency graph)
    let metadata = MetadataCommand::new()
        .exec()?;

    // Collect crate information
    let mut crates: Vec<CrateInfo> = metadata
        .workspace_members()
        .iter()
        .map(|id| {
            let pkg = &metadata[id];
            CrateInfo {
                name: pkg.name.clone(),
                path: pkg.manifest_path.to_string_lossy().to_string(),
                publishable: pkg.publish.map_or(true, |v| !v.is_empty()),
                is_workspace: pkg.source.is_none(), // No source = workspace-local
            }
        })
        .collect();

    // Build dependency graph
    let mut graph = DiGraph::<String, ()>::new();
    let mut indices: std::collections::HashMap<String, NodeIndex> = std::collections::HashMap::new();

    // Add nodes for all publishable workspace crates (exclude xtask itself)
    for crate_info in &crates {
        if crate_info.publishable && crate_info.is_workspace && crate_info.name != "xtask" {
            let idx = graph.add_node(crate_info.name.clone());
            indices.insert(crate_info.name.clone(), idx);
        }
    }

    // Add edges for dependencies (crate A depends on crate B => B -> A)
    for pkg_id in metadata.workspace_members() {
        let pkg = &metadata[pkg_id];
        if pkg.name == "xtask" {
            continue; // Skip xtask from dependency graph
        }

        if let Some(from_idx) = indices.get(&pkg.name) {
            // Include both normal and build dependencies
            for dep in pkg.dependencies.iter().chain(pkg.build_dependencies.iter()) {
                if let Some(to_idx) = indices.get(&dep.name) {
                    graph.add_edge(*to_idx, *from_idx, ());
                }
            }
        }
    }

    // Topologically sort to get publish order
    let sorted = toposort(&graph, None)
        .map_err(|e| anyhow!("Cycle detected in dependencies: {:?}", e))?;

    // Convert to crate names in publish order
    let mut order: Vec<String> = sorted
        .into_iter()
        .map(|idx| graph[idx].clone())
        .collect();

    // Apply --crates filter with dependency closure
    if let Some(selected) = &args.crates {
        let closure = compute_dependency_closure(&graph, &indices, selected)?;
        order.retain(|name| closure.contains(name));
    }

    // Apply --exclude filter
    if let Some(excluded) = &args.exclude {
        order.retain(|name| !excluded.contains(name));
    }

    Ok(order)
}

fn compute_dependency_closure(
    graph: &DiGraph<String, ()>,
    indices: &std::collections::HashMap<String, NodeIndex>,
    selected: &[String],
) -> Result<std::collections::HashSet<String>> {
    let mut closure = std::collections::HashSet::new();
    
    for crate_name in selected {
        if let Some(&idx) = indices.get(crate_name) {
            // Use DFS to find all transitive dependencies
            let mut visited = std::collections::HashSet::new();
            let mut stack = vec![idx];
            
            while let Some(current) = stack.pop() {
                if visited.insert(current) {
                    // Add all dependencies (incoming edges)
                    for neighbor in graph.neighbors_directed(current, petgraph::Direction::Incoming) {
                        stack.push(neighbor);
                    }
                }
            }
            
            // Add all visited crates to closure
            for node in visited {
                closure.insert(graph[node].clone());
            }
        }
    }
    
    // Also include the selected crates themselves
    for crate_name in selected {
        closure.insert(crate_name.clone());
    }
    
    Ok(closure)
}
```

**Benefits of Dynamic Resolution:**
- Automatically adapts to changes in workspace structure
- No need to manually maintain a crate list
- Detects dependency cycles early
- Handles new crates automatically
- Validates dependency relationships

**Configuration for Non-publishable Crates:**

Crates that should NOT be published (e.g., internal-only crates) can be marked in their Cargo.toml:

```toml
[package]
name = "tokmd-internal"
publish = false  # This crate will be excluded from publish
```

**Config File Override:**

For cases where dynamic resolution needs to be overridden (e.g., specific crates need to be published in a different order), a config file can be used:

```toml
# xtask.toml
[publish]
# Explicitly list crates to publish (overrides dynamic resolution)
crates = [
    "tokmd-types",
    "tokmd-config",
    "tokmd-model",
    "tokmd-format",
    "tokmd-scan",
    "tokmd-tokeignore",
    "tokmd-core",
    "tokmd",
]
```


### 5. Dependencies

**Required Dependencies:**

```toml
[package]
name = "xtask"
version.workspace = true
edition.workspace = true
publish = false  # Never publish xtask

[dependencies]
anyhow = "1.0"
clap = { version = "4.5", features = ["derive"] }
indicatif = "0.17"
console = "0.15"
serde = { version = "1.0", features = ["derive"] }
toml = "0.8"
cargo_metadata = "0.18"
petgraph = "0.6"
```

**Rationale for Dependencies:**
- `anyhow`: Error handling and propagation
- `clap`: CLI argument parsing with derive macros
- `indicatif`: Progress bars and spinners
- `console`: Cross-platform terminal styling
- `serde` and `toml`: Parsing `Cargo.toml` to get workspace version for CHANGELOG verification
- `cargo_metadata`: Dynamic dependency resolution from workspace metadata
- `petgraph`: Topological sorting of dependencies

**Optional Dependencies (for future enhancements):**

```toml
# For git operations (beyond basic status checking)
git2 = "0.18"
```

### 6. Cross-Platform Considerations

The xtask approach inherently solves the cross-platform issues:

| Issue | PowerShell Script | xtask (Rust) |
|-------|------------------|--------------|
| Windows | Native | Native |
| macOS/Linux | Requires PowerShell | Native |
| CI/CD | May need pwsh | Standard cargo |
| Dependencies | PowerShell 5.1+ | cargo only |

**Additional Considerations:**

1. **Path Handling:** Use `std::path::Path` and `std::path::PathBuf` for cross-platform paths
2. **Command Execution:** Use `std::process::Command` which works on all platforms
3. **Terminal Output:** Use `indicatif` and `console` crates for consistent cross-platform output
4. **Sleep:** Use `std::thread::sleep` with `std::time::Duration`

### 7. Integration with Existing Tooling

#### 7.1 Justfile Integration

Update the [`Justfile`](Justfile:1) to use xtask:

```justfile
# Publishing helpers (now using xtask)
publish-dry:
    cargo xtask publish --dry-run

publish:
    cargo xtask publish

publish-verbose:
    cargo xtask publish --verbose
```

#### 7.2 GitHub Actions Integration

The xtask can be used in CI workflows:

```yaml
- name: Publish crates
  run: cargo xtask publish
  env:
    CARGO_REGISTRY_TOKEN: ${{ secrets.CARGO_REGISTRY_TOKEN }}
```

### 8. Error Handling Strategy

**Error Categories:**

1. **User Errors:**
   - Invalid arguments
   - Missing credentials
   - Network issues
   - Action: Clear error message, exit with non-zero code

2. **Publish Errors:**
   - Version already published
   - Dependency conflicts
   - Registry errors
   - Action: Display error, continue if `--continue-on-error`, otherwise abort

3. **System Errors:**
   - Cargo not found
   - File system issues
   - Action: Fatal error, abort immediately

**Error Output Format:**

```
❌ Failed to publish tokmd-config: version 1.3.0 is already published
   Run with --continue-on-error to skip this crate and continue
```

### 9. Future Enhancements

**Phase 2 Features:**

1. **Version Bumping:**
   ```bash
   cargo xtask publish --bump patch
   ```

2. **Config File Support:**
   - Store publish order in `xtask.toml`
   - Configure custom intervals per crate
   - Define pre-publish check profiles

3. **Rollback Support:**
   - `cargo xtask publish --rollback`
   - Yank published crates in reverse order

4. **Parallel Publishing:**
   - Publish independent crates in parallel (with proper dependency tracking)
   - Speed up publish process for large workspaces

5. **Release Notes Generation:**
   - Auto-generate release notes from CHANGELOG
   - Create GitHub releases with notes

6. **Workspace Member Categories:**
   - Define categories (core, optional, examples, etc.)
   - Publish by category

### 10. Testing Strategy

**Unit Tests:**
- Test crate ordering logic
- Test argument parsing
- Test error handling

**Integration Tests:**
- Test dry-run mode (safe to run in CI)
- Mock cargo publish command
- Verify correct cargo commands are generated

**Manual Testing:**
- Test actual publish in a test registry
- Verify cross-platform behavior

## Summary

The xtask architecture provides:

1. **Cross-platform compatibility** - Works on Windows, macOS, and Linux
2. **No external dependencies** - Only requires cargo
3. **Type safety** - Rust's type system catches errors at compile time
4. **Extensibility** - Easy to add new features and commands
5. **Integration** - Works seamlessly with existing tooling (Justfile, CI/CD)
6. **Better UX** - Consistent CLI, better error messages, progress indicators

The design follows Rust and cargo conventions, integrates well with the existing tokmd project structure, and provides a solid foundation for future enhancements.
