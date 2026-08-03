//! CLI scan-option tests (w66).
//!
//! Validates that scan-related flags (--exclude, --module-depth, --children,
//! --hidden, --no-ignore, etc.) work correctly, including edge cases like
//! empty directories and self-referential scans.

mod common;

use assert_cmd::Command;
use predicates::prelude::*;
use serde_json::Value;

fn tokmd_cmd() -> Command {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_tokmd"));
    cmd.current_dir(common::fixture_root());
    cmd
}

// ===========================================================================
// 1. --exclude pattern tests
// ===========================================================================

#[test]
fn exclude_pattern_reduces_output() {
    let baseline = tokmd_cmd()
        .args(["lang", "--format", "json"])
        .output()
        .expect("baseline run");
    assert!(baseline.status.success());
    let base_json: Value = serde_json::from_slice(&baseline.stdout).unwrap();
    let base_total: u64 = base_json["total"]["code"]
        .as_u64()
        .or_else(|| base_json["totals"]["code"].as_u64())
        .unwrap_or(0);

    let excluded = tokmd_cmd()
        .args(["lang", "--format", "json", "--exclude", "*.rs"])
        .output()
        .expect("excluded run");
    assert!(excluded.status.success());
    let excl_json: Value = serde_json::from_slice(&excluded.stdout).unwrap();
    let excl_total: u64 = excl_json["total"]["code"]
        .as_u64()
        .or_else(|| excl_json["totals"]["code"].as_u64())
        .unwrap_or(0);

    assert!(
        excl_total <= base_total,
        "excluding *.rs should not increase code count: excluded={excl_total} baseline={base_total}"
    );
}

#[test]
fn exclude_multiple_patterns() {
    tokmd_cmd()
        .args([
            "lang",
            "--format",
            "json",
            "--exclude",
            "*.rs",
            "--exclude",
            "*.js",
        ])
        .assert()
        .success();
}

#[test]
fn exclude_all_files_produces_zero_rows() {
    let output = tokmd_cmd()
        .args(["lang", "--format", "json", "--exclude", "*"])
        .output()
        .expect("failed to run");

    assert!(output.status.success());
    let json: Value = serde_json::from_slice(&output.stdout).unwrap();
    let rows = json["rows"].as_array().expect("rows array");
    assert!(
        rows.is_empty(),
        "excluding everything should produce zero rows"
    );
}

#[test]
fn export_exclude_filters_files() {
    let output = tokmd_cmd()
        .args(["export", "--format", "json", "--exclude", "*.js"])
        .output()
        .expect("failed to run");

    assert!(output.status.success());
    let json: Value = serde_json::from_slice(&output.stdout).unwrap();
    let rows = json["rows"].as_array().expect("rows array");

    for row in rows {
        let path = row["path"].as_str().unwrap_or("");
        assert!(
            !path.ends_with(".js"),
            "excluded .js file still appears: {path}"
        );
    }
}

// ===========================================================================
// 2. --module-depth tests
// ===========================================================================

#[test]
fn module_depth_one_groups_at_top_level() {
    let output = tokmd_cmd()
        .args(["module", "--format", "json", "--module-depth", "1"])
        .output()
        .expect("failed to run");

    assert!(output.status.success());
    let json: Value = serde_json::from_slice(&output.stdout).unwrap();
    let rows = json["rows"].as_array().expect("rows array");

    for row in rows {
        let module = row["module"].as_str().unwrap_or("");
        // At depth 1, module keys should have at most one path separator
        let depth = module.matches('/').count();
        assert!(
            depth <= 1,
            "module-depth 1 should produce shallow keys, got: {module}"
        );
    }
}

#[test]
fn module_depth_large_value_succeeds() {
    tokmd_cmd()
        .args(["module", "--format", "json", "--module-depth", "100"])
        .assert()
        .success();
}

// ===========================================================================
// 3. Scanning empty directories
// ===========================================================================

