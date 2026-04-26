//! Publish-specific deep tests (Wave 71).
//!
//! Covers: crate ordering in publish plan, rate limit detection (429),
//! dry run behavior, crate dependency resolution, and exclusion validation.

use std::collections::BTreeMap;
use std::path::PathBuf;
use std::process::Command;

use serde_json::Value;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn workspace_root() -> PathBuf {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    manifest_dir.parent().unwrap().to_path_buf()
}

fn run_xtask(args: &[&str]) -> (String, String, bool) {
    let root = workspace_root();
    let output = Command::new("cargo")
        .arg("run")
        .arg("-q")
        .arg("-p")
        .arg("xtask")
        .arg("--")
        .args(args)
        .current_dir(&root)
        .output()
        .expect("failed to run cargo xtask");
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    (stdout, stderr, output.status.success())
}

fn parse_publish_order(stdout: &str) -> Vec<String> {
    let mut result = Vec::new();
    let mut in_order = false;
    for line in stdout.lines() {
        if line.contains("Publish order") {
            in_order = true;
            continue;
        }
        if in_order {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with("Excluded") || trimmed.starts_with("Flags")
            {
                break;
            }
            if let Some(dot_pos) = trimmed.find(". ") {
                let name_part = &trimmed[dot_pos + 2..];
                let name = name_part.split_whitespace().next().unwrap_or("");
                if !name.is_empty() {
                    result.push(name.to_string());
                }
            }
        }
    }
    result
}

#[test]
fn publish_surface_json_distinguishes_current_and_target_surfaces() {
    let (stdout, stderr, success) = run_xtask(&["publish-surface", "--json"]);
    assert!(
        success,
        "publish-surface --json failed.\nstderr: {stderr}\nstdout: {stdout}"
    );

    let report: Value = serde_json::from_str(&stdout).expect("publish-surface JSON should parse");
    let summary = report
        .get("summary")
        .expect("publish-surface report should include summary");

    let current_public = summary["current_public_surface"]
        .as_array()
        .expect("current_public_surface should be an array");
    let current_support = summary["current_support_surface"]
        .as_array()
        .expect("current_support_surface should be an array");
    let target_support = summary["target_support_surface"]
        .as_array()
        .expect("target_support_surface should be an array");
    let target_gap = summary["target_gap"]
        .as_array()
        .expect("target_gap should be an array");
    let new_unapproved = summary["new_unapproved_support_crates"]
        .as_array()
        .expect("new_unapproved_support_crates should be an array");
    let public_product = summary["public_product_crates"]
        .as_array()
        .expect("public_product_crates should be an array");
    let public_contract = summary["public_contract_crates"]
        .as_array()
        .expect("public_contract_crates should be an array");
    let public_workflow = summary["public_workflow_crates"]
        .as_array()
        .expect("public_workflow_crates should be an array");
    let public_capability = summary["public_capability_crates"]
        .as_array()
        .expect("public_capability_crates should be an array");
    let conditional_public = summary["conditional_public_crates"]
        .as_array()
        .expect("conditional_public_crates should be an array");
    let internal_modules = summary["internal_module_families"]
        .as_array()
        .expect("internal_module_families should be an array");
    let dev_only = summary["dev_only_packages"]
        .as_array()
        .expect("dev_only_packages should be an array");
    let new_unclassified = summary["new_unclassified_packages"]
        .as_array()
        .expect("new_unclassified_packages should be an array");

    assert_eq!(current_public.len(), 12);
    assert_eq!(current_support.len(), 9);
    assert_eq!(target_support.len(), 9);
    assert!(target_gap.is_empty());
    assert!(new_unapproved.is_empty());
    assert_eq!(public_product.len(), 3);
    assert_eq!(public_contract.len(), 5);
    assert_eq!(public_workflow.len(), 3);
    assert_eq!(public_capability.len(), 5);
    assert_eq!(conditional_public.len(), 4);
    assert!(internal_modules.is_empty());
    assert_eq!(dev_only.len(), 1);
    assert!(new_unclassified.is_empty());
    assert_eq!(summary["public_surface"], summary["current_public_surface"]);
    assert_eq!(
        summary["support_surface"],
        summary["current_support_surface"]
    );
    assert_eq!(
        summary["non_crates_io_packages"],
        summary["current_non_crates_io_surface"]
    );
}

// ===========================================================================
// 1. Crate ordering in publish plan (dependency order)
// ===========================================================================

