#![cfg(feature = "analysis")]

//! Wave 42 — comprehensive CLI E2E tests covering all major subcommands,
//! flag combinations, error cases, and edge conditions.

mod common;

use assert_cmd::Command;
use predicates::prelude::*;
use serde_json::Value;
use tempfile::tempdir;

fn tokmd_cmd() -> Command {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_tokmd"));
    cmd.current_dir(common::fixture_root());
    cmd
}

// ===========================================================================
// 1. tokmd lang (default) — produces output, exits 0
// ===========================================================================

#[test]
fn lang_default_produces_markdown_output() {
    tokmd_cmd()
        .arg("lang")
        .assert()
        .success()
        .stdout(predicate::str::is_empty().not())
        .stdout(predicate::str::contains("Lang"))
        .stdout(predicate::str::contains("Code"));
}

// ===========================================================================
// 2. tokmd lang --format json — valid JSON
// ===========================================================================

#[test]
fn lang_json_produces_valid_json() {
    let output = tokmd_cmd()
        .args(["lang", "--format", "json"])
        .output()
        .expect("failed to run");

    assert!(output.status.success());
    let json: Value =
        serde_json::from_slice(&output.stdout).expect("lang --format json must produce valid JSON");
    assert!(json["schema_version"].is_number());
    assert_eq!(json["mode"], "lang");
    assert!(json["rows"].is_array());
    assert!(json["total"].is_object());
}

// ===========================================================================
// 3. tokmd module — produces output
// ===========================================================================

#[test]
fn module_default_produces_output() {
    tokmd_cmd()
        .arg("module")
        .assert()
        .success()
        .stdout(predicate::str::is_empty().not())
        .stdout(predicate::str::contains("Module"))
        .stdout(predicate::str::contains("Code"));
}

// ===========================================================================
// 4. tokmd module --format json — valid JSON
// ===========================================================================

#[test]
fn module_json_produces_valid_json() {
    let output = tokmd_cmd()
        .args(["module", "--format", "json"])
        .output()
        .expect("failed to run");

    assert!(output.status.success());
    let json: Value = serde_json::from_slice(&output.stdout)
        .expect("module --format json must produce valid JSON");
    assert!(json["schema_version"].is_number());
    assert_eq!(json["mode"], "module");
    assert!(json["rows"].is_array());
    let rows = json["rows"].as_array().unwrap();
    assert!(!rows.is_empty(), "should have at least one module row");
}

// ===========================================================================
// 5. tokmd export --format jsonl — produces JSONL lines
// ===========================================================================

#[test]
fn export_jsonl_produces_valid_lines() {
    let output = tokmd_cmd()
        .args(["export", "--format", "jsonl"])
        .output()
        .expect("failed to run");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    let lines: Vec<&str> = stdout.lines().filter(|l| !l.trim().is_empty()).collect();
    assert!(lines.len() >= 2, "should have meta + at least one data row");
    for (i, line) in lines.iter().enumerate() {
        let _: Value = serde_json::from_str(line)
            .unwrap_or_else(|e| panic!("JSONL line {} is not valid JSON: {}", i + 1, e));
    }
}

// ===========================================================================
// 6. tokmd badge — produces SVG
// ===========================================================================

#[test]
fn badge_produces_svg() {
    tokmd_cmd()
        .args(["badge", "--metric", "lines"])
        .assert()
        .success()
        .stdout(predicate::str::contains("<svg"))
        .stdout(predicate::str::contains("</svg>"));
}

// ===========================================================================
// 7. tokmd init — generates .tokeignore (use tempdir)
// ===========================================================================

#[test]
fn init_creates_tokeignore_in_tempdir() {
    let dir = tempdir().unwrap();
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_tokmd"));
    cmd.current_dir(dir.path())
        .args(["init", "--non-interactive"])
        .assert()
        .success();

    assert!(
        dir.path().join(".tokeignore").exists(),
        ".tokeignore should be created"
    );
    let content = std::fs::read_to_string(dir.path().join(".tokeignore")).unwrap();
    assert!(!content.is_empty(), ".tokeignore should not be empty");
}

// ===========================================================================
// 8. tokmd tools --format openai — produces tool definitions
// ===========================================================================

