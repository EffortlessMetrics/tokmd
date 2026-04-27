#![cfg(feature = "analysis")]

//! Comprehensive end-to-end CLI integration tests exercising every subcommand
//! and major flag combination.  Each test invokes the real `tokmd` binary
//! against the hermetic fixture directory.

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
// 1. --version / --help
// ===========================================================================

#[test]
fn version_flag_prints_semver() {
    tokmd_cmd()
        .arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::is_match(r"\d+\.\d+\.\d+").unwrap());
}

#[test]
fn help_flag_lists_all_subcommands() {
    tokmd_cmd()
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("lang"))
        .stdout(predicate::str::contains("module"))
        .stdout(predicate::str::contains("export"))
        .stdout(predicate::str::contains("analyze"))
        .stdout(predicate::str::contains("badge"))
        .stdout(predicate::str::contains("run"))
        .stdout(predicate::str::contains("diff"))
        .stdout(predicate::str::contains("context"))
        .stdout(predicate::str::contains("init"))
        .stdout(predicate::str::contains("tools"))
        .stdout(predicate::str::contains("gate"))
        .stdout(predicate::str::contains("completions"))
        .stdout(predicate::str::contains("check-ignore"));
}

#[test]
fn help_text_contains_usage_section() {
    tokmd_cmd()
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Usage"));
}

// ===========================================================================
// 2. Default command (lang) with format variants
// ===========================================================================

#[test]
fn default_command_produces_markdown() {
    tokmd_cmd()
        .assert()
        .success()
        .stdout(predicate::str::contains("Lang"))
        .stdout(predicate::str::contains("Code"));
}

#[test]
fn default_command_format_json() {
    let output = tokmd_cmd()
        .args(["--format", "json"])
        .output()
        .expect("failed to run");

    assert!(output.status.success());
    let json: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(json["mode"], "lang");
    assert!(json["rows"].is_array());
}

#[test]
fn default_command_format_tsv() {
    let output = tokmd_cmd()
        .args(["--format", "tsv"])
        .output()
        .expect("failed to run");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains('\t'), "TSV output must contain tabs");
}

#[test]
fn default_command_format_md() {
    tokmd_cmd()
        .args(["--format", "md"])
        .assert()
        .success()
        .stdout(predicate::str::contains("|"));
}

// ===========================================================================
// 3. lang subcommand
// ===========================================================================

#[test]
fn lang_json_has_schema_version_and_rows() {
    let output = tokmd_cmd()
        .args(["lang", "--format", "json"])
        .output()
        .expect("failed to run");

    assert!(output.status.success());
    let json: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert!(json["schema_version"].is_number());
    assert_eq!(json["mode"], "lang");
    let rows = json["rows"].as_array().expect("rows is array");
    assert!(!rows.is_empty(), "should detect at least one language");
    for row in rows {
        assert!(row["code"].is_number());
        assert!(row["lang"].is_string());
    }
}

#[test]
fn lang_json_has_total() {
    let output = tokmd_cmd()
        .args(["lang", "--format", "json"])
        .output()
        .expect("failed to run");

    assert!(output.status.success());
    let json: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert!(json["total"].is_object());
    assert!(json["total"]["code"].is_number());
    assert!(json["total"]["lines"].is_number());
}

#[test]
fn lang_tsv_has_header_and_data() {
    let output = tokmd_cmd()
        .args(["lang", "--format", "tsv"])
        .output()
        .expect("failed to run");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    let lines: Vec<&str> = stdout.lines().collect();
    assert!(lines.len() >= 2, "TSV should have header + data");
    assert!(lines[0].contains("Lang") || lines[0].contains("language"));
}

#[test]
fn lang_md_renders_table() {
    tokmd_cmd()
        .args(["lang", "--format", "md"])
        .assert()
        .success()
        .stdout(predicate::str::contains("|"))
        .stdout(predicate::str::contains("Lang"))
        .stdout(predicate::str::contains("Code"));
}