#[test]
fn publish_order_respects_tier_hierarchy() {
    let (stdout, _, _) = run_xtask(&["publish", "--plan"]);
    let order = parse_publish_order(&stdout);

    // Build a position map for quick lookup
    let pos: BTreeMap<&str, usize> = order
        .iter()
        .enumerate()
        .map(|(i, n)| (n.as_str(), i))
        .collect();

    // Tier 0 must come before tier 1
    if let (Some(&types_p), Some(&scan_p)) = (pos.get("tokmd-types"), pos.get("tokmd-scan")) {
        assert!(types_p < scan_p, "tier 0 (types) before tier 1 (scan)");
    }

    // Tier 1 must come before tier 2
    if let (Some(&scan_p), Some(&format_p)) = (pos.get("tokmd-scan"), pos.get("tokmd-format")) {
        assert!(scan_p < format_p, "tier 1 (scan) before tier 2 (format)");
    }

    // Tier 2 must come before tier 3
    if let (Some(&format_p), Some(&analysis_p)) =
        (pos.get("tokmd-format"), pos.get("tokmd-analysis"))
    {
        assert!(
            format_p < analysis_p,
            "tier 2 (format) before tier 3 (analysis)"
        );
    }

    // Tier 3 before tier 4
    if let (Some(&analysis_p), Some(&core_p)) = (pos.get("tokmd-analysis"), pos.get("tokmd-core")) {
        assert!(
            analysis_p < core_p,
            "tier 3 (analysis) before tier 4 (core)"
        );
    }

    // Tier 4 before tier 5
    if let (Some(&core_p), Some(&cli_p)) = (pos.get("tokmd-core"), pos.get("tokmd")) {
        assert!(core_p < cli_p, "tier 4 (core) before tier 5 (cli)");
    }
}

#[test]
fn publish_order_settings_before_dependents() {
    let (stdout, _, _) = run_xtask(&["publish", "--plan"]);
    let order = parse_publish_order(&stdout);
    let pos: BTreeMap<&str, usize> = order
        .iter()
        .enumerate()
        .map(|(i, n)| (n.as_str(), i))
        .collect();

    if let (Some(&settings_p), Some(&scan_p)) = (pos.get("tokmd-settings"), pos.get("tokmd-scan")) {
        assert!(
            settings_p < scan_p,
            "tokmd-settings must come before tokmd-scan"
        );
    }
}

#[test]
fn publish_order_every_crate_after_its_deps() {
    // Load actual workspace metadata to verify
    let root = workspace_root();
    let crates_dir = root.join("crates");

    let (stdout, _, _) = run_xtask(&["publish", "--plan"]);
    let order = parse_publish_order(&stdout);
    let pos: BTreeMap<&str, usize> = order
        .iter()
        .enumerate()
        .map(|(i, n)| (n.as_str(), i))
        .collect();

    // For each crate in the plan, check its Cargo.toml deps
    for (name, &idx) in &pos {
        let crate_dir = if *name == "tokmd" {
            crates_dir.join("tokmd")
        } else {
            crates_dir.join(name)
        };
        let cargo_toml = crate_dir.join("Cargo.toml");
        if !cargo_toml.exists() {
            continue;
        }
        let content = std::fs::read_to_string(&cargo_toml).unwrap();
        let table: toml::Table = toml::from_str(&content).unwrap();

        if let Some(toml::Value::Table(deps)) = table.get("dependencies") {
            for dep_name in deps.keys() {
                if let Some(&dep_idx) = pos.get(dep_name.as_str()) {
                    assert!(
                        dep_idx < idx,
                        "{name} (pos {idx}) depends on {dep_name} (pos {dep_idx}), \
                         but {dep_name} is later in publish order"
                    );
                }
            }
        }
    }
}

// ===========================================================================
// 2. Rate limit detection (429 codes)
// ===========================================================================

#[test]
fn publish_classify_rate_limit_status_429() {
    // Verify the source code handles "status 429" pattern
    let src = std::fs::read_to_string(workspace_root().join("xtask/src/tasks/publish.rs")).unwrap();
    assert!(
        src.contains("status 429"),
        "rate limit classifier should detect 'status 429'"
    );
}

#[test]
fn publish_classify_rate_limit_too_many_requests() {
    let src = std::fs::read_to_string(workspace_root().join("xtask/src/tasks/publish.rs")).unwrap();
    assert!(
        src.contains("too many requests"),
        "rate limit classifier should detect 'too many requests'"
    );
}

#[test]
fn publish_rate_limit_has_retry_after_parsing() {
    let src = std::fs::read_to_string(workspace_root().join("xtask/src/tasks/publish.rs")).unwrap();
    assert!(
        src.contains("parse_rate_limit_timestamp"),
        "should have retry-after timestamp parsing"
    );
    assert!(
        src.contains("try again after"),
        "should look for 'try again after' marker"
    );
}

#[test]
fn publish_rate_limit_has_max_waits() {
    let src = std::fs::read_to_string(workspace_root().join("xtask/src/tasks/publish.rs")).unwrap();
    assert!(
        src.contains("MAX_RATE_LIMIT_WAITS"),
        "should cap maximum rate limit wait iterations"
    );
}

#[test]
fn publish_rate_limit_timeout_cli_arg_exists() {
    let src = std::fs::read_to_string(workspace_root().join("xtask/src/cli.rs")).unwrap();
    assert!(
        src.contains("rate_limit_timeout"),
        "PublishArgs should have rate_limit_timeout field"
    );
}