#[test]
fn tools_openai_produces_valid_json_with_functions() {
    let output = tokmd_cmd()
        .args(["tools", "--format", "openai"])
        .output()
        .expect("failed to run");

    assert!(output.status.success());
    let json: Value = serde_json::from_slice(&output.stdout)
        .expect("tools --format openai must produce valid JSON");
    assert!(
        json.get("functions").is_some(),
        "OpenAI format should have 'functions' key"
    );
}

// ===========================================================================
// 9. tokmd completions bash — produces shell completions
// ===========================================================================

#[test]
fn completions_bash_produces_output() {
    tokmd_cmd()
        .args(["completions", "bash"])
        .assert()
        .success()
        .stdout(predicate::str::is_empty().not());
}

// ===========================================================================
// 10. tokmd --help — shows help text
// ===========================================================================

#[test]
fn help_shows_subcommands() {
    tokmd_cmd()
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("lang"))
        .stdout(predicate::str::contains("module"))
        .stdout(predicate::str::contains("export"))
        .stdout(predicate::str::contains("badge"))
        .stdout(predicate::str::contains("init"));
}

// ===========================================================================
// 11. tokmd --version — shows version
// ===========================================================================

#[test]
fn version_shows_semver() {
    tokmd_cmd()
        .arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::is_match(r"\d+\.\d+\.\d+").unwrap());
}

// ===========================================================================
// 12. tokmd lang --format tsv — produces TSV output
// ===========================================================================

#[test]
fn lang_tsv_produces_tab_separated_output() {
    let output = tokmd_cmd()
        .args(["lang", "--format", "tsv"])
        .output()
        .expect("failed to run");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(
        stdout.contains('\t'),
        "TSV output should contain tab characters"
    );
    let lines: Vec<&str> = stdout.lines().collect();
    assert!(lines.len() >= 2, "TSV should have header + data rows");
}

// ===========================================================================
// 13. tokmd run --output-dir — produces run receipt
// ===========================================================================

#[test]
fn run_produces_receipt_json() {
    let dir = tempdir().unwrap();
    let output_dir = dir.path().join("run_out");

    tokmd_cmd()
        .args(["run", "--output-dir"])
        .arg(output_dir.to_str().unwrap())
        .arg(".")
        .assert()
        .success();

    assert!(
        output_dir.join("receipt.json").exists(),
        "receipt.json should exist"
    );
    let content = std::fs::read_to_string(output_dir.join("receipt.json")).unwrap();
    let json: Value = serde_json::from_str(&content).expect("receipt.json must be valid JSON");
    assert!(json["schema_version"].is_number());
}

// ===========================================================================
// Error case: tokmd lang --format invalid — exits non-zero
// ===========================================================================

#[test]
fn lang_invalid_format_exits_nonzero() {
    tokmd_cmd()
        .args(["lang", "--format", "invalid"])
        .assert()
        .failure();
}

// ===========================================================================
// Error case: tokmd on empty directory — handles gracefully
// ===========================================================================

#[test]
fn empty_dir_handles_gracefully() {
    let dir = tempdir().unwrap();
    std::fs::create_dir_all(dir.path().join(".git")).unwrap();

    let mut cmd = Command::new(env!("CARGO_BIN_EXE_tokmd"));
    cmd.current_dir(dir.path()).arg("lang").assert().success();
}

// ===========================================================================
// Additional tests (16–35+)
// ===========================================================================

#[test]
fn lang_json_rows_have_code_field() {
    let output = tokmd_cmd()
        .args(["lang", "--format", "json"])
        .output()
        .expect("failed to run");

    assert!(output.status.success());
    let json: Value = serde_json::from_slice(&output.stdout).unwrap();
    let rows = json["rows"].as_array().expect("rows should be an array");
    assert!(!rows.is_empty());
    for row in rows {
        assert!(row["code"].is_number(), "each row should have a code count");
    }
}

#[test]
fn lang_json_total_has_code_and_lines() {
    let output = tokmd_cmd()
        .args(["lang", "--format", "json"])
        .output()
        .expect("failed to run");

    assert!(output.status.success());
    let json: Value = serde_json::from_slice(&output.stdout).unwrap();
    let total = &json["total"];
    assert!(total["code"].is_number());
    assert!(total["lines"].is_number());
}