#[test]
fn lang_top_flag_limits_rows() {
    let output = tokmd_cmd()
        .args(["lang", "--format", "json", "--top", "1"])
        .output()
        .expect("failed to run");

    assert!(output.status.success());
    let json: Value = serde_json::from_slice(&output.stdout).unwrap();
    let rows = json["rows"].as_array().unwrap();
    assert!(
        rows.len() <= 2,
        "--top 1 should yield at most 2 rows (top + Other)"
    );
}

#[test]
fn lang_children_collapse_records_mode() {
    let output = tokmd_cmd()
        .args(["lang", "--format", "json", "--children", "collapse"])
        .output()
        .expect("failed to run");

    assert!(output.status.success());
    let json: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(json["args"]["children"].as_str().unwrap(), "collapse");
}

#[test]
fn lang_children_separate_records_mode() {
    let output = tokmd_cmd()
        .args(["lang", "--format", "json", "--children", "separate"])
        .output()
        .expect("failed to run");

    assert!(output.status.success());
    let json: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(json["args"]["children"].as_str().unwrap(), "separate");
}

// ===========================================================================
// 4. module subcommand
// ===========================================================================

#[test]
fn module_json_has_rows_and_total() {
    let output = tokmd_cmd()
        .args(["module", "--format", "json"])
        .output()
        .expect("failed to run");

    assert!(output.status.success());
    let json: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(json["mode"], "module");
    assert!(!json["rows"].as_array().unwrap().is_empty());
    assert!(json["total"].is_object());
    assert!(json["total"]["code"].is_number());
}

#[test]
fn module_json_has_schema_version() {
    let output = tokmd_cmd()
        .args(["module", "--format", "json"])
        .output()
        .expect("failed to run");

    assert!(output.status.success());
    let json: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert!(json["schema_version"].is_number());
}

#[test]
fn module_md_renders_table() {
    tokmd_cmd()
        .args(["module", "--format", "md"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Module"))
        .stdout(predicate::str::contains("Code"));
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
    for row in json["rows"].as_array().unwrap() {
        let module = row["module"].as_str().unwrap();
        assert!(
            !module.contains('/'),
            "depth 0 should not produce nested modules, got: {module}"
        );
    }
}

#[test]
fn module_tsv_has_tabs() {
    let output = tokmd_cmd()
        .args(["module", "--format", "tsv"])
        .output()
        .expect("failed to run");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains('\t'), "TSV output must contain tabs");
}

// ===========================================================================
// 5. export subcommand
// ===========================================================================

#[test]
fn export_jsonl_each_line_valid_json() {
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
            .unwrap_or_else(|e| panic!("line {} is not valid JSON: {}", i + 1, e));
    }
}

#[test]
fn export_jsonl_first_line_is_meta() {
    let output = tokmd_cmd()
        .args(["export", "--format", "jsonl"])
        .output()
        .expect("failed to run");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    let first = stdout.lines().next().unwrap();
    let meta: Value = serde_json::from_str(first).unwrap();
    assert_eq!(meta["type"], "meta");
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
    assert!(
        lines.len() >= 2,
        "CSV should have header + at least one row"
    );
    let header = lines[0];
    assert!(
        header.contains("path") || header.contains("language"),
        "CSV header should contain column names"
    );
}

#[test]
fn export_json_has_envelope() {
    let output = tokmd_cmd()
        .args(["export", "--format", "json"])
        .output()
        .expect("failed to run");

    assert!(output.status.success());
    let json: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert!(json["schema_version"].is_number());
    assert_eq!(json["mode"], "export");
    assert!(json["rows"].is_array());
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
    assert!(rows.len() <= 1, "--max-rows 1 should limit to 1 row");
}

// ===========================================================================
// 6. run subcommand
// ===========================================================================

