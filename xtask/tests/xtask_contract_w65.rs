//! Contract tests for xtask (Wave 65).
//!
//! Covers: publish plan ordering & invariants, boundaries check contracts,
//! docs validation, schema version extraction, gate steps, lint-fix structure,
//! CLI argument parsing, workspace metadata contracts, and BDD scenarios.

use std::collections::BTreeSet;
use std::path::PathBuf;
use std::process::Command;

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

fn read_root_cargo_toml() -> toml::Table {
    let content =
        std::fs::read_to_string(workspace_root().join("Cargo.toml")).unwrap();
    toml::from_str(&content).unwrap()
}

/// Extract workspace member paths from root Cargo.toml.
fn workspace_members() -> Vec<String> {
    let table = read_root_cargo_toml();
    table
        .get("workspace")
        .and_then(|w| w.get("members"))
        .and_then(|m| m.as_array())
        .unwrap()
        .iter()
        .filter_map(|v| v.as_str().map(|s| s.to_string()))
        .collect()
}

/// Parse the publish plan output and extract crate names in order.
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
            if trimmed.is_empty() || trimmed.starts_with("Excluded") || trimmed.starts_with("Flags") {
                break;
            }
            // Lines like "  1. tokmd-types"
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

// ===========================================================================
// 1. Publish plan — ordering contracts
// ===========================================================================

#[test]
fn publish_plan_succeeds() {
    let (stdout, stderr, success) = run_xtask(&["publish", "--plan"]);
    assert!(success, "publish --plan failed. stderr: {stderr}");
    assert!(
        stdout.contains("Publish order"),
        "should contain publish order: {stdout}"
    );
}

#[test]
fn publish_plan_types_before_scan() {
    let (stdout, _, _) = run_xtask(&["publish", "--plan"]);
    let order = parse_publish_order(&stdout);
    let types_pos = order.iter().position(|n| n == "tokmd-types");
    let scan_pos = order.iter().position(|n| n == "tokmd-scan");
    if let (Some(t), Some(s)) = (types_pos, scan_pos) {
        assert!(t < s, "tokmd-types (pos {t}) must come before tokmd-scan (pos {s})");
    }
}

#[test]
fn publish_plan_types_before_model() {
    let (stdout, _, _) = run_xtask(&["publish", "--plan"]);
    let order = parse_publish_order(&stdout);
    let types_pos = order.iter().position(|n| n == "tokmd-types");
    let model_pos = order.iter().position(|n| n == "tokmd-model");
    if let (Some(t), Some(m)) = (types_pos, model_pos) {
        assert!(t < m, "tokmd-types must come before tokmd-model");
    }
}

#[test]
fn publish_plan_scan_before_format() {
    let (stdout, _, _) = run_xtask(&["publish", "--plan"]);
    let order = parse_publish_order(&stdout);
    let scan_pos = order.iter().position(|n| n == "tokmd-scan");
    let format_pos = order.iter().position(|n| n == "tokmd-format");
    if let (Some(s), Some(f)) = (scan_pos, format_pos) {
        assert!(s < f, "tokmd-scan must come before tokmd-format");
    }
}

#[test]
fn publish_plan_core_before_cli() {
    let (stdout, _, _) = run_xtask(&["publish", "--plan"]);
    let order = parse_publish_order(&stdout);
    let core_pos = order.iter().position(|n| n == "tokmd-core");
    let cli_pos = order.iter().position(|n| n == "tokmd");
    if let (Some(c), Some(cli)) = (core_pos, cli_pos) {
        assert!(c < cli, "tokmd-core must come before tokmd (CLI)");
    }
}

#[test]
fn publish_plan_no_duplicates() {
    let (stdout, _, _) = run_xtask(&["publish", "--plan"]);
    let order = parse_publish_order(&stdout);
    let unique: BTreeSet<&str> = order.iter().map(|s| s.as_str()).collect();
    assert_eq!(
        order.len(),
        unique.len(),
        "publish plan should not have duplicates"
    );
}

#[test]
fn publish_plan_all_crates_start_with_tokmd() {
    let (stdout, _, _) = run_xtask(&["publish", "--plan"]);
    let order = parse_publish_order(&stdout);
    for name in &order {
        assert!(
            name.starts_with("tokmd"),
            "unexpected crate in publish plan: {name}"
        );
    }
}

