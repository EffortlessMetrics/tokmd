//! Deep xtask tests (Wave 74).
//!
//! Focuses on logic not covered by earlier waves: error classification,
//! publish command reconstruction, gate step structure, boundaries on
//! non-analysis crates, task registration completeness, and edge-case
//! argument handling.

use std::collections::BTreeMap;
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

fn read_source(relative: &str) -> String {
    std::fs::read_to_string(workspace_root().join(relative)).unwrap()
}

// ===========================================================================
// 1. Boundaries-check: tier violation detection
// ===========================================================================

#[test]
fn boundaries_check_forbidden_list_covers_higher_tiers() {
    // The FORBIDDEN list must prevent analysis crates (tier 3) from depending
    // on tier-4 crates. Verify tokmd-config is the minimum set.
    let src = read_source("xtask/src/tasks/boundaries_check.rs");
    assert!(
        src.contains("tokmd-config"),
        "FORBIDDEN must block tier-4 tokmd-config"
    );
}

#[test]
fn boundaries_no_analysis_crate_depends_on_core() {
    // tokmd-core is tier 4 — analysis crates (tier 3) must not depend on it.
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

        if let Some(toml::Value::Table(deps)) = table.get("dependencies") {
            assert!(
                !deps.contains_key("tokmd-core"),
                "{name} must not depend on tokmd-core (tier 4)"
            );
        }
        checked += 1;
    }
    assert!(
        checked > 0,
        "should have checked at least one analysis crate"
    );
}

#[test]
fn boundaries_check_scans_sorted_manifests() {
    // The source collects and sorts manifests for deterministic ordering.
    let src = read_source("xtask/src/tasks/boundaries_check.rs");
    assert!(
        src.contains("manifests.sort()"),
        "boundaries_check should sort manifests for determinism"
    );
}

// ===========================================================================
// 2. Publish: error classification patterns
// ===========================================================================

#[test]
fn publish_classifies_already_published() {
    let src = read_source("xtask/src/tasks/publish.rs");
    assert!(
        src.contains("is already uploaded"),
        "should detect 'is already uploaded' pattern"
    );
    assert!(
        src.contains("already exists"),
        "should detect 'already exists' pattern"
    );
}

#[test]
fn publish_classifies_auth_errors() {
    let src = read_source("xtask/src/tasks/publish.rs");
    assert!(
        src.contains("not logged in"),
        "should detect 'not logged in' auth error"
    );
    assert!(
        src.contains("unauthorized"),
        "should detect 'unauthorized' auth error"
    );
}

#[test]
fn publish_classifies_propagation_delay() {
    let src = read_source("xtask/src/tasks/publish.rs");
    assert!(
        src.contains("no matching package named"),
        "should detect propagation delay via 'no matching package named'"
    );
    assert!(
        src.contains("failed to select a version"),
        "should detect propagation delay via version selection failure"
    );
}

#[test]
fn publish_classifies_network_errors() {
    let src = read_source("xtask/src/tasks/publish.rs");
    for pattern in &["network", "connection", "timeout", "timed out"] {
        assert!(
            src.contains(pattern),
            "should detect network error pattern: {pattern}"
        );
    }
}

#[test]
fn publish_reconstruct_command_includes_scope_filters() {
    // The reconstruct function should preserve --crates, --exclude, --from flags.
    let src = read_source("xtask/src/tasks/publish.rs");
    assert!(
        src.contains("reconstruct_publish_command"),
        "should have reconstruct_publish_command helper"
    );
    // Verify it handles all scope filters
    for flag in &["--crates", "--exclude", "--from"] {
        assert!(
            src.contains(flag),
            "reconstruct_publish_command should handle {flag}"
        );
    }
}

