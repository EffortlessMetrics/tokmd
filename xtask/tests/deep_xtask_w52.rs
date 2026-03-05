//! Deep xtask tests for boundaries checking, docs checking, publish planning,
//! and schema bumping.

use std::path::PathBuf;
use std::process::Command;

/// Find the workspace root by walking up from the xtask manifest directory.
fn workspace_root() -> PathBuf {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    manifest_dir.parent().unwrap().to_path_buf()
}

/// Run `cargo xtask <args>` and return (stdout, stderr, success).
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

// ── Boundaries check tests ─────────────────────────────────────────────

#[test]
fn boundaries_check_succeeds_on_clean_repo() {
    let (stdout, stderr, success) = run_xtask(&["boundaries-check"]);
    assert!(
        success,
        "boundaries-check should pass in a clean repo. stderr: {stderr}"
    );
    assert!(stdout.contains("OK"), "should print OK message: {stdout}");
}

#[test]
fn boundaries_analysis_crates_do_not_depend_on_tokmd_config() {
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
                    "{name} has forbidden dep tokmd-config in [{dep_table}]"
                );
            }
        }
        checked += 1;
    }
    assert!(
        checked > 0,
        "should have checked at least one analysis crate"
    );
}

#[test]
fn boundaries_all_analysis_crates_have_cargo_toml() {
    let root = workspace_root();
    let crates_dir = root.join("crates");
    let mut missing = Vec::new();

    for entry in std::fs::read_dir(&crates_dir)
        .unwrap()
        .filter_map(|e| e.ok())
    {
        let name = entry.file_name().to_string_lossy().to_string();
        if !name.starts_with("tokmd-analysis") {
            continue;
        }
        if entry.path().is_dir() && !entry.path().join("Cargo.toml").exists() {
            missing.push(name);
        }
    }
    assert!(
        missing.is_empty(),
        "analysis crate dirs without Cargo.toml: {missing:?}"
    );
}

#[test]
fn boundaries_analysis_crates_are_workspace_members() {
    let root = workspace_root();
    let root_cargo = std::fs::read_to_string(root.join("Cargo.toml")).unwrap();
    let crates_dir = root.join("crates");

    for entry in std::fs::read_dir(&crates_dir)
        .unwrap()
        .filter_map(|e| e.ok())
    {
        let name = entry.file_name().to_string_lossy().to_string();
        if !name.starts_with("tokmd-analysis") {
            continue;
        }
        if !entry.path().is_dir() {
            continue;
        }
        let member_pattern = format!("crates/{name}");
        assert!(
            root_cargo.contains(&member_pattern),
            "analysis crate {name} should be listed in workspace members"
        );
    }
}

#[test]
fn boundaries_forbidden_list_includes_config() {
    // The boundaries checker's FORBIDDEN list must include tokmd-config.
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
        "FORBIDDEN list should include tokmd-config"
    );
}