#[test]
fn publish_plan_excludes_xtask() {
    let (stdout, _, _) = run_xtask(&["publish", "--plan"]);
    let order = parse_publish_order(&stdout);
    assert!(
        !order.contains(&"xtask".to_string()),
        "xtask should not be in publish plan"
    );
}

#[test]
fn publish_plan_excludes_fuzz() {
    let (stdout, _, _) = run_xtask(&["publish", "--plan"]);
    let order = parse_publish_order(&stdout);
    assert!(
        !order.iter().any(|n| n == "tokmd-fuzz" || n == "fuzz"),
        "fuzz crate should not be in publish plan"
    );
}

#[test]
fn publish_plan_crate_count_positive() {
    let (stdout, _, _) = run_xtask(&["publish", "--plan"]);
    let order = parse_publish_order(&stdout);
    assert!(!order.is_empty(), "publish plan must include at least one crate");
}

#[test]
fn publish_plan_verbose_lists_exclusion_reasons() {
    let (stdout, _, success) = run_xtask(&["publish", "--plan", "--verbose"]);
    assert!(success);
    assert!(
        stdout.contains("Excluded crates"),
        "verbose plan should show excluded crates"
    );
    assert!(
        stdout.contains("IsXtask") || stdout.contains("xtask"),
        "should mention xtask exclusion"
    );
}

#[test]
fn publish_plan_shows_workspace_version() {
    let (stdout, _, success) = run_xtask(&["publish", "--plan"]);
    assert!(success);
    assert!(stdout.contains("Workspace version:"));
}

#[test]
fn publish_plan_shows_execution_command() {
    let (stdout, _, success) = run_xtask(&["publish", "--plan"]);
    assert!(success);
    assert!(
        stdout.contains("To execute this plan"),
        "should show execution command"
    );
}

// ===========================================================================
// 2. Boundaries check contracts
// ===========================================================================

#[test]
fn boundaries_check_passes_on_clean_repo() {
    let (stdout, stderr, success) = run_xtask(&["boundaries-check"]);
    assert!(success, "boundaries-check should pass. stderr: {stderr}");
    assert!(stdout.contains("OK"));
}

#[test]
fn boundaries_analysis_crates_no_config_dep() {
    let root = workspace_root();
    let crates_dir = root.join("crates");
    let mut checked = 0u32;

    for entry in std::fs::read_dir(&crates_dir)
        .unwrap()
        .filter_map(|e| e.ok())
    {
        let name = entry.file_name().to_string_lossy().to_string();
        if !name.starts_with("tokmd-analysis") {
            continue;
        }
        let cargo_toml = entry.path().join("Cargo.toml");
        if !cargo_toml.exists() {
            continue;
        }
        let content = std::fs::read_to_string(&cargo_toml).unwrap();
        let table: toml::Table = toml::from_str(&content).unwrap();

        for dep_table in &["dependencies", "dev-dependencies", "build-dependencies"] {
            if let Some(toml::Value::Table(deps)) = table.get(*dep_table) {
                assert!(
                    !deps.contains_key("tokmd-config"),
                    "{name} must not depend on tokmd-config in [{dep_table}]"
                );
            }
        }
        checked += 1;
    }
    assert!(checked > 0, "should check at least one analysis crate");
}

#[test]
fn boundaries_forbidden_list_in_source() {
    let src = std::fs::read_to_string(
        workspace_root()
            .join("xtask")
            .join("src")
            .join("tasks")
            .join("boundaries_check.rs"),
    )
    .unwrap();
    assert!(
        src.contains("\"tokmd-config\""),
        "FORBIDDEN list must include tokmd-config"
    );
}

#[test]
fn boundaries_all_analysis_crates_are_workspace_members() {
    let root = workspace_root();
    let root_cargo = std::fs::read_to_string(root.join("Cargo.toml")).unwrap();
    let crates_dir = root.join("crates");

    for entry in std::fs::read_dir(&crates_dir)
        .unwrap()
        .filter_map(|e| e.ok())
    {
        let name = entry.file_name().to_string_lossy().to_string();
        if !name.starts_with("tokmd-analysis") || !entry.path().is_dir() {
            continue;
        }
        let member = format!("crates/{name}");
        assert!(
            root_cargo.contains(&member),
            "analysis crate {name} missing from workspace members"
        );
    }
}