#[test]
fn publish_plan_with_crates_filter() {
    // --crates with a known crate should succeed and include transitive deps.
    let (stdout, stderr, success) = run_xtask(&["publish", "--plan", "--crates", "tokmd-types"]);
    assert!(
        success,
        "publish --plan --crates tokmd-types should succeed. stderr: {stderr}"
    );
    let order = parse_publish_order(&stdout);
    assert!(
        order.iter().any(|n| n == "tokmd-types"),
        "filtered plan should include tokmd-types"
    );
    // Should NOT include unrelated crates
    assert!(
        !order.iter().any(|n| n == "tokmd"),
        "filtered plan should not include the CLI binary"
    );
}

#[test]
fn publish_plan_with_nonexistent_crate_fails() {
    let (_, stderr, success) =
        run_xtask(&["publish", "--plan", "--crates", "nonexistent-crate-xyz"]);
    assert!(!success, "plan with nonexistent crate should fail");
    assert!(
        stderr.contains("nonexistent-crate-xyz"),
        "error should mention the bad crate name: {stderr}"
    );
}

// ===========================================================================
// 3. Docs: marker and task structure
// ===========================================================================

#[test]
fn docs_task_markers_list_covers_all_subcommands() {
    // The docs task has a `markers` array that should include every major
    // CLI subcommand to keep reference-cli.md in sync.
    let src = read_source("xtask/src/tasks/docs.rs");
    let required = [
        "lang",
        "module",
        "export",
        "run",
        "analyze",
        "badge",
        "diff",
        "context",
        "cockpit",
        "gate",
        "completions",
    ];
    for cmd in &required {
        assert!(
            src.contains(&format!("\"{cmd}\"")),
            "docs task markers should include \"{cmd}\""
        );
    }
}