#[test]
fn module_json_total_present() {
    let output = tokmd_cmd()
        .args(["module", "--format", "json"])
        .output()
        .expect("failed to run");

    assert!(output.status.success());
    let json: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert!(json["total"].is_object(), "total should be present");
    assert!(json["total"]["code"].is_number());
}

#[test]
fn export_csv_has_header_and_rows() {
    let output = tokmd_cmd()
        .args(["export", "--format", "csv"])
        .output()
        .expect("failed to run");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    let lines: Vec<&str> = stdout.lines().collect();
    assert!(lines.len() >= 2, "should have header + at least one row");
    assert!(
        lines[0].contains("path") || lines[0].contains("language"),
        "CSV header should contain column names"
    );
}

#[test]
fn export_jsonl_first_line_is_meta() {
    let output = tokmd_cmd()
        .args(["export", "--format", "jsonl"])
        .output()
        .expect("failed to run");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    let first_line = stdout.lines().next().unwrap();
    let parsed: Value = serde_json::from_str(first_line).unwrap();
    assert_eq!(parsed["type"].as_str().unwrap(), "meta");
}

#[test]
fn badge_tokens_metric_contains_tokens() {
    tokmd_cmd()
        .args(["badge", "--metric", "tokens"])
        .assert()
        .success()
        .stdout(predicate::str::contains("<svg"))
        .stdout(predicate::str::contains("tokens"));
}

#[test]
fn badge_bytes_metric_contains_bytes() {
    tokmd_cmd()
        .args(["badge", "--metric", "bytes"])
        .assert()
        .success()
        .stdout(predicate::str::contains("<svg"))
        .stdout(predicate::str::contains("bytes"));
}