#[test]
fn boundaries_analysis_crates_each_have_package_name() {
    let crates_dir = workspace_root().join("crates");
    for entry in std::fs::read_dir(&crates_dir)
        .unwrap()
        .filter_map(|e| e.ok())
    {
        let name = entry.file_name().to_string_lossy().to_string();
        if !name.starts_with("tokmd-analysis") {
            continue;
        }
        let cargo_toml = entry.path().join("Cargo.toml");
        if !cargo_toml.exists() {
            continue;
        }
        let content = std::fs::read_to_string(&cargo_toml).unwrap();
        let table: toml::Table = toml::from_str(&content).unwrap();
        let pkg_name = table
            .get("package")
            .and_then(|p| p.get("name"))
            .and_then(|n| n.as_str());
        assert!(pkg_name.is_some(), "{name}/Cargo.toml must have [package].name");
    }
}

// ===========================================================================
// 3. Docs check contracts
// ===========================================================================

#[test]
fn docs_reference_cli_exists() {
    let path = workspace_root().join("docs").join("reference-cli.md");
    assert!(path.exists(), "docs/reference-cli.md must exist");
}

#[test]
fn docs_reference_cli_has_paired_markers() {
    let path = workspace_root().join("docs").join("reference-cli.md");
    let content = std::fs::read_to_string(&path).unwrap();
    let mut open_markers = Vec::new();

    for line in content.lines() {
        if let Some(start) = line.find("<!-- HELP: ") {
            if line.contains("<!-- /HELP:") {
                continue;
            }
            let after = &line[start + 11..];
            if let Some(end) = after.find(" -->") {
                open_markers.push(after[..end].to_string());
            }
        }
    }
    for cmd in &open_markers {
        let close = format!("<!-- /HELP: {cmd} -->");
        assert!(
            content.contains(&close),
            "missing close marker for {cmd}"
        );
    }
    assert!(!open_markers.is_empty(), "should have at least one marker pair");
}

#[test]
fn docs_check_succeeds_or_reports_drift() {
    let (stdout, stderr, success) = run_xtask(&["docs", "--check"]);
    if success {
        assert!(stdout.contains("up to date"));
    } else {
        assert!(
            stderr.contains("drift") || stderr.contains("Documentation drift"),
            "failure should mention drift: {stderr}"
        );
    }
}

#[test]
fn docs_without_flags_reports_status() {
    let (stdout, _stderr, success) = run_xtask(&["docs"]);
    assert!(success, "docs without flags should succeed");
    assert!(
        stdout.contains("up to date") || stdout.contains("Updated"),
        "should report documentation status"
    );
}

// ===========================================================================
// 4. Schema version extraction contracts
// ===========================================================================

fn read_schema_constant(relative_path: &str, constant_name: &str) -> Option<u32> {
    let path = workspace_root().join(relative_path);
    let content = std::fs::read_to_string(&path).ok()?;
    let pattern = format!("pub const {constant_name}: u32 = ");
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with(&pattern) {
            let after = &trimmed[pattern.len()..];
            let num_str = after.trim_end_matches(';').trim();
            return num_str.parse().ok();
        }
    }
    None
}

#[test]
fn schema_version_core_exists() {
    let v = read_schema_constant("crates/tokmd-types/src/lib.rs", "SCHEMA_VERSION");
    assert!(v.is_some(), "SCHEMA_VERSION must exist in tokmd-types");
    assert!(v.unwrap() >= 1, "SCHEMA_VERSION must be >= 1");
}

#[test]
fn schema_version_analysis_exists() {
    let v = read_schema_constant(
        "crates/tokmd-analysis-types/src/lib.rs",
        "ANALYSIS_SCHEMA_VERSION",
    );
    assert!(v.is_some(), "ANALYSIS_SCHEMA_VERSION must exist");
    assert!(v.unwrap() >= 1);
}

#[test]
fn schema_version_cockpit_exists() {
    let v = read_schema_constant(
        "crates/tokmd-types/src/cockpit.rs",
        "COCKPIT_SCHEMA_VERSION",
    );
    assert!(v.is_some(), "COCKPIT_SCHEMA_VERSION must exist");
    assert!(v.unwrap() >= 1);
}

#[test]
fn schema_version_context_exists() {
    let v = read_schema_constant("crates/tokmd-types/src/lib.rs", "CONTEXT_SCHEMA_VERSION");
    assert!(v.is_some(), "CONTEXT_SCHEMA_VERSION must exist");
    assert!(v.unwrap() >= 1);
}

#[test]
fn schema_version_handoff_exists() {
    let v = read_schema_constant("crates/tokmd-types/src/lib.rs", "HANDOFF_SCHEMA_VERSION");
    assert!(v.is_some(), "HANDOFF_SCHEMA_VERSION must exist");
    assert!(v.unwrap() >= 1);
}