#[test]
fn docs_task_uses_cargo_run_for_help() {
    // Verify docs task invokes `cargo run -p tokmd` (not the binary directly)
    // so it works without a pre-built binary.
    let src = read_source("xtask/src/tasks/docs.rs");
    assert!(
        src.contains(r#""cargo""#) && src.contains(r#""-p""#) && src.contains(r#""tokmd""#),
        "docs task should invoke cargo run -p tokmd for --help output"
    );
}

#[test]
fn docs_no_flags_does_not_fail() {
    // Running `cargo xtask docs` (no --check, no --update) should succeed.
    let (stdout, stderr, success) = run_xtask(&["docs"]);
    assert!(
        success,
        "docs without flags should not error. stderr: {stderr}"
    );
    assert!(
        stdout.contains("up to date") || stdout.contains("Updated"),
        "should report status: {stdout}"
    );
}

// ===========================================================================
// 4. Gate: step structure and check-mode semantics
// ===========================================================================

#[test]
fn gate_steps_include_fmt_check_clippy_test() {
    // Verify the STEPS constant covers the four critical pipeline stages.
    let src = read_source("xtask/src/tasks/gate.rs");
    for label in &["fmt", "check", "clippy", "test"] {
        assert!(
            src.contains(label),
            "gate STEPS should include a '{label}' step"
        );
    }
}

#[test]
fn gate_fmt_step_has_check_mode_variant() {
    // The fmt step should have a check_args variant that adds `--check`.
    let src = read_source("xtask/src/tasks/gate.rs");
    assert!(
        src.contains("check_args: Some"),
        "at least one step should have a check_args variant"
    );
    assert!(
        src.contains("\"--check\""),
        "check_args should include --check flag"
    );
}

#[test]
fn gate_excludes_tokmd_python() {
    // tokmd-python requires PyO3/maturin and should be excluded from gate checks.
    let src = read_source("xtask/src/tasks/gate.rs");
    assert!(
        src.contains("tokmd-python"),
        "gate steps should exclude tokmd-python"
    );
}

#[test]
fn gate_excludes_xtask_from_compile_only_step() {
    let src = read_source("xtask/src/tasks/gate.rs");
    assert!(
        src.contains("\"xtask\""),
        "gate compile-only step should exclude xtask to avoid self-rebuild conflicts"
    );
}

#[test]
fn gate_reports_pass_fail_count() {
    // The gate runner should print a summary like "gate result: N/M steps passed".
    let src = read_source("xtask/src/tasks/gate.rs");
    assert!(
        src.contains("gate result:"),
        "gate should print 'gate result:' summary"
    );
}

#[test]
fn gate_check_flag_exists_in_cli() {
    let src = read_source("xtask/src/cli.rs");
    // GateArgs should have a --check flag
    assert!(
        src.contains("pub check: bool") || src.contains("check:"),
        "GateArgs should have a check flag"
    );
}

#[test]
fn gate_runtime_guard_keeps_curated_jules_deps_history() {
    let src = read_source("xtask/src/tasks/gate.rs");
    assert!(
        src.contains("\".jules/runs\""),
        "gate should treat root .jules/runs as runtime state"
    );
    assert!(
        src.contains("Curated `.jules/deps/**` history is allowed"),
        "gate should document the curated .jules/deps allowance"
    );
}

// ===========================================================================
// 5. Bump: version logic edge cases
// ===========================================================================

#[test]
fn bump_same_version_is_noop() {
    // Read the current workspace version and try bumping to it.
    let root_toml = std::fs::read_to_string(workspace_root().join("Cargo.toml")).unwrap();
    let current_version = root_toml
        .lines()
        .find(|l| l.trim().starts_with("version = \""))
        .and_then(|l| {
            let start = l.find('"')? + 1;
            let end = l[start..].find('"')? + start;
            Some(l[start..end].to_string())
        })
        .expect("should find workspace version");

    let (stdout, _, success) = run_xtask(&["bump", &current_version, "--dry-run"]);
    assert!(success, "bumping to same version should succeed");
    assert!(
        stdout.contains("Nothing to do") || stdout.contains("already"),
        "same-version bump should be a no-op: {stdout}"
    );
}

#[test]
fn bump_schema_invalid_format_rejected() {
    // Schema bump without '=' separator should fail.
    let (_, stderr, success) =
        run_xtask(&["bump", "1.0.0", "--dry-run", "--schema", "SCHEMA_VERSION"]);
    assert!(!success, "schema without = should fail");
    assert!(
        stderr.contains("NAME=VERSION") || stderr.contains("format"),
        "should explain required format: {stderr}"
    );
}

#[test]
fn bump_schema_non_numeric_version_rejected() {
    let (_, stderr, success) = run_xtask(&[
        "bump",
        "1.0.0",
        "--dry-run",
        "--schema",
        "SCHEMA_VERSION=abc",
    ]);
    assert!(!success, "non-numeric schema version should fail");
    assert!(
        stderr.contains("Invalid") || stderr.contains("abc"),
        "should report parsing failure: {stderr}"
    );
}

#[test]
fn bump_workspace_dep_update_targets_tokmd_prefix() {
    // The update_workspace_dependencies function should only touch tokmd-* deps.
    let src = read_source("xtask/src/tasks/bump.rs");
    assert!(
        src.contains(r#"starts_with("tokmd")"#),
        "workspace dep update should only target tokmd-* crates"
    );
}

// ===========================================================================
// 6. Task registration: all Commands variants are wired up
// ===========================================================================

#[test]
fn all_commands_dispatched_in_main() {
    // Every Commands variant from cli.rs should be matched in main.rs.
    let cli_src = read_source("xtask/src/cli.rs");
    let main_src = read_source("xtask/src/main.rs");

    // Extract variant names from the Commands enum
    let mut variants = Vec::new();
    let mut in_enum = false;
    for line in cli_src.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("pub enum Commands") {
            in_enum = true;
            continue;
        }
        if in_enum && trimmed == "}" {
            break;
        }
        if in_enum {
            // Lines like "    Bump(BumpArgs)," or "    /// doc comment"
            if let Some(name) = trimmed.strip_suffix(')').or(Some(trimmed)) {
                let name = name.split('(').next().unwrap_or("").trim();
                if !name.is_empty() && !name.starts_with("//") && !name.starts_with('#') {
                    variants.push(name.trim_end_matches(',').to_string());
                }
            }
        }
    }

    assert!(
        variants.len() >= 5,
        "should find at least 5 command variants, found: {variants:?}"
    );

    for variant in &variants {
        assert!(
            main_src.contains(variant),
            "Command variant {variant} should be dispatched in main.rs"
        );
    }
}

#[test]
fn all_task_modules_declared_in_mod_rs() {
    let mod_src = read_source("xtask/src/tasks/mod.rs");
    let expected_modules = [
        "boundaries_check",
        "bump",
        "cockpit",
        "docs",
        "gate",
        "lint_fix",
        "publish",
        "sccache",
        "trim_target",
        "workspace",
    ];
    for module in &expected_modules {
        assert!(
            mod_src.contains(&format!("pub mod {module}")),
            "tasks/mod.rs should declare pub mod {module}"
        );
    }
}

// ===========================================================================
// 7. Crate ordering: fine-grained dependency constraints
// ===========================================================================

#[test]
fn publish_order_envelope_before_sensor() {
    let (stdout, _, _) = run_xtask(&["publish", "--plan"]);
    let order = parse_publish_order(&stdout);
    let pos: BTreeMap<&str, usize> = order
        .iter()
        .enumerate()
        .map(|(i, n)| (n.as_str(), i))
        .collect();

    if let (Some(&env_p), Some(&sensor_p)) = (pos.get("tokmd-envelope"), pos.get("tokmd-sensor")) {
        assert!(
            env_p < sensor_p,
            "tokmd-envelope must come before tokmd-sensor"
        );
    }
}

#[test]
fn publish_order_analysis_deps_before_analysis() {
    // tokmd-analysis's own workspace deps should appear before it.
    let root = workspace_root();
    let cargo_toml = root
        .join("crates")
        .join("tokmd-analysis")
        .join("Cargo.toml");
    if !cargo_toml.exists() {
        return;
    }
    let content = std::fs::read_to_string(&cargo_toml).unwrap();
    let table: toml::Table = toml::from_str(&content).unwrap();
    let deps: Vec<String> = table
        .get("dependencies")
        .and_then(|d| d.as_table())
        .map(|d| {
            d.keys()
                .filter(|k| k.starts_with("tokmd-"))
                .cloned()
                .collect()
        })
        .unwrap_or_default();

    let (stdout, _, _) = run_xtask(&["publish", "--plan"]);
    let order = parse_publish_order(&stdout);
    let pos: BTreeMap<&str, usize> = order
        .iter()
        .enumerate()
        .map(|(i, n)| (n.as_str(), i))
        .collect();

    let analysis_pos = match pos.get("tokmd-analysis") {
        Some(&p) => p,
        None => return,
    };

    for dep in &deps {
        if let Some(&dep_pos) = pos.get(dep.as_str()) {
            assert!(
                dep_pos < analysis_pos,
                "{dep} (pos {dep_pos}) should come before tokmd-analysis (pos {analysis_pos})"
            );
        }
    }
}

#[test]
fn publish_order_config_before_core() {
    let (stdout, _, _) = run_xtask(&["publish", "--plan"]);
    let order = parse_publish_order(&stdout);
    let pos: BTreeMap<&str, usize> = order
        .iter()
        .enumerate()
        .map(|(i, n)| (n.as_str(), i))
        .collect();

    if let (Some(&cfg_p), Some(&core_p)) = (pos.get("tokmd-config"), pos.get("tokmd-core")) {
        assert!(
            cfg_p < core_p,
            "tokmd-config (tier 4) must come before tokmd-core (tier 4 facade)"
        );
    }
}

// ===========================================================================
// 8. Error handling: invalid arguments
// ===========================================================================

#[test]
fn publish_plan_invalid_exclude_dependency_fails() {
    // Excluding tokmd-types should fail because many crates depend on it.
    let (_, stderr, success) = run_xtask(&["publish", "--plan", "--exclude", "tokmd-types"]);
    assert!(
        !success,
        "excluding a critical dep (tokmd-types) should fail"
    );
    assert!(
        stderr.contains("Cannot exclude") || stderr.contains("depends on"),
        "should explain the dependency violation: {stderr}"
    );
}

#[test]
fn publish_plan_invalid_from_crate_fails() {
    // --from with a nonexistent crate should fail (tested on non-plan mode
    // which validates --from against the order).
    let (_, stderr, success) = run_xtask(&[
        "publish",
        "--dry-run",
        "--skip-checks",
        "--from",
        "nonexistent-crate",
    ]);
    assert!(!success, "--from with nonexistent crate should fail");
    assert!(
        stderr.contains("nonexistent-crate") || stderr.contains("not found"),
        "should report the bad --from crate: {stderr}"
    );
}

#[test]
fn lint_fix_help_shows_check_and_no_clippy() {
    let output = Command::new("cargo")
        .args(["run", "-q", "-p", "xtask", "--", "lint-fix", "--help"])
        .current_dir(workspace_root())
        .output()
        .expect("failed to run lint-fix --help");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(output.status.success());
    assert!(
        stdout.contains("--check"),
        "lint-fix should have --check flag"
    );
    assert!(
        stdout.contains("--no-clippy"),
        "lint-fix should have --no-clippy flag"
    );
}

#[test]
fn trim_target_help_shows_trim_flags() {
    let output = Command::new("cargo")
        .args(["run", "-q", "-p", "xtask", "--", "trim-target", "--help"])
        .current_dir(workspace_root())
        .output()
        .expect("failed to run trim-target --help");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(output.status.success(), "trim-target --help should succeed");
    assert!(
        stdout.contains("--check"),
        "trim-target should have --check flag"
    );
    assert!(
        stdout.contains("--keep-pdb"),
        "trim-target should have --keep-pdb flag"
    );
    assert!(
        stdout.contains("--keep-incremental"),
        "trim-target should have --keep-incremental flag"
    );
}

#[test]
fn sccache_help_shows_wrapper_flags() {
    let output = Command::new("cargo")
        .args(["run", "-q", "-p", "xtask", "--", "sccache", "--help"])
        .current_dir(workspace_root())
        .output()
        .expect("failed to run sccache --help");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(output.status.success(), "sccache --help should succeed");
    for flag in &[
        "--check",
        "--stats",
        "--stop",
        "--keep-incremental",
        "--basedir",
    ] {
        assert!(stdout.contains(flag), "sccache help should mention {flag}");
    }
}

#[test]
fn sccache_task_sets_wrapper_and_incremental_policy() {
    let src = read_source("xtask/src/tasks/sccache.rs");
    assert!(
        src.contains("RUSTC_WRAPPER"),
        "sccache task should set RUSTC_WRAPPER"
    );
    assert!(
        src.contains("CARGO_INCREMENTAL"),
        "sccache task should manage incremental policy"
    );
    assert!(
        src.contains("SCCACHE_SERVER_PORT"),
        "sccache task should isolate the repo-native server port"
    );
    assert!(
        src.contains("SCCACHE_BASEDIRS"),
        "sccache task should propagate basedir normalization"
    );
    assert!(
        src.contains("cargo with-sccache test"),
        "sccache check output should document the repo-native opt-in path"
    );
}

#[test]
fn cockpit_help_shows_base_head_format() {
    let output = Command::new("cargo")
        .args(["run", "-q", "-p", "xtask", "--", "cockpit", "--help"])
        .current_dir(workspace_root())
        .output()
        .expect("failed to run cockpit --help");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(output.status.success());
    assert!(stdout.contains("--base"), "cockpit should have --base flag");
    assert!(stdout.contains("--head"), "cockpit should have --head flag");
    assert!(
        stdout.contains("--format"),
        "cockpit should have --format flag"
    );
}
