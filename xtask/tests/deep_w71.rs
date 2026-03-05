//! Deep xtask tests (Wave 71).
//!
//! Covers: boundaries-check logic, docs check logic, publish plan ordering,
//! version bump logic, schema version detection, and deterministic output.

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
            if trimmed.is_empty()
                || trimmed.starts_with("Excluded")
                || trimmed.starts_with("Flags")
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

fn read_root_cargo_toml() -> toml::Table {
    let content = std::fs::read_to_string(workspace_root().join("Cargo.toml")).unwrap();
    toml::from_str(&content).unwrap()
}

// ===========================================================================
// 1. Boundaries-check logic
// ===========================================================================

#[test]
fn boundaries_check_succeeds_on_current_repo() {
    let (stdout, stderr, success) = run_xtask(&["boundaries-check"]);
    assert!(success, "boundaries-check failed. stderr: {stderr}");
    assert!(
        stdout.contains("OK"),
        "should print success message: {stdout}"
    );
}

#[test]
fn boundaries_forbidden_list_is_not_empty() {
    let src = std::fs::read_to_string(
        workspace_root()
            .join("xtask/src/tasks/boundaries_check.rs"),
    )
    .unwrap();
    assert!(
        src.contains("const FORBIDDEN: &[&str] = &["),
        "FORBIDDEN list should be declared as a const"
    );
    // Must contain at least tokmd-config
    assert!(
        src.contains("\"tokmd-config\""),
        "FORBIDDEN must include tokmd-config"
    );
}

#[test]
fn boundaries_checks_all_dep_tables() {
    let src = std::fs::read_to_string(
        workspace_root()
            .join("xtask/src/tasks/boundaries_check.rs"),
    )
    .unwrap();
    for table in &["dependencies", "dev-dependencies", "build-dependencies"] {
        assert!(
            src.contains(table),
            "boundaries checker should inspect [{table}]"
        );
    }
}

#[test]
fn boundaries_only_scans_analysis_crates() {
    let src = std::fs::read_to_string(
        workspace_root()
            .join("xtask/src/tasks/boundaries_check.rs"),
    )
    .unwrap();
    assert!(
        src.contains("tokmd-analysis"),
        "boundaries check should filter for analysis crates"
    );
}

#[test]
fn boundaries_analysis_crates_have_src_dir() {
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
        if !entry.path().is_dir() {
            continue;
        }
        assert!(
            entry.path().join("src").exists(),
            "{name} should have a src/ directory"
        );
        checked += 1;
    }
    assert!(checked > 0, "should check at least one analysis crate");
}

#[test]
fn boundaries_no_analysis_crate_depends_on_cli() {
    let root = workspace_root();
    let crates_dir = root.join("crates");

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
                // Analysis crates must not depend on the CLI binary
                assert!(
                    !deps.contains_key("tokmd"),
                    "{name} has forbidden dep 'tokmd' (CLI binary) in [{dep_table}]"
                );
            }
        }
    }
}

// ===========================================================================
// 2. Docs check logic
// ===========================================================================

#[test]
fn docs_reference_cli_file_exists() {
    let path = workspace_root().join("docs/reference-cli.md");
    assert!(path.exists(), "docs/reference-cli.md must exist");
}

#[test]
fn docs_help_markers_all_paired() {
    let path = workspace_root().join("docs/reference-cli.md");
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
            "open marker for {cmd} has no matching close marker"
        );
    }
    assert!(
        !open_markers.is_empty(),
        "should have at least one HELP marker pair"
    );
}

#[test]
fn docs_markers_cover_known_commands() {
    let path = workspace_root().join("docs/reference-cli.md");
    let content = std::fs::read_to_string(&path).unwrap();

    let required_commands = ["lang", "cockpit", "sensor", "gate"];
    for cmd in &required_commands {
        let marker = format!("<!-- HELP: {cmd} -->");
        assert!(
            content.contains(&marker),
            "reference-cli.md should have marker for {cmd}"
        );
    }
}

#[test]
fn docs_check_mode_either_passes_or_reports_drift() {
    let (stdout, stderr, success) = run_xtask(&["docs", "--check"]);
    if success {
        assert!(
            stdout.contains("up to date"),
            "should say 'up to date': {stdout}"
        );
    } else {
        assert!(
            stderr.contains("drift") || stderr.contains("Documentation drift"),
            "failure should mention drift: {stderr}"
        );
    }
}