#[test]
fn bump_dry_run_validates_known_schema_constants() {
    let (stdout, _, success) = run_xtask(&[
        "bump",
        "99.99.99",
        "--dry-run",
        "--schema",
        "SCHEMA_VERSION=99",
    ]);
    assert!(success, "bump --dry-run with schema should succeed");
    assert!(stdout.contains("SCHEMA_VERSION"));
    assert!(stdout.contains("99"));
}

#[test]
fn bump_dry_run_rejects_unknown_schema() {
    let (_, stderr, success) = run_xtask(&[
        "bump",
        "99.99.99",
        "--dry-run",
        "--schema",
        "FAKE_VERSION=1",
    ]);
    assert!(!success, "unknown schema constant should fail");
    assert!(
        stderr.contains("Unknown schema constant") || stderr.contains("FAKE_VERSION"),
        "should mention unknown constant"
    );
}

// ===========================================================================
// 5. Gate step contracts
// ===========================================================================

#[test]
fn gate_source_defines_all_expected_steps() {
    let src = std::fs::read_to_string(
        workspace_root()
            .join("xtask")
            .join("src")
            .join("tasks")
            .join("gate.rs"),
    )
    .unwrap();
    // The gate task defines STEPS with label fields
    assert!(src.contains("\"fmt\""), "gate should have fmt step");
    assert!(src.contains("\"clippy\""), "gate should have clippy step");
    assert!(
        src.contains("\"test"),
        "gate should have test step"
    );
}

#[test]
fn gate_check_mode_uses_check_args_for_fmt() {
    let src = std::fs::read_to_string(
        workspace_root()
            .join("xtask")
            .join("src")
            .join("tasks")
            .join("gate.rs"),
    )
    .unwrap();
    assert!(
        src.contains("--check"),
        "gate fmt step should have --check variant"
    );
}

#[test]
fn gate_excludes_python_crate() {
    let src = std::fs::read_to_string(
        workspace_root()
            .join("xtask")
            .join("src")
            .join("tasks")
            .join("gate.rs"),
    )
    .unwrap();
    assert!(
        src.contains("tokmd-python"),
        "gate should exclude tokmd-python from checks"
    );
}

// ===========================================================================
// 6. CLI argument parsing contracts
// ===========================================================================

#[test]
fn cli_help_succeeds_and_mentions_tasks() {
    let output = Command::new("cargo")
        .args(["run", "-q", "-p", "xtask", "--", "--help"])
        .current_dir(workspace_root())
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Development tasks for tokmd"));
}

#[test]
fn cli_bump_requires_version() {
    let (_, stderr, success) = run_xtask(&["bump"]);
    assert!(!success);
    assert!(stderr.contains("required") || stderr.contains("Usage"));
}

#[test]
fn cli_publish_plan_flag_accepted() {
    let (_, _, success) = run_xtask(&["publish", "--plan"]);
    assert!(success, "--plan flag should be accepted");
}

#[test]
fn cli_unknown_subcommand_fails() {
    let (_, stderr, success) = run_xtask(&["nonexistent-command-xyz"]);
    assert!(!success);
    assert!(
        stderr.contains("error") || stderr.contains("unrecognized"),
        "should show error for unknown subcommand"
    );
}

#[test]
fn cli_gate_help_shows_check_flag() {
    let output = Command::new("cargo")
        .args(["run", "-q", "-p", "xtask", "--", "gate", "--help"])
        .current_dir(workspace_root())
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("--check"));
}

#[test]
fn cli_lint_fix_help_shows_flags() {
    let output = Command::new("cargo")
        .args(["run", "-q", "-p", "xtask", "--", "lint-fix", "--help"])
        .current_dir(workspace_root())
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("--check"));
    assert!(stdout.contains("--no-clippy"));
}

// ===========================================================================
// 7. Workspace metadata contracts
// ===========================================================================

#[test]
fn workspace_has_members() {
    let members = workspace_members();
    assert!(!members.is_empty(), "workspace should have members");
}

#[test]
fn workspace_includes_xtask() {
    let members = workspace_members();
    assert!(
        members.iter().any(|m| m.contains("xtask")),
        "workspace should include xtask"
    );
}

#[test]
fn workspace_includes_tokmd_types() {
    let members = workspace_members();
    assert!(
        members.iter().any(|m| m.contains("tokmd-types")),
        "workspace should include tokmd-types"
    );
}