// ===========================================================================
// 3. Dry run behavior
// ===========================================================================

#[test]
fn publish_dry_run_uses_cargo_package_list() {
    let src = std::fs::read_to_string(workspace_root().join("xtask/src/tasks/publish.rs")).unwrap();
    // Dry run should use `cargo package --list` not `cargo publish --dry-run`
    assert!(
        src.contains("cargo package") || src.contains(r#""package""#),
        "dry run should use cargo package for validation"
    );
    assert!(
        src.contains("--list") || src.contains(r#""--list""#),
        "dry run should use --list flag"
    );
}

#[test]
fn publish_preflight_excludes_xtask_on_windows() {
    let src = std::fs::read_to_string(workspace_root().join("xtask/src/tasks/publish.rs")).unwrap();
    assert!(
        src.contains("cfg!(windows)") && src.contains("\"xtask\""),
        "publish preflight should exclude xtask on Windows to avoid self-rebuild conflicts"
    );
}

#[test]
fn publish_plan_mode_does_not_execute() {
    // --plan should return quickly and not attempt actual publishing
    let start = std::time::Instant::now();
    let (stdout, _, success) = run_xtask(&["publish", "--plan"]);
    let elapsed = start.elapsed();

    assert!(success, "plan mode should succeed");
    assert!(
        stdout.contains("Publish Plan"),
        "should print publish plan header"
    );
    // Plan mode should be fast (< 60s even on slow CI)
    assert!(
        elapsed.as_secs() < 60,
        "plan mode should be fast, took {}s",
        elapsed.as_secs()
    );
}

#[test]
fn publish_plan_shows_execution_command() {
    let (stdout, _, success) = run_xtask(&["publish", "--plan"]);
    assert!(success);
    assert!(
        stdout.contains("To execute this plan"),
        "plan should show how to execute"
    );
    assert!(
        stdout.contains("cargo xtask publish"),
        "execution command should use cargo xtask publish"
    );
}

// ===========================================================================
// 4. Crate dependency resolution
// ===========================================================================

#[test]
fn publish_excludes_non_publishable_crates() {
    let (stdout, _, success) = run_xtask(&["publish", "--plan", "--verbose"]);
    assert!(success);
    let order = parse_publish_order(&stdout);

    // Non-publishable crates should not be in the plan
    let non_publishable = ["xtask", "tokmd-fuzz", "fuzz"];
    for name in &non_publishable {
        assert!(
            !order.iter().any(|n| n == name),
            "{name} should not be in publish plan"
        );
    }
}

#[test]
fn publish_all_planned_crates_are_workspace_members() {
    let root = workspace_root();
    let root_cargo = std::fs::read_to_string(root.join("Cargo.toml")).unwrap();

    let (stdout, _, _) = run_xtask(&["publish", "--plan"]);
    let order = parse_publish_order(&stdout);

    for name in &order {
        let member_pattern = format!("crates/{name}");
        assert!(
            root_cargo.contains(&member_pattern),
            "planned crate {name} should be a workspace member"
        );
    }
}

#[test]
fn publish_plan_consistent_crate_count() {
    let (stdout, _, _) = run_xtask(&["publish", "--plan"]);
    let order = parse_publish_order(&stdout);

    // Parse the declared count from "Publish order (N crates):"
    let declared: usize = stdout
        .lines()
        .find(|l| l.contains("crates):"))
        .and_then(|l| {
            l.split('(')
                .nth(1)
                .and_then(|s| s.split_whitespace().next())
                .and_then(|n| n.parse().ok())
        })
        .unwrap_or(0);

    assert_eq!(
        order.len(),
        declared,
        "parsed order length ({}) should match declared count ({declared})",
        order.len()
    );
}

#[test]
fn publish_plan_dependencies_form_dag() {
    // The publish plan should produce a valid topological ordering
    // (no crate appears before a crate it depends on)
    let (stdout, _, success) = run_xtask(&["publish", "--plan"]);
    assert!(success, "plan should succeed (implies no cycles)");

    let order = parse_publish_order(&stdout);
    // A successful plan output already proves the DAG is valid
    // (petgraph::toposort would fail on cycles)
    assert!(!order.is_empty(), "plan should produce a non-empty order");
}

#[test]
fn publish_plan_workspace_version_is_semver() {
    let (stdout, _, success) = run_xtask(&["publish", "--plan"]);
    assert!(success);

    let version_line = stdout
        .lines()
        .find(|l| l.contains("Workspace version:"))
        .expect("should show workspace version");

    let version = version_line
        .split(':')
        .nth(1)
        .map(|s| s.trim())
        .unwrap_or("");

    assert!(
        semver::Version::parse(version).is_ok(),
        "workspace version should be valid semver: {version}"
    );
}