#[test]
fn run_generates_receipt_and_artifacts() {
    let dir = tempdir().unwrap();
    let output_dir = dir.path().join("run_out");

    let mut cmd = Command::new(env!("CARGO_BIN_EXE_tokmd"));
    cmd.current_dir(common::fixture_root())
        .args(["run", "--output-dir"])
        .arg(output_dir.to_str().unwrap())
        .arg(".")
        .assert()
        .success();

    assert!(
        output_dir.join("receipt.json").exists(),
        "receipt.json missing"
    );
    assert!(output_dir.join("lang.json").exists(), "lang.json missing");
    assert!(
        output_dir.join("module.json").exists(),
        "module.json missing"
    );
    assert!(
        output_dir.join("export.jsonl").exists(),
        "export.jsonl missing"
    );

    let receipt: Value =
        serde_json::from_str(&std::fs::read_to_string(output_dir.join("receipt.json")).unwrap())
            .unwrap();
    assert!(receipt["schema_version"].is_number());
}

// ===========================================================================
// 7. analyze subcommand
// ===========================================================================

#[test]
fn analyze_receipt_json_has_derived_metrics() {
    let output = tokmd_cmd()
        .args(["analyze", "--preset", "receipt", "--format", "json"])
        .output()
        .expect("failed to run");

    assert!(output.status.success());
    let json: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert!(json["schema_version"].is_number());
    assert!(json["derived"].is_object(), "should have derived metrics");
}

#[test]
fn analyze_receipt_markdown_contains_heading() {
    tokmd_cmd()
        .args(["analyze", "--preset", "receipt", "--format", "md"])
        .assert()
        .success()
        .stdout(predicate::str::contains("#"));
}

#[test]
fn analyze_health_preset_json() {
    let output = tokmd_cmd()
        .args(["analyze", "--preset", "health", "--format", "json"])
        .output()
        .expect("failed to run");

    assert!(output.status.success());
    let json: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert!(json["schema_version"].is_number());
    assert!(json["derived"].is_object());
}

// ===========================================================================
// 8. badge subcommand
// ===========================================================================

#[test]
fn badge_lines_metric_outputs_svg() {
    tokmd_cmd()
        .args(["badge", "--metric", "lines"])
        .assert()
        .success()
        .stdout(predicate::str::contains("<svg"))
        .stdout(predicate::str::contains("</svg>"));
}

#[test]
fn badge_tokens_metric_outputs_svg() {
    tokmd_cmd()
        .args(["badge", "--metric", "tokens"])
        .assert()
        .success()
        .stdout(predicate::str::contains("<svg"))
        .stdout(predicate::str::contains("tokens"));
}

#[test]
fn badge_bytes_metric_outputs_svg() {
    tokmd_cmd()
        .args(["badge", "--metric", "bytes"])
        .assert()
        .success()
        .stdout(predicate::str::contains("<svg"));
}

#[test]
fn badge_out_flag_writes_file() {
    let dir = tempdir().unwrap();
    let out = dir.path().join("badge.svg");

    tokmd_cmd()
        .args(["badge", "--metric", "lines", "--out"])
        .arg(&out)
        .assert()
        .success()
        .stdout("");

    let content = std::fs::read_to_string(&out).unwrap();
    assert!(content.contains("<svg"));
    assert!(content.contains("</svg>"));
}

// ===========================================================================
// 9. tools subcommand
// ===========================================================================

#[test]
fn tools_openai_format_has_functions() {
    let output = tokmd_cmd()
        .args(["tools", "--format", "openai"])
        .output()
        .expect("failed to run");

    assert!(output.status.success());
    let json: Value = serde_json::from_slice(&output.stdout).unwrap();
    let funcs = json["functions"].as_array().expect("'functions' key");
    assert!(!funcs.is_empty());
    for f in funcs {
        assert!(f["parameters"].is_object());
    }
}

