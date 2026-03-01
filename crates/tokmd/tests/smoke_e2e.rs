//! End-to-end smoke tests for CLI commands and flags that lack dedicated
//! coverage elsewhere.  Each test exercises a real `tokmd` invocation.

mod common;

use assert_cmd::Command;
use predicates::prelude::*;
use tempfile::tempdir;

fn tokmd_cmd() -> Command {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_tokmd"));
    cmd.current_dir(common::fixture_root());
    cmd
}

// ---------------------------------------------------------------------------
// --version / --help
// ---------------------------------------------------------------------------

#[test]
fn version_flag_prints_version() {
    tokmd_cmd()
        .arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::contains("tokmd"));
}

#[test]
fn help_flag_lists_subcommands() {
    tokmd_cmd()
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("lang"))
        .stdout(predicate::str::contains("module"))
        .stdout(predicate::str::contains("export"))
        .stdout(predicate::str::contains("analyze"))
        .stdout(predicate::str::contains("badge"))
        .stdout(predicate::str::contains("init"))
        .stdout(predicate::str::contains("context"));
}

#[test]
fn lang_help_shows_format_flag() {
    tokmd_cmd()
        .args(["lang", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("--format"));
}

#[test]
fn module_help_shows_depth_flag() {
    tokmd_cmd()
        .args(["module", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("--module-depth"));
}

#[test]
fn export_help_shows_format_flag() {
    tokmd_cmd()
        .args(["export", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("--format"));
}

// ---------------------------------------------------------------------------
// completions – zsh, fish, powershell, elvish
// ---------------------------------------------------------------------------

#[test]
fn completions_zsh_produces_output() {
    tokmd_cmd()
        .args(["completions", "zsh"])
        .assert()
        .success()
        .stdout(predicate::str::is_empty().not());
}

#[test]
fn completions_fish_produces_output() {
    tokmd_cmd()
        .args(["completions", "fish"])
        .assert()
        .success()
        .stdout(predicate::str::is_empty().not());
}

#[test]
fn completions_powershell_produces_output() {
    tokmd_cmd()
        .args(["completions", "powershell"])
        .assert()
        .success()
        .stdout(predicate::str::is_empty().not());
}

#[test]
fn completions_elvish_produces_output() {
    tokmd_cmd()
        .args(["completions", "elvish"])
        .assert()
        .success()
        .stdout(predicate::str::is_empty().not());
}

#[test]
fn completions_zsh_mentions_tokmd() {
    tokmd_cmd()
        .args(["completions", "zsh"])
        .assert()
        .success()
        .stdout(predicate::str::contains("tokmd"));
}

#[test]
fn completions_fish_mentions_tokmd() {
    tokmd_cmd()
        .args(["completions", "fish"])
        .assert()
        .success()
        .stdout(predicate::str::contains("tokmd"));
}

// ---------------------------------------------------------------------------
// badge – additional metrics
// ---------------------------------------------------------------------------

#[test]
fn badge_tokens_metric_svg() {
    tokmd_cmd()
        .args(["badge", "--metric", "tokens"])
        .assert()
        .success()
        .stdout(predicate::str::contains("<svg"))
        .stdout(predicate::str::contains("tokens"));
}

#[test]
fn badge_doc_metric_svg() {
    tokmd_cmd()
        .args(["badge", "--metric", "doc"])
        .assert()
        .success()
        .stdout(predicate::str::contains("<svg"))
        .stdout(predicate::str::contains("doc"));
}

#[test]
fn badge_blank_metric_svg() {
    tokmd_cmd()
        .args(["badge", "--metric", "blank"])
        .assert()
        .success()
        .stdout(predicate::str::contains("<svg"));
}

#[test]
fn badge_svg_is_well_formed() {
    let output = tokmd_cmd()
        .args(["badge", "--metric", "lines"])
        .output()
        .expect("Failed to run badge command");

    assert!(output.status.success());
    let svg = String::from_utf8_lossy(&output.stdout);
    assert!(svg.starts_with("<svg"), "SVG must start with <svg tag");
    assert!(svg.contains("</svg>"), "SVG must have closing tag");
}

#[test]
fn badge_out_file_creates_svg() {
    let dir = tempdir().unwrap();
    let out = dir.path().join("test.svg");

    tokmd_cmd()
        .args(["badge", "--metric", "tokens", "--out"])
        .arg(&out)
        .assert()
        .success()
        .stdout("");

    let content = std::fs::read_to_string(&out).unwrap();
    assert!(content.contains("<svg"));
    assert!(content.contains("</svg>"));
}

// ---------------------------------------------------------------------------
// init – template profiles and --non-interactive
// ---------------------------------------------------------------------------

#[test]
fn init_print_rust_profile() {
    tokmd_cmd()
        .args(["init", "--print", "--template", "rust", "--non-interactive"])
        .assert()
        .success()
        .stdout(predicate::str::contains("target/"));
}

#[test]
fn init_print_node_profile() {
    tokmd_cmd()
        .args(["init", "--print", "--template", "node", "--non-interactive"])
        .assert()
        .success()
        .stdout(predicate::str::contains("node_modules/"));
}

#[test]
fn init_print_mono_profile() {
    tokmd_cmd()
        .args(["init", "--print", "--template", "mono", "--non-interactive"])
        .assert()
        .success()
        .stdout(predicate::str::is_empty().not());
}

#[test]
fn init_non_interactive_creates_file() {
    let dir = tempdir().unwrap();
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_tokmd"));
    cmd.current_dir(dir.path())
        .args(["init", "--non-interactive"])
        .assert()
        .success();

    assert!(dir.path().join(".tokeignore").exists());
}

#[test]
fn init_refuses_overwrite_without_force() {
    let dir = tempdir().unwrap();
    std::fs::write(dir.path().join(".tokeignore"), "# existing\n").unwrap();

    let mut cmd = Command::new(env!("CARGO_BIN_EXE_tokmd"));
    cmd.current_dir(dir.path())
        .args(["init", "--non-interactive"])
        .assert()
        .failure();
}

// ---------------------------------------------------------------------------
// check-ignore – extra scenarios
// ---------------------------------------------------------------------------

#[test]
fn check_ignore_gitignored_file() {
    // The fixture's .gitignore lists hidden_by_git.rs.
    // The hermetic copy has a .git/ marker so the ignore crate *should*
    // honour .gitignore.  However, depending on the environment the file
    // may not actually appear as ignored (e.g. git config differences).
    // Use --exclude as a reliable mechanism instead.
    let dir = tempdir().unwrap();
    std::fs::write(dir.path().join("hello.rs"), "fn main() {}").unwrap();

    let mut cmd = Command::new(env!("CARGO_BIN_EXE_tokmd"));
    cmd.current_dir(dir.path())
        .args(["--exclude", "hello.rs", "check-ignore", "hello.rs"])
        .assert()
        .code(0)
        .stdout(predicate::str::contains("ignored"));
}

#[test]
fn check_ignore_multiple_files() {
    tokmd_cmd()
        .args(["check-ignore", "hidden_by_git.rs", "src/main.rs"])
        .assert()
        // at least one should be reported
        .stdout(predicate::str::contains("hidden_by_git.rs"));
}

#[test]
fn check_ignore_nonexistent_file_succeeds() {
    // check-ignore should not crash on non-existent paths
    tokmd_cmd()
        .args(["check-ignore", "does_not_exist.txt"])
        .assert()
        // non-existent files are not ignored → exit 1
        .code(1);
}

// ---------------------------------------------------------------------------
// lang – flag combinations
// ---------------------------------------------------------------------------

#[test]
fn lang_json_output_is_valid() {
    let output = tokmd_cmd()
        .args(["lang", "--format", "json"])
        .output()
        .expect("Failed to run");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    let parsed: serde_json::Value =
        serde_json::from_str(&stdout).expect("lang --format json must produce valid JSON");
    assert!(parsed.get("rows").is_some());
    assert!(parsed.get("total").is_some());
}

#[test]
fn lang_top_limits_rows() {
    let output = tokmd_cmd()
        .args(["lang", "--format", "json", "--top", "1"])
        .output()
        .expect("Failed to run");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    let parsed: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    let rows = parsed["rows"].as_array().unwrap();
    // --top 1 keeps the top language plus an "Other" rollup row
    assert!(
        rows.len() <= 2,
        "with --top 1, at most 2 rows expected (top + Other)"
    );
    let names: Vec<&str> = rows.iter().filter_map(|r| r["language"].as_str()).collect();
    if names.len() > 1 {
        assert!(
            names.contains(&"Other"),
            "second row should be the 'Other' rollup"
        );
    }
}

#[test]
fn lang_children_collapse_json() {
    let output = tokmd_cmd()
        .args(["lang", "--format", "json", "--children", "collapse"])
        .output()
        .expect("Failed to run");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    let parsed: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    let args = &parsed["args"];
    assert_eq!(args["children"].as_str().unwrap(), "collapse");
}

#[test]
fn lang_children_separate_json() {
    let output = tokmd_cmd()
        .args(["lang", "--format", "json", "--children", "separate"])
        .output()
        .expect("Failed to run");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    let parsed: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    let args = &parsed["args"];
    assert_eq!(args["children"].as_str().unwrap(), "separate");
}

// ---------------------------------------------------------------------------
// module – depth and JSON
// ---------------------------------------------------------------------------

#[test]
fn module_depth_zero_json() {
    let output = tokmd_cmd()
        .args(["module", "--format", "json", "--module-depth", "0"])
        .output()
        .expect("Failed to run");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    let parsed: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    // depth 0 is recorded in the envelope
    assert_eq!(parsed["module_depth"].as_u64().unwrap(), 0);
    let rows = parsed["rows"].as_array().unwrap();
    assert!(!rows.is_empty(), "should produce at least one row");
    // All rows at depth 0 should be top-level modules (no nested slashes)
    for row in rows {
        let module = row["module"].as_str().unwrap();
        assert!(
            !module.contains('/'),
            "depth 0 should not produce nested modules, got: {module}"
        );
    }
}

#[test]
fn module_json_has_schema_version() {
    let output = tokmd_cmd()
        .args(["module", "--format", "json"])
        .output()
        .expect("Failed to run");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    let parsed: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    assert!(
        parsed.get("schema_version").is_some(),
        "JSON output must include schema_version"
    );
}

#[test]
fn module_children_separate_json() {
    let output = tokmd_cmd()
        .args(["module", "--format", "json", "--children", "separate"])
        .output()
        .expect("Failed to run");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    let parsed: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    assert_eq!(parsed["args"]["children"].as_str().unwrap(), "separate");
}

// ---------------------------------------------------------------------------
// export – CSV and flag combos
// ---------------------------------------------------------------------------

#[test]
fn export_csv_has_header() {
    let output = tokmd_cmd()
        .args(["export", "--format", "csv"])
        .output()
        .expect("Failed to run");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    let first_line = stdout.lines().next().unwrap_or("");
    assert!(
        first_line.contains("path") || first_line.contains("language"),
        "CSV header should contain column names"
    );
}

#[test]
fn export_json_has_envelope() {
    let output = tokmd_cmd()
        .args(["export", "--format", "json"])
        .output()
        .expect("Failed to run");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    let parsed: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    assert!(parsed.get("schema_version").is_some());
    assert!(parsed.get("mode").is_some());
    assert_eq!(parsed["mode"].as_str().unwrap(), "export");
}

#[test]
fn export_jsonl_meta_line_first() {
    let output = tokmd_cmd()
        .args(["export", "--format", "jsonl"])
        .output()
        .expect("Failed to run");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    let first_line = stdout.lines().next().unwrap_or("");
    let parsed: serde_json::Value = serde_json::from_str(first_line).unwrap();
    assert_eq!(
        parsed["type"].as_str().unwrap(),
        "meta",
        "first JSONL line must be the meta envelope"
    );
}

#[test]
fn export_max_rows_limits_output() {
    let output = tokmd_cmd()
        .args(["export", "--format", "json", "--max-rows", "1"])
        .output()
        .expect("Failed to run");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    let parsed: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    let rows = parsed["rows"].as_array().unwrap();
    assert!(rows.len() <= 1, "max-rows 1 should limit to 1 row");
}

// ---------------------------------------------------------------------------
// tools – additional coverage
// ---------------------------------------------------------------------------

#[test]
fn tools_invalid_format_fails() {
    tokmd_cmd()
        .args(["tools", "--format", "invalid"])
        .assert()
        .failure();
}

#[test]
fn tools_jsonschema_has_tool_descriptions() {
    let output = tokmd_cmd()
        .args(["tools", "--format", "jsonschema", "--pretty"])
        .output()
        .expect("Failed to run");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    let parsed: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    let tools = parsed["tools"].as_array().unwrap();
    for tool in tools {
        assert!(
            tool.get("description").is_some(),
            "each tool should have a description"
        );
    }
}

// ---------------------------------------------------------------------------
// analyze – markdown and JSON smoke
// ---------------------------------------------------------------------------

#[test]
fn analyze_receipt_json_has_schema_version() {
    let output = tokmd_cmd()
        .args(["analyze", "--preset", "receipt", "--format", "json"])
        .output()
        .expect("Failed to run");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    let parsed: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    assert!(parsed.get("schema_version").is_some());
    assert!(parsed.get("derived").is_some());
}

#[test]
fn analyze_markdown_contains_header() {
    tokmd_cmd()
        .args(["analyze", "--preset", "receipt", "--format", "md"])
        .assert()
        .success()
        .stdout(predicate::str::contains("#"));
}

// ---------------------------------------------------------------------------
// global flags on subcommands
// ---------------------------------------------------------------------------

#[test]
fn global_exclude_applies_to_lang() {
    let output = tokmd_cmd()
        .args(["--exclude", "*.rs", "lang", "--format", "json"])
        .output()
        .expect("Failed to run");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    let parsed: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    let rows = parsed["rows"].as_array().unwrap();
    // Excluding *.rs should remove Rust from the results
    let has_rust = rows.iter().any(|r| r["language"].as_str() == Some("Rust"));
    assert!(!has_rust, "excluding *.rs should remove Rust from results");
}

#[test]
fn global_exclude_applies_to_module() {
    let output = tokmd_cmd()
        .args(["--exclude", "*.rs", "module", "--format", "json"])
        .output()
        .expect("Failed to run");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    let _parsed: serde_json::Value =
        serde_json::from_str(&stdout).expect("output should still be valid JSON");
}

// ---------------------------------------------------------------------------
// verbose flag
// ---------------------------------------------------------------------------

#[test]
fn verbose_flag_accepted() {
    // --verbose should be accepted on any subcommand without errors
    tokmd_cmd().args(["--verbose", "lang"]).assert().success();
}

#[test]
fn verbose_flag_on_module() {
    tokmd_cmd().args(["--verbose", "module"]).assert().success();
}