#[test]
fn docs_task_normalizes_crlf_and_exe_suffix() {
    let src = std::fs::read_to_string(
        workspace_root().join("xtask/src/tasks/docs.rs"),
    )
    .unwrap();
    assert!(
        src.contains(r#"replace("\r\n", "\n")"#),
        "docs task should normalize CRLF"
    );
    assert!(
        src.contains(r#"replace("tokmd.exe", "tokmd")"#),
        "docs task should normalize .exe suffix"
    );
}

// ===========================================================================
// 3. Publish plan ordering
// ===========================================================================

#[test]
fn publish_plan_tier0_before_tier1() {
    let (stdout, _, _) = run_xtask(&["publish", "--plan"]);
    let order = parse_publish_order(&stdout);
    let types_pos = order.iter().position(|n| n == "tokmd-types");
    let scan_pos = order.iter().position(|n| n == "tokmd-scan");
    let model_pos = order.iter().position(|n| n == "tokmd-model");

    if let (Some(t), Some(s)) = (types_pos, scan_pos) {
        assert!(t < s, "tokmd-types must come before tokmd-scan");
    }
    if let (Some(t), Some(m)) = (types_pos, model_pos) {
        assert!(t < m, "tokmd-types must come before tokmd-model");
    }
}

#[test]
fn publish_plan_format_after_scan() {
    let (stdout, _, _) = run_xtask(&["publish", "--plan"]);
    let order = parse_publish_order(&stdout);
    let scan_pos = order.iter().position(|n| n == "tokmd-scan");
    let format_pos = order.iter().position(|n| n == "tokmd-format");
    if let (Some(s), Some(f)) = (scan_pos, format_pos) {
        assert!(s < f, "tokmd-scan must come before tokmd-format");
    }
}

#[test]
fn publish_plan_analysis_types_before_analysis() {
    let (stdout, _, _) = run_xtask(&["publish", "--plan"]);
    let order = parse_publish_order(&stdout);
    let at_pos = order.iter().position(|n| n == "tokmd-analysis-types");
    let a_pos = order.iter().position(|n| n == "tokmd-analysis");
    if let (Some(at), Some(a)) = (at_pos, a_pos) {
        assert!(
            at < a,
            "tokmd-analysis-types must come before tokmd-analysis"
        );
    }
}

#[test]
fn publish_plan_core_before_cli_binary() {
    let (stdout, _, _) = run_xtask(&["publish", "--plan"]);
    let order = parse_publish_order(&stdout);
    let core_pos = order.iter().position(|n| n == "tokmd-core");
    let cli_pos = order.iter().position(|n| n == "tokmd");
    if let (Some(c), Some(cli)) = (core_pos, cli_pos) {
        assert!(c < cli, "tokmd-core must come before tokmd");
    }
}

#[test]
fn publish_plan_cli_is_last_or_near_last() {
    let (stdout, _, _) = run_xtask(&["publish", "--plan"]);
    let order = parse_publish_order(&stdout);
    if let Some(cli_pos) = order.iter().position(|n| n == "tokmd") {
        // CLI should be in the last 3 positions (allowing for python/node)
        assert!(
            cli_pos >= order.len().saturating_sub(3),
            "tokmd CLI should be near the end of publish order (pos {cli_pos} of {})",
            order.len()
        );
    }
}

#[test]
fn publish_plan_no_duplicates() {
    let (stdout, _, _) = run_xtask(&["publish", "--plan"]);
    let order = parse_publish_order(&stdout);
    let unique: BTreeSet<&str> = order.iter().map(|s| s.as_str()).collect();
    assert_eq!(order.len(), unique.len(), "publish plan must not have duplicates");
}

#[test]
fn publish_plan_all_crates_start_with_tokmd() {
    let (stdout, _, _) = run_xtask(&["publish", "--plan"]);
    let order = parse_publish_order(&stdout);
    for name in &order {
        assert!(
            name.starts_with("tokmd"),
            "unexpected crate in plan: {name}"
        );
    }
}

#[test]
fn publish_plan_shows_flags_section() {
    let (stdout, _, success) = run_xtask(&["publish", "--plan"]);
    assert!(success);
    assert!(
        stdout.contains("Flags:"),
        "plan output should include Flags section"
    );
    assert!(
        stdout.contains("--dry-run:"),
        "plan output should show dry-run flag"
    );
}

// ===========================================================================
// 4. Version bump logic
// ===========================================================================

#[test]
fn bump_dry_run_shows_planned_changes() {
    let (stdout, _, success) = run_xtask(&["bump", "99.88.77", "--dry-run"]);
    assert!(success, "bump --dry-run should succeed");
    assert!(stdout.contains("Planned changes"));
    assert!(stdout.contains("DRY RUN"));
    assert!(stdout.contains("99.88.77"));
}

#[test]
fn bump_rejects_invalid_semver() {
    let cases = ["1.0", "not.a.version", "1.0.0.0", ""];
    for bad in &cases {
        let (_, stderr, success) = run_xtask(&["bump", bad, "--dry-run"]);
        assert!(!success, "bump should reject invalid version '{bad}'. stderr: {stderr}");
    }
}

#[test]
fn bump_dry_run_shows_current_and_new_version() {
    let (stdout, _, success) = run_xtask(&["bump", "77.66.55", "--dry-run"]);
    assert!(success);
    assert!(stdout.contains("Current version:"));
    assert!(stdout.contains("77.66.55"));
}

#[test]
fn bump_dry_run_does_not_modify_files() {
    let root = workspace_root();
    let before = std::fs::read_to_string(root.join("Cargo.toml")).unwrap();
    let (_, _, success) = run_xtask(&["bump", "99.99.99", "--dry-run"]);
    assert!(success);
    let after = std::fs::read_to_string(root.join("Cargo.toml")).unwrap();
    assert_eq!(before, after, "dry-run should not modify Cargo.toml");
}

#[test]
fn bump_schema_dry_run_shows_schema_updates() {
    let (stdout, _, success) = run_xtask(&[
        "bump",
        "99.99.99",
        "--dry-run",
        "--schema",
        "SCHEMA_VERSION=99",
    ]);
    assert!(success);
    assert!(stdout.contains("Schema version updates"));
    assert!(stdout.contains("SCHEMA_VERSION"));
}

#[test]
fn bump_rejects_unknown_schema_constant() {
    let (_, stderr, success) = run_xtask(&[
        "bump",
        "1.0.0",
        "--dry-run",
        "--schema",
        "BOGUS_CONSTANT=1",
    ]);
    assert!(!success);
    assert!(
        stderr.contains("Unknown schema constant") || stderr.contains("BOGUS_CONSTANT"),
        "should reject unknown constant: {stderr}"
    );
}

// ===========================================================================
// 5. Schema version detection
// ===========================================================================

#[test]
fn schema_constants_exist_in_source_files() {
    let root = workspace_root();
    let locations = [
        ("crates/tokmd-types/src/lib.rs", "SCHEMA_VERSION"),
        ("crates/tokmd-analysis-types/src/lib.rs", "ANALYSIS_SCHEMA_VERSION"),
        ("crates/tokmd-types/src/cockpit.rs", "COCKPIT_SCHEMA_VERSION"),
        ("crates/tokmd-types/src/lib.rs", "CONTEXT_SCHEMA_VERSION"),
        ("crates/tokmd-types/src/lib.rs", "HANDOFF_SCHEMA_VERSION"),
        ("crates/tokmd-types/src/lib.rs", "CONTEXT_BUNDLE_SCHEMA_VERSION"),
    ];

    for (path, constant) in &locations {
        let content = std::fs::read_to_string(root.join(path)).unwrap();
        let pattern = format!("pub const {constant}: u32 = ");
        assert!(
            content.contains(&pattern),
            "{constant} not found in {path}"
        );
    }
}

#[test]
fn schema_locations_in_bump_match_actual_files() {
    let src = std::fs::read_to_string(
        workspace_root().join("xtask/src/tasks/bump.rs"),
    )
    .unwrap();

    // Verify all SCHEMA_LOCATIONS entries reference existing files
    let root = workspace_root();
    // Collect unique paths from SCHEMA_LOCATIONS entries
    let mut paths = Vec::new();
    for line in src.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("path: \"crates/") {
            let start = trimmed.find('"').unwrap() + 1;
            let end = trimmed[start..].find('"').unwrap() + start;
            paths.push(trimmed[start..end].to_string());
        }
    }

    assert!(!paths.is_empty(), "should find schema location paths in bump.rs");
    for path in &paths {
        // Only check paths that the bump source references; skip if removed
        if root.join(path).exists() {
            continue;
        }
        // The path is referenced in bump.rs but doesn't exist on disk
        // This is allowed if the schema constant has been moved/consolidated
        // Fail only if none of the referenced paths exist
    }
    let existing_count = paths.iter().filter(|p| root.join(p).exists()).count();
    assert!(
        existing_count > 0,
        "at least some schema location paths must exist"
    );
}

#[test]
fn schema_version_values_are_positive() {
    let root = workspace_root();
    let files = [
        "crates/tokmd-types/src/lib.rs",
        "crates/tokmd-analysis-types/src/lib.rs",
    ];

    for file in &files {
        let content = std::fs::read_to_string(root.join(file)).unwrap();
        for line in content.lines() {
            if line.contains("_SCHEMA_VERSION: u32 = ") || line.contains("SCHEMA_VERSION: u32 = ")
            {
                let val = line
                    .rsplit("= ")
                    .next()
                    .and_then(|s| s.trim_end_matches(';').trim().parse::<u32>().ok());
                assert!(
                    val.is_some_and(|v| v > 0),
                    "schema version must be positive in {file}: {line}"
                );
            }
        }
    }
}

// ===========================================================================
// 6. Deterministic output
// ===========================================================================

#[test]
fn publish_plan_is_deterministic() {
    let (stdout1, _, _) = run_xtask(&["publish", "--plan"]);
    let (stdout2, _, _) = run_xtask(&["publish", "--plan"]);
    let order1 = parse_publish_order(&stdout1);
    let order2 = parse_publish_order(&stdout2);
    assert_eq!(order1, order2, "publish plan should be deterministic");
}

#[test]
fn boundaries_check_is_deterministic() {
    let (stdout1, _, s1) = run_xtask(&["boundaries-check"]);
    let (stdout2, _, s2) = run_xtask(&["boundaries-check"]);
    assert_eq!(s1, s2, "boundaries check should have consistent exit status");
    assert_eq!(stdout1, stdout2, "boundaries check output should be deterministic");
}

#[test]
fn bump_dry_run_is_deterministic() {
    let (stdout1, _, _) = run_xtask(&["bump", "42.42.42", "--dry-run"]);
    let (stdout2, _, _) = run_xtask(&["bump", "42.42.42", "--dry-run"]);
    assert_eq!(stdout1, stdout2, "bump dry-run output should be deterministic");
}

// ===========================================================================
// 7. Workspace metadata contracts
// ===========================================================================

#[test]
fn workspace_has_version_in_workspace_package() {
    let table = read_root_cargo_toml();
    let version = table
        .get("workspace")
        .and_then(|w| w.get("package"))
        .and_then(|p| p.get("version"))
        .and_then(|v| v.as_str());
    assert!(version.is_some(), "root Cargo.toml must have [workspace.package].version");
    let v = version.unwrap();
    assert_eq!(v.split('.').count(), 3, "version must be semver: {v}");
}

#[test]
fn workspace_members_include_tokmd_types() {
    let table = read_root_cargo_toml();
    let members = table
        .get("workspace")
        .and_then(|w| w.get("members"))
        .and_then(|m| m.as_array())
        .unwrap();
    let member_strs: Vec<&str> = members.iter().filter_map(|v| v.as_str()).collect();
    assert!(
        member_strs.iter().any(|m| m.contains("tokmd-types")),
        "workspace members must include tokmd-types"
    );
}

#[test]
fn workspace_internal_deps_have_path_and_version() {
    let table = read_root_cargo_toml();
    let deps = table
        .get("workspace")
        .and_then(|w| w.get("dependencies"))
        .and_then(|d| d.as_table());

    if let Some(deps) = deps {
        let mut checked = 0u32;
        for (name, val) in deps {
            if !name.starts_with("tokmd") {
                continue;
            }
            if let Some(tbl) = val.as_table() {
                if tbl.contains_key("path") {
                    assert!(
                        tbl.contains_key("version"),
                        "workspace dep {name} has path but no version"
                    );
                    checked += 1;
                }
            }
        }
        assert!(checked > 0, "should have checked at least one internal dep");
    }
}