#[test]
fn scan_empty_directory_succeeds_with_zero_results() {
    let dir = tempfile::tempdir().expect("create temp dir");
    std::fs::create_dir(dir.path().join(".git")).expect("create .git");

    let mut cmd = Command::new(env!("CARGO_BIN_EXE_tokmd"));
    cmd.current_dir(dir.path());
    let output = cmd
        .args(["lang", "--format", "json"])
        .output()
        .expect("failed to run");

    assert!(output.status.success());
    let json: Value = serde_json::from_slice(&output.stdout).unwrap();
    let rows = json["rows"].as_array().expect("rows array");
    assert!(rows.is_empty(), "empty dir should have zero rows");
}

#[test]
fn export_empty_directory_succeeds() {
    let dir = tempfile::tempdir().expect("create temp dir");
    std::fs::create_dir(dir.path().join(".git")).expect("create .git");

    let mut cmd = Command::new(env!("CARGO_BIN_EXE_tokmd"));
    cmd.current_dir(dir.path());
    cmd.args(["export", "--format", "json"]).assert().success();
}

#[test]
fn module_empty_directory_succeeds() {
    let dir = tempfile::tempdir().expect("create temp dir");
    std::fs::create_dir(dir.path().join(".git")).expect("create .git");

    let mut cmd = Command::new(env!("CARGO_BIN_EXE_tokmd"));
    cmd.current_dir(dir.path());
    cmd.args(["module", "--format", "json"]).assert().success();
}

// ===========================================================================
// 4. Self-referential scan (scan the tokmd repo itself)
// ===========================================================================

#[test]
fn self_scan_succeeds() {
    let repo_root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .to_path_buf();

    let mut cmd = Command::new(env!("CARGO_BIN_EXE_tokmd"));
    cmd.current_dir(&repo_root);
    cmd.args(["lang", "--format", "json", "--exclude", "target"])
        .assert()
        .success();
}

#[test]
fn self_scan_finds_rust_language() {
    let repo_root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .to_path_buf();

    let mut cmd = Command::new(env!("CARGO_BIN_EXE_tokmd"));
    cmd.current_dir(&repo_root);
    let output = cmd
        .args(["lang", "--format", "json", "--exclude", "target"])
        .output()
        .expect("failed to run");

    assert!(output.status.success());
    let json: Value = serde_json::from_slice(&output.stdout).unwrap();
    let rows = json["rows"].as_array().expect("rows array");

    let has_rust = rows.iter().any(|r| r["lang"].as_str() == Some("Rust"));
    assert!(has_rust, "self-scan should find Rust language");
}

// ===========================================================================
// 5. --children mode flags
// ===========================================================================

#[test]
fn lang_children_collapse_succeeds() {
    tokmd_cmd()
        .args(["lang", "--format", "json", "--children", "collapse"])
        .assert()
        .success();
}

#[test]
fn lang_children_separate_succeeds() {
    tokmd_cmd()
        .args(["lang", "--format", "json", "--children", "separate"])
        .assert()
        .success();
}

#[test]
fn module_children_separate_succeeds() {
    tokmd_cmd()
        .args(["module", "--format", "json", "--children", "separate"])
        .assert()
        .success();
}

#[test]
fn export_children_separate_succeeds() {
    tokmd_cmd()
        .args(["export", "--format", "json", "--children", "separate"])
        .assert()
        .success();
}