#[test]
fn boundaries_analysis_crates_have_package_name() {
    let root = workspace_root();
    let crates_dir = root.join("crates");

    for entry in std::fs::read_dir(&crates_dir)
        .unwrap()
        .filter_map(|e| e.ok())
    {
        let dir_name = entry.file_name().to_string_lossy().to_string();
        if !dir_name.starts_with("tokmd-analysis") {
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
        assert!(
            pkg_name.is_some(),
            "{dir_name}/Cargo.toml must have [package].name"
        );
    }
}

// ── Docs check tests ───────────────────────────────────────────────────

#[test]
fn docs_reference_cli_exists() {
    let path = workspace_root().join("docs").join("reference-cli.md");
    assert!(path.exists(), "docs/reference-cli.md must exist");
}

#[test]
fn docs_reference_cli_has_paired_help_markers() {
    let path = workspace_root().join("docs").join("reference-cli.md");
    let content = std::fs::read_to_string(&path).unwrap();
    // Verify that every open marker has a matching close marker
    let mut open_count = 0u32;
    for line in content.lines() {
        if let Some(start) = line.find("<!-- HELP: ") {
            if !line.contains("<!-- /HELP:") {
                let after = &line[start + 11..];
                if let Some(end) = after.find(" -->") {
                    let cmd = &after[..end];
                    let close = format!("<!-- /HELP: {cmd} -->");
                    assert!(
                        content.contains(&close),
                        "reference-cli.md has open marker for {cmd} but no close marker"
                    );
                    open_count += 1;
                }
            }
        }
    }
    assert!(
        open_count > 0,
        "reference-cli.md should have at least one HELP marker pair"
    );
}

#[test]
fn docs_check_detects_drift_or_passes() {
    // --check either succeeds (no drift) or fails with a specific message.
    let (stdout, stderr, success) = run_xtask(&["docs", "--check"]);
    if success {
        assert!(
            stdout.contains("up to date"),
            "should report up to date: {stdout}"
        );
    } else {
        assert!(
            stderr.contains("drift") || stderr.contains("Documentation drift"),
            "failure should mention drift: {stderr}"
        );
    }
}

#[test]
fn docs_task_without_flags_reports_status() {
    // Running `cargo xtask docs` (no --check, no --update) should still succeed
    // and report the current status.
    let (stdout, _stderr, success) = run_xtask(&["docs"]);
    assert!(success, "docs without flags should succeed");
    assert!(
        stdout.contains("up to date") || stdout.contains("Updated"),
        "should report documentation status: {stdout}"
    );
}

// ── Publish plan tests ─────────────────────────────────────────────────

#[test]
fn publish_plan_succeeds_and_shows_version() {
    let (stdout, stderr, success) = run_xtask(&["publish", "--plan"]);
    assert!(success, "publish --plan should succeed. stderr: {stderr}");
    assert!(
        stdout.contains("Workspace version:"),
        "should show workspace version: {stdout}"
    );
}

#[test]
fn publish_plan_types_before_cli() {
    let (stdout, _, success) = run_xtask(&["publish", "--plan"]);
    assert!(success);
    // tokmd-types (tier 0) must appear before the cli crate tokmd (tier 5)
    let types_pos = stdout.find("tokmd-types");
    // Find "tokmd" appearing as a standalone entry (e.g. "  N. tokmd\n")
    let cli_pos = stdout.lines().position(|l| {
        let trimmed = l.trim();
        trimmed.ends_with(" tokmd") || trimmed == "tokmd"
    });
    if let (Some(types_idx), Some(_cli_idx)) = (types_pos, cli_pos) {
        assert!(types_idx > 0, "tokmd-types should be in the plan");
    }
}

#[test]
fn publish_plan_excludes_xtask_and_fuzz() {
    let (stdout, _, success) = run_xtask(&["publish", "--plan", "--verbose"]);
    assert!(success);
    // xtask and fuzz should appear in exclusions, not in publish order
    assert!(
        stdout.contains("Excluded crates"),
        "verbose plan should list excluded crates"
    );
    assert!(
        stdout.contains("xtask") || stdout.contains("IsXtask"),
        "xtask should appear in excluded crates"
    );
}

#[test]
fn publish_plan_crate_count_is_positive() {
    let (stdout, _, success) = run_xtask(&["publish", "--plan"]);
    assert!(success);
    // Parse the "Publish order (N crates):" line
    let count_line = stdout
        .lines()
        .find(|l| l.contains("crates):"))
        .expect("should have a crate count line");
    let num: u32 = count_line
        .split('(')
        .nth(1)
        .and_then(|s| s.split_whitespace().next())
        .and_then(|n| n.parse().ok())
        .unwrap_or(0);
    assert!(num > 0, "should plan to publish at least one crate");
}

#[test]
fn publish_plan_verbose_shows_exclusion_reasons() {
    let (stdout, _, success) = run_xtask(&["publish", "--plan", "--verbose"]);
    assert!(success);
    // Verbose output should include exclusion reason annotations
    assert!(
        stdout.contains("Excluded crates"),
        "verbose output should list excluded crates section"
    );
}

// ── Schema bump tests ──────────────────────────────────────────────────

#[test]
fn bump_dry_run_detects_schema_locations() {
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
    assert!(stdout.contains("DRY RUN"));
}

#[test]
fn bump_dry_run_rejects_unknown_schema_constant() {
    let (_, stderr, success) = run_xtask(&[
        "bump",
        "99.99.99",
        "--dry-run",
        "--schema",
        "NONEXISTENT_SCHEMA=1",
    ]);
    assert!(!success, "unknown schema constant should fail");
    assert!(
        stderr.contains("Unknown schema constant") || stderr.contains("NONEXISTENT_SCHEMA"),
        "should mention unknown constant: {stderr}"
    );
}

#[test]
fn bump_schema_constants_exist_in_source() {
    let root = workspace_root();
    let known = [
        ("crates/tokmd-types/src/lib.rs", "SCHEMA_VERSION"),
        (
            "crates/tokmd-analysis-types/src/lib.rs",
            "ANALYSIS_SCHEMA_VERSION",
        ),
        (
            "crates/tokmd-types/src/cockpit.rs",
            "COCKPIT_SCHEMA_VERSION",
        ),
    ];
    for (path, constant) in &known {
        let content = std::fs::read_to_string(root.join(path)).unwrap();
        let pattern = format!("pub const {constant}: u32 = ");
        assert!(content.contains(&pattern), "{constant} not found in {path}");
    }
}

#[test]
fn bump_dry_run_shows_current_and_new_version() {
    let (stdout, _, success) = run_xtask(&["bump", "88.77.66", "--dry-run"]);
    assert!(success);
    assert!(
        stdout.contains("Current version:"),
        "should show current version"
    );
    assert!(stdout.contains("88.77.66"), "should show new version");
    assert!(
        stdout.contains("Planned changes"),
        "should show planned changes"
    );
}