#[test]
fn tools_anthropic_format_has_tools() {
    let output = tokmd_cmd()
        .args(["tools", "--format", "anthropic"])
        .output()
        .expect("failed to run");

    assert!(output.status.success());
    let json: Value = serde_json::from_slice(&output.stdout).unwrap();
    let tools = json["tools"].as_array().expect("'tools' key");
    assert!(!tools.is_empty());
    for t in tools {
        assert!(t["input_schema"].is_object());
    }
}

#[test]
fn tools_jsonschema_format_has_schema_version() {
    let output = tokmd_cmd()
        .args(["tools", "--format", "jsonschema"])
        .output()
        .expect("failed to run");

    assert!(output.status.success());
    let json: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert!(json["schema_version"].is_number());
    assert!(json["tools"].is_array());
}

#[test]
fn tools_pretty_flag_adds_whitespace() {
    let compact = tokmd_cmd()
        .args(["tools", "--format", "jsonschema"])
        .output()
        .expect("compact");
    let pretty = tokmd_cmd()
        .args(["tools", "--format", "jsonschema", "--pretty"])
        .output()
        .expect("pretty");

    assert!(pretty.stdout.len() > compact.stdout.len());
}

// ===========================================================================
// 10. context subcommand
// ===========================================================================

#[test]
fn context_default_mode_lists_files() {
    tokmd_cmd()
        .arg("context")
        .assert()
        .success()
        .stdout(predicate::str::is_empty().not());
}

#[test]
fn context_list_mode_includes_source_file() {
    tokmd_cmd()
        .args(["context", "--mode", "list"])
        .assert()
        .success()
        .stdout(predicate::str::contains("src/main.rs"));
}

#[test]
fn context_json_mode_produces_valid_json() {
    let output = tokmd_cmd()
        .args(["context", "--mode", "json"])
        .output()
        .expect("failed to run");

    assert!(output.status.success());
    let json: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert!(json.is_object());
}

// ===========================================================================
// 11. init subcommand
// ===========================================================================

#[test]
fn init_print_outputs_tokeignore_template() {
    tokmd_cmd()
        .args(["init", "--print", "--non-interactive"])
        .assert()
        .success()
        .stdout(predicate::str::is_empty().not());
}

#[test]
fn init_print_rust_template_contains_target() {
    tokmd_cmd()
        .args(["init", "--print", "--template", "rust", "--non-interactive"])
        .assert()
        .success()
        .stdout(predicate::str::contains("target/"));
}

#[test]
fn init_print_node_template_contains_node_modules() {
    tokmd_cmd()
        .args(["init", "--print", "--template", "node", "--non-interactive"])
        .assert()
        .success()
        .stdout(predicate::str::contains("node_modules/"));
}