#[test]
fn init_print_outputs_to_stdout() {
    tokmd_cmd()
        .args(["init", "--print", "--non-interactive"])
        .assert()
        .success()
        .stdout(predicate::str::is_empty().not());
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

#[test]
fn tools_anthropic_produces_valid_json() {
    let output = tokmd_cmd()
        .args(["tools", "--format", "anthropic"])
        .output()
        .expect("failed to run");

    assert!(output.status.success());
    let json: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert!(
        json.get("tools").is_some(),
        "Anthropic format should have 'tools' key"
    );
}

#[test]
fn tools_jsonschema_produces_valid_json() {
    let output = tokmd_cmd()
        .args(["tools", "--format", "jsonschema"])
        .output()
        .expect("failed to run");

    assert!(output.status.success());
    let json: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert!(json.get("tools").is_some());
    assert!(json.get("schema_version").is_some());
}

#[test]
fn tools_invalid_format_fails() {
    tokmd_cmd()
        .args(["tools", "--format", "invalid"])
        .assert()
        .failure();
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
fn lang_top_limits_rows() {
    let output = tokmd_cmd()
        .args(["lang", "--format", "json", "--top", "1"])
        .output()
        .expect("failed to run");

    assert!(output.status.success());
    let json: Value = serde_json::from_slice(&output.stdout).unwrap();
    let rows = json["rows"].as_array().unwrap();
    assert!(
        rows.len() <= 2,
        "with --top 1, at most 2 rows expected (top + Other)"
    );
}

#[test]
fn lang_children_collapse_recorded_in_json() {
    let output = tokmd_cmd()
        .args(["lang", "--format", "json", "--children", "collapse"])
        .output()
        .expect("failed to run");

    assert!(output.status.success());
    let json: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(json["args"]["children"].as_str().unwrap(), "collapse");
}

#[test]
fn module_depth_zero_produces_top_level_only() {
    let output = tokmd_cmd()
        .args(["module", "--format", "json", "--module-depth", "0"])
        .output()
        .expect("failed to run");

    assert!(output.status.success());
    let json: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(json["module_depth"].as_u64().unwrap(), 0);
    let rows = json["rows"].as_array().unwrap();
    for row in rows {
        let module = row["module"].as_str().unwrap();
        assert!(
            !module.contains('/'),
            "depth 0 should not produce nested modules, got: {module}"
        );
    }
}

#[test]
fn analyze_receipt_json_has_derived_metrics() {
    let output = tokmd_cmd()
        .args(["analyze", "--preset", "receipt", "--format", "json"])
        .output()
        .expect("failed to run");

    assert!(output.status.success());
    let json: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert!(json["schema_version"].is_number());
    assert!(json["derived"].is_object());
}

#[test]
fn analyze_receipt_markdown_contains_header() {
    tokmd_cmd()
        .args(["analyze", "--preset", "receipt", "--format", "md"])
        .assert()
        .success()
        .stdout(predicate::str::contains("#"));
}

#[test]
fn export_json_has_envelope_fields() {
    let output = tokmd_cmd()
        .args(["export", "--format", "json"])
        .output()
        .expect("failed to run");

    assert!(output.status.success());
    let json: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert!(json.get("schema_version").is_some());
    assert_eq!(json["mode"].as_str().unwrap(), "export");
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

#[test]
fn global_exclude_removes_rust_from_lang() {
    let output = tokmd_cmd()
        .args(["--exclude", "*.rs", "lang", "--format", "json"])
        .output()
        .expect("failed to run");

    assert!(output.status.success());
    let json: Value = serde_json::from_slice(&output.stdout).unwrap();
    let rows = json["rows"].as_array().unwrap();
    let has_rust = rows.iter().any(|r| r["language"].as_str() == Some("Rust"));
    assert!(!has_rust, "excluding *.rs should remove Rust from results");
}

#[test]
fn run_generates_all_artifacts() {
    let dir = tempdir().unwrap();
    let output_dir = dir.path().join("run_all");

    tokmd_cmd()
        .args(["run", "--output-dir"])
        .arg(output_dir.to_str().unwrap())
        .arg(".")
        .assert()
        .success();

    assert!(output_dir.join("receipt.json").exists());
    assert!(output_dir.join("lang.json").exists());
    assert!(output_dir.join("module.json").exists());
    assert!(output_dir.join("export.jsonl").exists());
}

#[test]
fn badge_out_file_creates_svg_file() {
    let dir = tempdir().unwrap();
    let out = dir.path().join("badge.svg");

    tokmd_cmd()
        .args(["badge", "--metric", "tokens", "--out"])
        .arg(&out)
        .assert()
        .success();

    let content = std::fs::read_to_string(&out).unwrap();
    assert!(content.contains("<svg"));
    assert!(content.contains("</svg>"));
}

#[test]
fn default_command_is_lang() {
    let output_default = tokmd_cmd().output().expect("failed to run default");
    let output_lang = tokmd_cmd()
        .arg("lang")
        .output()
        .expect("failed to run lang");

    assert!(output_default.status.success());
    assert!(output_lang.status.success());

    let default_stdout = String::from_utf8(output_default.stdout).unwrap();
    let lang_stdout = String::from_utf8(output_lang.stdout).unwrap();

    // Both should contain the same structural markers
    assert!(default_stdout.contains("Lang"));
    assert!(lang_stdout.contains("Lang"));
}

#[test]
fn verbose_flag_accepted_on_lang() {
    tokmd_cmd().args(["--verbose", "lang"]).assert().success();
}

#[test]
fn export_max_rows_limits_output() {
    let output = tokmd_cmd()
        .args(["export", "--format", "json", "--max-rows", "1"])
        .output()
        .expect("failed to run");

    assert!(output.status.success());
    let json: Value = serde_json::from_slice(&output.stdout).unwrap();
    let rows = json["rows"].as_array().unwrap();
    assert!(rows.len() <= 1, "max-rows 1 should limit to 1 row");
}

#[test]
fn empty_dir_module_handles_gracefully() {
    let dir = tempdir().unwrap();
    std::fs::create_dir_all(dir.path().join(".git")).unwrap();

    let mut cmd = Command::new(env!("CARGO_BIN_EXE_tokmd"));
    cmd.current_dir(dir.path()).arg("module").assert().success();
}

#[test]
fn empty_dir_export_handles_gracefully() {
    let dir = tempdir().unwrap();
    std::fs::create_dir_all(dir.path().join(".git")).unwrap();

    let mut cmd = Command::new(env!("CARGO_BIN_EXE_tokmd"));
    cmd.current_dir(dir.path())
        .args(["export", "--format", "jsonl"])
        .assert()
        .success();
}