#[test]
fn lang_children_invalid_value_fails() {
    tokmd_cmd()
        .args(["lang", "--children", "invalid"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("invalid value"));
}

// ===========================================================================
// 6. --hidden and --no-ignore flags (global, placed before subcommand)
// ===========================================================================

#[test]
fn hidden_flag_succeeds() {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_tokmd"));
    cmd.current_dir(common::fixture_root());
    cmd.args(["--hidden", "lang", "--format", "json"])
        .assert()
        .success();
}

#[test]
fn no_ignore_flag_succeeds() {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_tokmd"));
    cmd.current_dir(common::fixture_root());
    cmd.args(["--no-ignore", "lang", "--format", "json"])
        .assert()
        .success();
}

// ===========================================================================
// 6b. Scan flags also work AFTER the subcommand
//
// These were previously root-only: `tokmd export --hidden` failed with
// "unexpected argument '--hidden' found" (and clap unhelpfully suggested
// '--children'), while `tokmd --hidden export` worked. Users have no way to
// tell which flags need to lead, so all scan options are now `global = true`.
// ===========================================================================

#[test]
fn hidden_flag_after_subcommand_is_accepted() {
    tokmd_cmd()
        .args(["export", "--hidden", "--format", "json"])
        .assert()
        .success();
}

type TestResult = Result<(), Box<dyn std::error::Error>>;

/// Run `tokmd` and return the `scan` object from the export meta line.
///
/// Compares the meta line's `scan` object rather than the whole line: the meta
/// record also carries `generated_at_ms`, which differs between runs.
///
/// Errors propagate with `?` rather than `expect` so callers add no
/// panic-family debt to policy/no-panic-allowlist.toml.
fn scan_options(args: &[&str]) -> Result<Value, Box<dyn std::error::Error>> {
    let out = tokmd_cmd().args(args).output()?;
    if !out.status.success() {
        return Err(format!("{args:?} failed: {}", String::from_utf8_lossy(&out.stderr)).into());
    }
    let stdout = String::from_utf8(out.stdout)?;
    let first = stdout.lines().next().ok_or("no meta line on stdout")?;
    let meta: Value = serde_json::from_str(first)?;
    Ok(meta
        .get("scan")
        .ok_or("meta line has no `scan` object")?
        .clone())
}

#[test]
fn hidden_flag_reaches_scan_options_from_either_position() -> TestResult {
    // Both orderings must produce the same scan configuration, not merely exit 0.
    let leading = scan_options(&["--hidden", "export", "--format", "json"])?;
    let trailing = scan_options(&["export", "--hidden", "--format", "json"])?;
    assert_eq!(
        leading, trailing,
        "both flag positions must yield the same scan options"
    );
    assert_eq!(
        trailing.get("hidden"),
        Some(&Value::Bool(true)),
        "--hidden after the subcommand must actually reach the scan"
    );
    Ok(())
}

#[test]
fn no_ignore_flag_after_subcommand_is_accepted() {
    tokmd_cmd()
        .args(["module", "--no-ignore", "--format", "json"])
        .assert()
        .success();
}

#[test]
fn every_global_scan_flag_reaches_scan_options_from_either_position() -> TestResult {
    // `global = true` on the clap arg and the value actually arriving in the
    // scan are two different things, and only the second is what users see.
    // Covering one flag would let a regression that globalizes all seven but
    // threads only some of them through slip past, so every flag the change
    // touched is checked here rather than trusting the shape to generalize.
    //
    // Each case is (flag tokens, meta key, expected value). `--config` takes a
    // value and the rest are switches, hence the token slice.
    let cases: &[(&[&str], &str, Value)] = &[
        (&["--hidden"], "hidden", Value::Bool(true)),
        (&["--no-ignore"], "no_ignore", Value::Bool(true)),
        (
            &["--no-ignore-parent"],
            "no_ignore_parent",
            Value::Bool(true),
        ),
        (&["--no-ignore-dot"], "no_ignore_dot", Value::Bool(true)),
        (&["--no-ignore-vcs"], "no_ignore_vcs", Value::Bool(true)),
        (
            &["--treat-doc-strings-as-comments"],
            "treat_doc_strings_as_comments",
            Value::Bool(true),
        ),
        // Non-default on purpose: `config` defaults to "auto", so asserting
        // "auto" would pass even if the flag never reached the scan at all.
        (
            &["--config", "none"],
            "config",
            Value::String("none".to_string()),
        ),
    ];

    // Globalizing an arg and each subcommand threading its value into the scan
    // are also two different things: a regression that wires the value into
    // only one sink would pass if `export` were the sole subcommand tested.
    // `module` is the other subcommand that emits a `meta.scan` object.
    for subcommand in ["export", "module"] {
        for (flag, key, expected) in cases {
            let mut lead: Vec<&str> = flag.to_vec();
            lead.extend_from_slice(&[subcommand, "--format", "json"]);
            let mut trail: Vec<&str> = vec![subcommand];
            trail.extend_from_slice(flag);
            trail.extend_from_slice(&["--format", "json"]);

            let leading = scan_options(&lead)?;
            let trailing = scan_options(&trail)?;
            assert_eq!(
                leading, trailing,
                "{flag:?} must yield the same scan options before and after `{subcommand}`"
            );
            assert_eq!(
                trailing.get(*key),
                Some(expected),
                "{flag:?} after `{subcommand}` must reach the scan as `{key}`"
            );
        }
    }
    Ok(())
}

#[test]
fn check_ignore_keeps_its_own_short_verbose() {
    // `check-ignore` defines its own -v/--verbose as a bool. The root
    // --verbose is deliberately NOT global so clap does not see a duplicate
    // arg id here (which would panic at startup rather than fail gracefully).
    //
    // Asserting the path is echoed would not discriminate: `check-ignore`
    // prints it either way, so a regression that re-globalized the root
    // --verbose and let it swallow `-v` would still pass. The verbose note is
    // the only output that distinguishes the two slots, so assert on that.
    // Exit code is 1 because `Cargo.toml` is tracked and therefore not ignored.
    //
    // Run against the repository rather than the scan fixture: the note says
    // the path is tracked by git, which requires a real checkout. The fixture
    // root is not a repository, so the note never appears there and the
    // assertion would fail for a reason unrelated to flag routing.
    // Built at compile time rather than walked with `parent().unwrap()`: the
    // no-panic policy counts those unwraps as new panic-family debt, and the
    // manifest dir is two levels below the workspace root by construction.
    let repo_root = std::path::Path::new(concat!(env!("CARGO_MANIFEST_DIR"), "/../.."));
    let in_repo = || {
        let mut cmd = Command::new(env!("CARGO_BIN_EXE_tokmd"));
        cmd.current_dir(repo_root);
        cmd
    };

    in_repo()
        .args(["check-ignore", "-v", "Cargo.toml"])
        .assert()
        .code(1)
        .stdout(predicate::str::contains("Cargo.toml"))
        .stdout(predicate::str::contains("note:"))
        .stderr(predicate::str::contains("unexpected argument").not());

    // The pair matters: without `-v` there is no note, which is what makes the
    // assertion above evidence of routing rather than incidentally true.
    in_repo()
        .args(["check-ignore", "Cargo.toml"])
        .assert()
        .code(1)
        .stdout(predicate::str::contains("Cargo.toml"))
        .stdout(predicate::str::contains("note:").not());
}

// ===========================================================================
// 7. --top flag
// ===========================================================================

#[test]
fn top_flag_limits_output() {
    let full = tokmd_cmd()
        .args(["lang", "--format", "json"])
        .output()
        .expect("full run");
    assert!(full.status.success());
    let full_json: Value = serde_json::from_slice(&full.stdout).unwrap();
    let full_rows = full_json["rows"].as_array().expect("rows array").len();

    let limited = tokmd_cmd()
        .args(["lang", "--format", "json", "--top", "1"])
        .output()
        .expect("limited run");
    assert!(limited.status.success());
    let lim_json: Value = serde_json::from_slice(&limited.stdout).unwrap();
    let lim_rows = lim_json["rows"].as_array().expect("rows array").len();

    assert!(
        lim_rows <= full_rows,
        "--top 1 should produce <= full rows: limited={lim_rows} full={full_rows}"
    );
}

#[test]
fn top_zero_shows_all_languages() {
    let output = tokmd_cmd()
        .args(["lang", "--format", "json", "--top", "0"])
        .output()
        .expect("failed to run");

    assert!(output.status.success());
    let json: Value = serde_json::from_slice(&output.stdout).unwrap();
    let rows = json["rows"].as_array().expect("rows array");
    assert!(
        !rows.is_empty(),
        "--top 0 should show all languages (fixture has code)"
    );
}