#[test]
fn init_non_interactive_creates_tokeignore_file() {
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

// ===========================================================================
// 12. check-ignore subcommand
// ===========================================================================

#[test]
fn check_ignore_with_excluded_file_reports_ignored() {
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
fn check_ignore_nonexistent_file_exits_nonzero() {
    tokmd_cmd()
        .args(["check-ignore", "does_not_exist.txt"])
        .assert()
        .code(1);
}

// ===========================================================================
// 13. completions subcommand
// ===========================================================================

#[test]
fn completions_bash_produces_script() {
    tokmd_cmd()
        .args(["completions", "bash"])
        .assert()
        .success()
        .stdout(predicate::str::is_empty().not());
}

#[test]
fn completions_zsh_produces_script() {
    tokmd_cmd()
        .args(["completions", "zsh"])
        .assert()
        .success()
        .stdout(predicate::str::is_empty().not());
}

#[test]
fn completions_fish_produces_script() {
    tokmd_cmd()
        .args(["completions", "fish"])
        .assert()
        .success()
        .stdout(predicate::str::is_empty().not());
}

#[test]
fn completions_powershell_produces_script() {
    tokmd_cmd()
        .args(["completions", "powershell"])
        .assert()
        .success()
        .stdout(predicate::str::is_empty().not());
}

#[test]
fn completions_elvish_produces_script() {
    tokmd_cmd()
        .args(["completions", "elvish"])
        .assert()
        .success()
        .stdout(predicate::str::is_empty().not());
}

// ===========================================================================
// 14. baseline subcommand
// ===========================================================================

#[test]
fn baseline_generates_valid_json_output() {
    let dir = tempdir().unwrap();
    let out = dir.path().join("baseline.json");

    let mut cmd = Command::new(env!("CARGO_BIN_EXE_tokmd"));
    cmd.current_dir(common::fixture_root())
        .args(["--no-progress", "baseline", "--output"])
        .arg(&out)
        .arg("--force")
        .assert()
        .success();

    let json: Value = serde_json::from_str(&std::fs::read_to_string(&out).unwrap()).unwrap();
    assert_eq!(json["baseline_version"].as_u64(), Some(1));
    assert!(json.get("metrics").is_some());
}

// ===========================================================================
// 15. diff subcommand (requires two runs)
// ===========================================================================

#[test]
fn diff_between_identical_runs_shows_no_changes() {
    let dir = tempdir().unwrap();
    let run1 = dir.path().join("r1");
    let run2 = dir.path().join("r2");

    // Produce two identical runs
    for out_dir in [&run1, &run2] {
        let mut cmd = Command::new(env!("CARGO_BIN_EXE_tokmd"));
        cmd.current_dir(common::fixture_root())
            .args(["run", "--output-dir"])
            .arg(out_dir.to_str().unwrap())
            .arg(".")
            .assert()
            .success();
    }

    let mut cmd = Command::new(env!("CARGO_BIN_EXE_tokmd"));
    let output = cmd
        .args([
            "diff",
            "--from",
            run1.join("lang.json").to_str().unwrap(),
            "--to",
            run2.join("lang.json").to_str().unwrap(),
            "--format",
            "json",
        ])
        .output()
        .expect("failed to run diff");

    assert!(output.status.success());
    let json: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert!(json.is_object());
}

// ===========================================================================
// 16. Global flag interactions
// ===========================================================================

#[test]
fn exclude_flag_removes_rust_from_lang() {
    let output = tokmd_cmd()
        .args(["--exclude", "*.rs", "lang", "--format", "json"])
        .output()
        .expect("failed to run");

    assert!(output.status.success());
    let json: Value = serde_json::from_slice(&output.stdout).unwrap();
    let has_rust = json["rows"]
        .as_array()
        .unwrap()
        .iter()
        .any(|r| r["lang"].as_str() == Some("Rust"));
    assert!(!has_rust, "excluding *.rs should remove Rust");
}

#[test]
fn verbose_flag_accepted_on_lang() {
    tokmd_cmd().args(["--verbose", "lang"]).assert().success();
}

#[test]
fn verbose_flag_accepted_on_module() {
    tokmd_cmd().args(["--verbose", "module"]).assert().success();
}

#[test]
fn no_progress_flag_accepted() {
    tokmd_cmd()
        .args(["--no-progress", "lang"])
        .assert()
        .success();
}

// ===========================================================================
// 17. Error cases
// ===========================================================================

#[test]
fn invalid_subcommand_fails() {
    tokmd_cmd()
        .arg("this-subcommand-does-not-exist")
        .assert()
        .failure()
        .stderr(predicate::str::is_empty().not());
}

#[test]
fn lang_invalid_format_fails() {
    tokmd_cmd()
        .args(["lang", "--format", "invalid_fmt"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("invalid value"));
}

#[test]
fn module_invalid_format_fails() {
    tokmd_cmd()
        .args(["module", "--format", "invalid_fmt"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("invalid value"));
}

#[test]
fn export_invalid_format_fails() {
    tokmd_cmd()
        .args(["export", "--format", "invalid_fmt"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("invalid value"));
}

#[test]
fn analyze_invalid_format_fails() {
    tokmd_cmd()
        .args(["analyze", "--format", "invalid_fmt"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("invalid value"));
}

#[test]
fn analyze_invalid_preset_fails() {
    tokmd_cmd()
        .args(["analyze", "--preset", "nonexistent"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("invalid value"));
}

#[test]
fn tools_invalid_format_fails() {
    tokmd_cmd()
        .args(["tools", "--format", "invalid_fmt"])
        .assert()
        .failure();
}

#[test]
fn unknown_flag_on_lang_fails() {
    tokmd_cmd()
        .args(["lang", "--this-flag-does-not-exist"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("unexpected argument"));
}

#[test]
fn lang_invalid_children_mode_fails() {
    tokmd_cmd()
        .args(["lang", "--children", "invalid_mode"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("invalid value"));
}

#[test]
fn gate_missing_args_fails() {
    tokmd_cmd()
        .arg("gate")
        .assert()
        .failure()
        .stderr(predicate::str::is_empty().not());
}

#[test]
fn diff_missing_args_fails() {
    tokmd_cmd()
        .arg("diff")
        .assert()
        .failure()
        .stderr(predicate::str::is_empty().not());
}

#[test]
fn diff_nonexistent_files_fails() {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_tokmd"));
    cmd.args([
        "diff",
        "--from",
        "/tmp/no_such_file_a.json",
        "--to",
        "/tmp/no_such_file_b.json",
    ])
    .assert()
    .failure()
    .stderr(predicate::str::is_empty().not());
}

// ===========================================================================
// 18. Subcommand --help flags
// ===========================================================================

#[test]
fn lang_help_shows_format() {
    tokmd_cmd()
        .args(["lang", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("--format"));
}

#[test]
fn module_help_shows_depth() {
    tokmd_cmd()
        .args(["module", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("--module-depth"));
}

#[test]
fn export_help_shows_format() {
    tokmd_cmd()
        .args(["export", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("--format"));
}

#[test]
fn analyze_help_shows_preset() {
    tokmd_cmd()
        .args(["analyze", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("--preset"));
}

#[test]
fn badge_help_shows_metric() {
    tokmd_cmd()
        .args(["badge", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("--metric"));
}

#[test]
fn context_help_shows_mode() {
    tokmd_cmd()
        .args(["context", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("--mode"));
}

// ===========================================================================
// 19. Empty directory behavior
// ===========================================================================

#[test]
fn lang_empty_dir_succeeds_with_zero_totals() {
    let dir = tempdir().unwrap();
    std::fs::create_dir_all(dir.path().join(".git")).unwrap();

    let mut cmd = Command::new(env!("CARGO_BIN_EXE_tokmd"));
    cmd.current_dir(dir.path())
        .arg("lang")
        .assert()
        .success()
        .stdout(predicate::str::contains("|**Total**|0|0|0|0|"));
}

#[test]
fn module_empty_dir_succeeds_with_zero_totals() {
    let dir = tempdir().unwrap();
    std::fs::create_dir_all(dir.path().join(".git")).unwrap();

    let mut cmd = Command::new(env!("CARGO_BIN_EXE_tokmd"));
    cmd.current_dir(dir.path())
        .arg("module")
        .assert()
        .success()
        .stdout(predicate::str::contains("|**Total**|0|0|0|0|0|0|"));
}

#[test]
fn export_empty_dir_produces_meta_only() {
    let dir = tempdir().unwrap();
    std::fs::create_dir_all(dir.path().join(".git")).unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_tokmd"))
        .current_dir(dir.path())
        .arg("export")
        .output()
        .expect("run export");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    let lines: Vec<&str> = stdout.lines().collect();
    assert_eq!(lines.len(), 1, "expected only the meta record");
    assert!(lines[0].contains(r#""type":"meta""#));
}