#[test]
fn workspace_root_cargo_toml_has_workspace_version() {
    let table = read_root_cargo_toml();
    let version = table
        .get("workspace")
        .and_then(|w| w.get("package"))
        .and_then(|p| p.get("version"))
        .and_then(|v| v.as_str());
    assert!(version.is_some(), "workspace should have a version");
    assert!(!version.unwrap().is_empty());
}

#[test]
fn workspace_all_member_dirs_exist() {
    let root = workspace_root();
    let members = workspace_members();
    for member in &members {
        let path = root.join(member);
        assert!(
            path.exists(),
            "workspace member dir does not exist: {member}"
        );
    }
}

// ===========================================================================
// 8. Publish plan — dependency validation contracts
// ===========================================================================

#[test]
fn publish_plan_types_before_dependents() {
    let (stdout, _, _) = run_xtask(&["publish", "--plan"]);
    let order = parse_publish_order(&stdout);
    // tokmd-types is tier-0 and a direct dependency of scan, model, format
    let types_pos = order.iter().position(|n| n == "tokmd-types");
    // Only check crates that actually depend on tokmd-types
    let dependents = ["tokmd-scan", "tokmd-model", "tokmd-format"];
    for dep in &dependents {
        let dep_pos = order.iter().position(|n| n == *dep);
        if let (Some(t), Some(d)) = (types_pos, dep_pos) {
            assert!(
                t < d,
                "tokmd-types (pos {t}) must come before {dep} (pos {d})"
            );
        }
    }
}

#[test]
fn publish_plan_cli_is_last_or_near_last() {
    let (stdout, _, _) = run_xtask(&["publish", "--plan"]);
    let order = parse_publish_order(&stdout);
    let cli_pos = order.iter().position(|n| n == "tokmd");
    if let Some(pos) = cli_pos {
        let tail_threshold = order.len().saturating_sub(5);
        assert!(
            pos >= tail_threshold,
            "tokmd CLI should be near the end of publish order (pos={pos}, len={})",
            order.len()
        );
    }
}

// ===========================================================================
// 9. BDD-style scenarios
// ===========================================================================

/// Given a clean workspace
/// When running boundaries-check
/// Then all analysis crates pass boundary validation.
#[test]
fn bdd_given_clean_workspace_when_boundaries_check_then_passes() {
    let (stdout, stderr, success) = run_xtask(&["boundaries-check"]);
    assert!(success, "stderr: {stderr}");
    assert!(stdout.contains("OK"));
}

/// Given the publish task
/// When running with --plan flag
/// Then it shows ordered crates and does not attempt actual publishing.
#[test]
fn bdd_given_publish_when_plan_then_shows_order_no_publish() {
    let (stdout, _, success) = run_xtask(&["publish", "--plan"]);
    assert!(success);
    assert!(stdout.contains("Publish order"));
    assert!(stdout.contains("Flags:"));
    assert!(!stdout.contains("Publishing"));
}

/// Given the bump task
/// When running with an invalid version string
/// Then it exits with an error mentioning semver format.
#[test]
fn bdd_given_bump_when_invalid_version_then_error() {
    let (_, stderr, success) = run_xtask(&["bump", "invalid", "--dry-run"]);
    assert!(!success);
    assert!(
        stderr.contains("semver") || stderr.contains("MAJOR.MINOR.PATCH"),
        "error should mention semver: {stderr}"
    );
}

/// Given the bump task
/// When running with --dry-run and valid version
/// Then it shows planned changes without modifying files.
#[test]
fn bdd_given_bump_when_dry_run_then_shows_plan() {
    let (stdout, _, success) = run_xtask(&["bump", "99.99.99", "--dry-run"]);
    assert!(success);
    assert!(stdout.contains("DRY RUN"));
    assert!(stdout.contains("Planned changes"));
    assert!(stdout.contains("99.99.99"));
}

/// Given a publish plan with verbose mode
/// When checking for non-publishable crate handling
/// Then excluded crates are listed with reasons.
#[test]
fn bdd_given_verbose_plan_when_checking_exclusions_then_reasons_shown() {
    let (stdout, _, success) = run_xtask(&["publish", "--plan", "--verbose"]);
    assert!(success);
    assert!(stdout.contains("Excluded crates"));
    // Should mention xtask in the excluded crates section
    assert!(
        stdout.contains("xtask"),
        "should list xtask in excluded crates"
    );
}
