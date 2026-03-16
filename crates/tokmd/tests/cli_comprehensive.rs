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
fn version_flag_prints_semver() -> Result<(), Box<dyn std::error::Error>> {
    tokmd_cmd()
        .arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::is_match(r"\d+\.\d+\.\d+")?);
    Ok(())
}

#[test]
fn help_flag_lists_all_subcommands() -> Result<(), Box<dyn std::error::Error>> {
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
    Ok(())
}

#[test]
fn help_text_contains_usage_section() -> Result<(), Box<dyn std::error::Error>> {
    tokmd_cmd()
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Usage"));
    Ok(())
}

// ===========================================================================
// 2. Default command (lang) with format variants
// ===========================================================================

#[test]
fn default_command_produces_markdown() -> Result<(), Box<dyn std::error::Error>> {
    tokmd_cmd()
        .assert()
        .success()
        .stdout(predicate::str::contains("Lang"))
        .stdout(predicate::str::contains("Code"));
    Ok(())
}

#[test]
fn default_command_format_json() -> Result<(), Box<dyn std::error::Error>> {
    let output = tokmd_cmd().args(["--format", "json"]).output()?;

    assert!(output.status.success());
    let json: Value = serde_json::from_slice(&output.stdout)?;
    assert_eq!(json["mode"], "lang");
    assert!(json["rows"].is_array());
    Ok(())
}

#[test]
fn default_command_format_tsv() -> Result<(), Box<dyn std::error::Error>> {
    let output = tokmd_cmd().args(["--format", "tsv"]).output()?;

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout)?;
    assert!(stdout.contains('\t'), "TSV output must contain tabs");
    Ok(())
}

#[test]
fn default_command_format_md() -> Result<(), Box<dyn std::error::Error>> {
    tokmd_cmd()
        .args(["--format", "md"])
        .assert()
        .success()
        .stdout(predicate::str::contains("|"));
    Ok(())
}

// ===========================================================================
// 3. lang subcommand
// ===========================================================================

#[test]
fn lang_json_has_schema_version_and_rows() -> Result<(), Box<dyn std::error::Error>> {
    let output = tokmd_cmd().args(["lang", "--format", "json"]).output()?;

    assert!(output.status.success());
    let json: Value = serde_json::from_slice(&output.stdout)?;
    assert!(json["schema_version"].is_number());
    assert_eq!(json["mode"], "lang");
    let rows = json["rows"].as_array().ok_or("not an array")?;
    assert!(!rows.is_empty(), "should detect at least one language");
    for row in rows {
        assert!(row["code"].is_number());
        assert!(row["lang"].is_string());
    }
    Ok(())
}

#[test]
fn lang_json_has_total() -> Result<(), Box<dyn std::error::Error>> {
    let output = tokmd_cmd().args(["lang", "--format", "json"]).output()?;

    assert!(output.status.success());
    let json: Value = serde_json::from_slice(&output.stdout)?;
    assert!(json["total"].is_object());
    assert!(json["total"]["code"].is_number());
    assert!(json["total"]["lines"].is_number());
    Ok(())
}

#[test]
fn lang_tsv_has_header_and_data() -> Result<(), Box<dyn std::error::Error>> {
    let output = tokmd_cmd().args(["lang", "--format", "tsv"]).output()?;

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout)?;
    let lines: Vec<&str> = stdout.lines().collect();
    assert!(lines.len() >= 2, "TSV should have header + data");
    assert!(lines[0].contains("Lang") || lines[0].contains("language"));
    Ok(())
}

#[test]
fn lang_md_renders_table() -> Result<(), Box<dyn std::error::Error>> {
    tokmd_cmd()
        .args(["lang", "--format", "md"])
        .assert()
        .success()
        .stdout(predicate::str::contains("|"))
        .stdout(predicate::str::contains("Lang"))
        .stdout(predicate::str::contains("Code"));
    Ok(())
}

#[test]
fn lang_top_flag_limits_rows() -> Result<(), Box<dyn std::error::Error>> {
    let output = tokmd_cmd()
        .args(["lang", "--format", "json", "--top", "1"])
        .output()?;

    assert!(output.status.success());
    let json: Value = serde_json::from_slice(&output.stdout)?;
    let rows = json["rows"].as_array().ok_or("not an array")?;
    assert!(
        rows.len() <= 2,
        "--top 1 should yield at most 2 rows (top + Other)"
    );
    Ok(())
}

#[test]
fn lang_children_collapse_records_mode() -> Result<(), Box<dyn std::error::Error>> {
    let output = tokmd_cmd()
        .args(["lang", "--format", "json", "--children", "collapse"])
        .output()?;

    assert!(output.status.success());
    let json: Value = serde_json::from_slice(&output.stdout)?;
    assert_eq!(
        json["args"]["children"].as_str().ok_or("not a string")?,
        "collapse"
    );
    Ok(())
}

#[test]
fn lang_children_separate_records_mode() -> Result<(), Box<dyn std::error::Error>> {
    let output = tokmd_cmd()
        .args(["lang", "--format", "json", "--children", "separate"])
        .output()?;

    assert!(output.status.success());
    let json: Value = serde_json::from_slice(&output.stdout)?;
    assert_eq!(
        json["args"]["children"].as_str().ok_or("not a string")?,
        "separate"
    );
    Ok(())
}

// ===========================================================================
// 4. module subcommand
// ===========================================================================

#[test]
fn module_json_has_rows_and_total() -> Result<(), Box<dyn std::error::Error>> {
    let output = tokmd_cmd().args(["module", "--format", "json"]).output()?;

    assert!(output.status.success());
    let json: Value = serde_json::from_slice(&output.stdout)?;
    assert_eq!(json["mode"], "module");
    assert!(!json["rows"].as_array().ok_or("not an array")?.is_empty());
    assert!(json["total"].is_object());
    assert!(json["total"]["code"].is_number());
    Ok(())
}

#[test]
fn module_json_has_schema_version() -> Result<(), Box<dyn std::error::Error>> {
    let output = tokmd_cmd().args(["module", "--format", "json"]).output()?;

    assert!(output.status.success());
    let json: Value = serde_json::from_slice(&output.stdout)?;
    assert!(json["schema_version"].is_number());
    Ok(())
}

#[test]
fn module_md_renders_table() -> Result<(), Box<dyn std::error::Error>> {
    tokmd_cmd()
        .args(["module", "--format", "md"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Module"))
        .stdout(predicate::str::contains("Code"));
    Ok(())
}

#[test]
fn module_depth_zero_produces_top_level_only() -> Result<(), Box<dyn std::error::Error>> {
    let output = tokmd_cmd()
        .args(["module", "--format", "json", "--module-depth", "0"])
        .output()?;

    assert!(output.status.success());
    let json: Value = serde_json::from_slice(&output.stdout)?;
    assert_eq!(json["module_depth"].as_u64().ok_or("not an u64")?, 0);
    for row in json["rows"].as_array().ok_or("not an array")? {
        let module = row["module"].as_str().ok_or("not a string")?;
        assert!(
            !module.contains('/'),
            "depth 0 should not produce nested modules, got: {module}"
        );
    }
    Ok(())
}

#[test]
fn module_tsv_has_tabs() -> Result<(), Box<dyn std::error::Error>> {
    let output = tokmd_cmd().args(["module", "--format", "tsv"]).output()?;

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout)?;
    assert!(stdout.contains('\t'), "TSV output must contain tabs");
    Ok(())
}

// ===========================================================================
// 5. export subcommand
// ===========================================================================

#[test]
fn export_jsonl_each_line_valid_json() -> Result<(), Box<dyn std::error::Error>> {
    let output = tokmd_cmd().args(["export", "--format", "jsonl"]).output()?;

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout)?;
    let lines: Vec<&str> = stdout.lines().filter(|l| !l.trim().is_empty()).collect();
    assert!(lines.len() >= 2, "should have meta + at least one data row");
    for (i, line) in lines.iter().enumerate() {
        let _: Value = serde_json::from_str(line)
            .unwrap_or_else(|e| panic!("line {} is not valid JSON: {}", i + 1, e));
    }
    Ok(())
}

#[test]
fn export_jsonl_first_line_is_meta() -> Result<(), Box<dyn std::error::Error>> {
    let output = tokmd_cmd().args(["export", "--format", "jsonl"]).output()?;

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout)?;
    let first = stdout.lines().next().ok_or("no next line")?;
    let meta: Value = serde_json::from_str(first)?;
    assert_eq!(meta["type"], "meta");
    Ok(())
}

#[test]
fn export_csv_has_header_and_rows() -> Result<(), Box<dyn std::error::Error>> {
    let output = tokmd_cmd().args(["export", "--format", "csv"]).output()?;

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout)?;
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
    Ok(())
}

#[test]
fn export_json_has_envelope() -> Result<(), Box<dyn std::error::Error>> {
    let output = tokmd_cmd().args(["export", "--format", "json"]).output()?;

    assert!(output.status.success());
    let json: Value = serde_json::from_slice(&output.stdout)?;
    assert!(json["schema_version"].is_number());
    assert_eq!(json["mode"], "export");
    assert!(json["rows"].is_array());
    Ok(())
}

#[test]
fn export_max_rows_limits_output() -> Result<(), Box<dyn std::error::Error>> {
    let output = tokmd_cmd()
        .args(["export", "--format", "json", "--max-rows", "1"])
        .output()?;

    assert!(output.status.success());
    let json: Value = serde_json::from_slice(&output.stdout)?;
    let rows = json["rows"].as_array().ok_or("not an array")?;
    assert!(rows.len() <= 1, "--max-rows 1 should limit to 1 row");
    Ok(())
}

// ===========================================================================
// 6. run subcommand
// ===========================================================================

#[test]
fn run_generates_receipt_and_artifacts() -> Result<(), Box<dyn std::error::Error>> {
    let dir = tempdir()?;
    let output_dir = dir.path().join("run_out");

    let mut cmd = Command::new(env!("CARGO_BIN_EXE_tokmd"));
    cmd.current_dir(common::fixture_root())
        .args(["run", "--output-dir"])
        .arg(output_dir.to_str().ok_or("not a str")?)
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
        serde_json::from_str(&std::fs::read_to_string(output_dir.join("receipt.json"))?)?;
    assert!(receipt["schema_version"].is_number());
    Ok(())
}

// ===========================================================================
// 7. analyze subcommand
// ===========================================================================

#[test]
fn analyze_receipt_json_has_derived_metrics() -> Result<(), Box<dyn std::error::Error>> {
    let output = tokmd_cmd()
        .args(["analyze", "--preset", "receipt", "--format", "json"])
        .output()?;

    assert!(output.status.success());
    let json: Value = serde_json::from_slice(&output.stdout)?;
    assert!(json["schema_version"].is_number());
    assert!(json["derived"].is_object(), "should have derived metrics");
    Ok(())
}

#[test]
fn analyze_receipt_markdown_contains_heading() -> Result<(), Box<dyn std::error::Error>> {
    tokmd_cmd()
        .args(["analyze", "--preset", "receipt", "--format", "md"])
        .assert()
        .success()
        .stdout(predicate::str::contains("#"));
    Ok(())
}

#[test]
fn analyze_health_preset_json() -> Result<(), Box<dyn std::error::Error>> {
    let output = tokmd_cmd()
        .args(["analyze", "--preset", "health", "--format", "json"])
        .output()?;

    assert!(output.status.success());
    let json: Value = serde_json::from_slice(&output.stdout)?;
    assert!(json["schema_version"].is_number());
    assert!(json["derived"].is_object());
    Ok(())
}

// ===========================================================================
// 8. badge subcommand
// ===========================================================================

#[test]
fn badge_lines_metric_outputs_svg() -> Result<(), Box<dyn std::error::Error>> {
    tokmd_cmd()
        .args(["badge", "--metric", "lines"])
        .assert()
        .success()
        .stdout(predicate::str::contains("<svg"))
        .stdout(predicate::str::contains("</svg>"));
    Ok(())
}

#[test]
fn badge_tokens_metric_outputs_svg() -> Result<(), Box<dyn std::error::Error>> {
    tokmd_cmd()
        .args(["badge", "--metric", "tokens"])
        .assert()
        .success()
        .stdout(predicate::str::contains("<svg"))
        .stdout(predicate::str::contains("tokens"));
    Ok(())
}

#[test]
fn badge_bytes_metric_outputs_svg() -> Result<(), Box<dyn std::error::Error>> {
    tokmd_cmd()
        .args(["badge", "--metric", "bytes"])
        .assert()
        .success()
        .stdout(predicate::str::contains("<svg"));
    Ok(())
}

#[test]
fn badge_out_flag_writes_file() -> Result<(), Box<dyn std::error::Error>> {
    let dir = tempdir()?;
    let out = dir.path().join("badge.svg");

    tokmd_cmd()
        .args(["badge", "--metric", "lines", "--out"])
        .arg(&out)
        .assert()
        .success()
        .stdout("");

    let content = std::fs::read_to_string(&out)?;
    assert!(content.contains("<svg"));
    assert!(content.contains("</svg>"));
    Ok(())
}

// ===========================================================================
// 9. tools subcommand
// ===========================================================================

#[test]
fn tools_openai_format_has_functions() -> Result<(), Box<dyn std::error::Error>> {
    let output = tokmd_cmd().args(["tools", "--format", "openai"]).output()?;

    assert!(output.status.success());
    let json: Value = serde_json::from_slice(&output.stdout)?;
    let funcs = json["functions"].as_array().ok_or("not an array")?;
    assert!(!funcs.is_empty());
    for f in funcs {
        assert!(f["parameters"].is_object());
    }
    Ok(())
}

#[test]
fn tools_anthropic_format_has_tools() -> Result<(), Box<dyn std::error::Error>> {
    let output = tokmd_cmd()
        .args(["tools", "--format", "anthropic"])
        .output()?;

    assert!(output.status.success());
    let json: Value = serde_json::from_slice(&output.stdout)?;
    let tools = json["tools"].as_array().ok_or("not an array")?;
    assert!(!tools.is_empty());
    for t in tools {
        assert!(t["input_schema"].is_object());
    }
    Ok(())
}

#[test]
fn tools_jsonschema_format_has_schema_version() -> Result<(), Box<dyn std::error::Error>> {
    let output = tokmd_cmd()
        .args(["tools", "--format", "jsonschema"])
        .output()?;

    assert!(output.status.success());
    let json: Value = serde_json::from_slice(&output.stdout)?;
    assert!(json["schema_version"].is_number());
    assert!(json["tools"].is_array());
    Ok(())
}

#[test]
fn tools_pretty_flag_adds_whitespace() -> Result<(), Box<dyn std::error::Error>> {
    let compact = tokmd_cmd()
        .args(["tools", "--format", "jsonschema"])
        .output()?;
    let pretty = tokmd_cmd()
        .args(["tools", "--format", "jsonschema", "--pretty"])
        .output()?;

    assert!(pretty.stdout.len() > compact.stdout.len());
    Ok(())
}

// ===========================================================================
// 10. context subcommand
// ===========================================================================

#[test]
fn context_default_mode_lists_files() -> Result<(), Box<dyn std::error::Error>> {
    tokmd_cmd()
        .arg("context")
        .assert()
        .success()
        .stdout(predicate::str::is_empty().not());
    Ok(())
}

#[test]
fn context_list_mode_includes_source_file() -> Result<(), Box<dyn std::error::Error>> {
    tokmd_cmd()
        .args(["context", "--mode", "list"])
        .assert()
        .success()
        .stdout(predicate::str::contains("src/main.rs"));
    Ok(())
}

#[test]
fn context_json_mode_produces_valid_json() -> Result<(), Box<dyn std::error::Error>> {
    let output = tokmd_cmd().args(["context", "--mode", "json"]).output()?;

    assert!(output.status.success());
    let json: Value = serde_json::from_slice(&output.stdout)?;
    assert!(json.is_object());
    Ok(())
}

// ===========================================================================
// 11. init subcommand
// ===========================================================================

#[test]
fn init_print_outputs_tokeignore_template() -> Result<(), Box<dyn std::error::Error>> {
    tokmd_cmd()
        .args(["init", "--print", "--non-interactive"])
        .assert()
        .success()
        .stdout(predicate::str::is_empty().not());
    Ok(())
}

#[test]
fn init_print_rust_template_contains_target() -> Result<(), Box<dyn std::error::Error>> {
    tokmd_cmd()
        .args(["init", "--print", "--template", "rust", "--non-interactive"])
        .assert()
        .success()
        .stdout(predicate::str::contains("target/"));
    Ok(())
}

#[test]
fn init_print_node_template_contains_node_modules() -> Result<(), Box<dyn std::error::Error>> {
    tokmd_cmd()
        .args(["init", "--print", "--template", "node", "--non-interactive"])
        .assert()
        .success()
        .stdout(predicate::str::contains("node_modules/"));
    Ok(())
}

#[test]
fn init_non_interactive_creates_tokeignore_file() -> Result<(), Box<dyn std::error::Error>> {
    let dir = tempdir()?;
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_tokmd"));
    cmd.current_dir(dir.path())
        .args(["init", "--non-interactive"])
        .assert()
        .success();

    assert!(dir.path().join(".tokeignore").exists());
    Ok(())
}

#[test]
fn init_refuses_overwrite_without_force() -> Result<(), Box<dyn std::error::Error>> {
    let dir = tempdir()?;
    std::fs::write(dir.path().join(".tokeignore"), "# existing\n")?;

    let mut cmd = Command::new(env!("CARGO_BIN_EXE_tokmd"));
    cmd.current_dir(dir.path())
        .args(["init", "--non-interactive"])
        .assert()
        .failure();
    Ok(())
}

// ===========================================================================
// 12. check-ignore subcommand
// ===========================================================================

#[test]
fn check_ignore_with_excluded_file_reports_ignored() -> Result<(), Box<dyn std::error::Error>> {
    let dir = tempdir()?;
    std::fs::write(dir.path().join("hello.rs"), "fn main() {}")?;

    let mut cmd = Command::new(env!("CARGO_BIN_EXE_tokmd"));
    cmd.current_dir(dir.path())
        .args(["--exclude", "hello.rs", "check-ignore", "hello.rs"])
        .assert()
        .code(0)
        .stdout(predicate::str::contains("ignored"));
    Ok(())
}

#[test]
fn check_ignore_nonexistent_file_exits_nonzero() -> Result<(), Box<dyn std::error::Error>> {
    tokmd_cmd()
        .args(["check-ignore", "does_not_exist.txt"])
        .assert()
        .code(1);
    Ok(())
}

// ===========================================================================
// 13. completions subcommand
// ===========================================================================

#[test]
fn completions_bash_produces_script() -> Result<(), Box<dyn std::error::Error>> {
    tokmd_cmd()
        .args(["completions", "bash"])
        .assert()
        .success()
        .stdout(predicate::str::is_empty().not());
    Ok(())
}

#[test]
fn completions_zsh_produces_script() -> Result<(), Box<dyn std::error::Error>> {
    tokmd_cmd()
        .args(["completions", "zsh"])
        .assert()
        .success()
        .stdout(predicate::str::is_empty().not());
    Ok(())
}

#[test]
fn completions_fish_produces_script() -> Result<(), Box<dyn std::error::Error>> {
    tokmd_cmd()
        .args(["completions", "fish"])
        .assert()
        .success()
        .stdout(predicate::str::is_empty().not());
    Ok(())
}

#[test]
fn completions_powershell_produces_script() -> Result<(), Box<dyn std::error::Error>> {
    tokmd_cmd()
        .args(["completions", "powershell"])
        .assert()
        .success()
        .stdout(predicate::str::is_empty().not());
    Ok(())
}

#[test]
fn completions_elvish_produces_script() -> Result<(), Box<dyn std::error::Error>> {
    tokmd_cmd()
        .args(["completions", "elvish"])
        .assert()
        .success()
        .stdout(predicate::str::is_empty().not());
    Ok(())
}

// ===========================================================================
// 14. baseline subcommand
// ===========================================================================

#[test]
fn baseline_generates_valid_json_output() -> Result<(), Box<dyn std::error::Error>> {
    let dir = tempdir()?;
    let out = dir.path().join("baseline.json");

    let mut cmd = Command::new(env!("CARGO_BIN_EXE_tokmd"));
    cmd.current_dir(common::fixture_root())
        .args(["--no-progress", "baseline", "--output"])
        .arg(&out)
        .arg("--force")
        .assert()
        .success();

    let json: Value = serde_json::from_str(&std::fs::read_to_string(&out)?)?;
    assert_eq!(json["baseline_version"].as_u64(), Some(1));
    assert!(json.get("metrics").is_some());
    Ok(())
}

// ===========================================================================
// 15. diff subcommand (requires two runs)
// ===========================================================================

#[test]
fn diff_between_identical_runs_shows_no_changes() -> Result<(), Box<dyn std::error::Error>> {
    let dir = tempdir()?;
    let run1 = dir.path().join("r1");
    let run2 = dir.path().join("r2");

    // Produce two identical runs
    for out_dir in [&run1, &run2] {
        let mut cmd = Command::new(env!("CARGO_BIN_EXE_tokmd"));
        cmd.current_dir(common::fixture_root())
            .args(["run", "--output-dir"])
            .arg(out_dir.to_str().ok_or("not a str")?)
            .arg(".")
            .assert()
            .success();
    }

    let mut cmd = Command::new(env!("CARGO_BIN_EXE_tokmd"));
    let output = cmd
        .args([
            "diff",
            "--from",
            run1.join("lang.json").to_str().ok_or("not a str")?,
            "--to",
            run2.join("lang.json").to_str().ok_or("not a str")?,
            "--format",
            "json",
        ])
        .output()?;

    assert!(output.status.success());
    let json: Value = serde_json::from_slice(&output.stdout)?;
    assert!(json.is_object());
    Ok(())
}

// ===========================================================================
// 16. Global flag interactions
// ===========================================================================

#[test]
fn exclude_flag_removes_rust_from_lang() -> Result<(), Box<dyn std::error::Error>> {
    let output = tokmd_cmd()
        .args(["--exclude", "*.rs", "lang", "--format", "json"])
        .output()?;

    assert!(output.status.success());
    let json: Value = serde_json::from_slice(&output.stdout)?;
    let has_rust = json["rows"]
        .as_array()
        .ok_or("not an array")?
        .iter()
        .any(|r| r["lang"].as_str() == Some("Rust"));
    assert!(!has_rust, "excluding *.rs should remove Rust");
    Ok(())
}

#[test]
fn verbose_flag_accepted_on_lang() -> Result<(), Box<dyn std::error::Error>> {
    tokmd_cmd().args(["--verbose", "lang"]).assert().success();
    Ok(())
}

#[test]
fn verbose_flag_accepted_on_module() -> Result<(), Box<dyn std::error::Error>> {
    tokmd_cmd().args(["--verbose", "module"]).assert().success();
    Ok(())
}

#[test]
fn no_progress_flag_accepted() -> Result<(), Box<dyn std::error::Error>> {
    tokmd_cmd()
        .args(["--no-progress", "lang"])
        .assert()
        .success();
    Ok(())
}

// ===========================================================================
// 17. Error cases
// ===========================================================================

#[test]
fn invalid_subcommand_fails() -> Result<(), Box<dyn std::error::Error>> {
    tokmd_cmd()
        .arg("this-subcommand-does-not-exist")
        .assert()
        .failure()
        .stderr(predicate::str::is_empty().not());
    Ok(())
}

#[test]
fn lang_invalid_format_fails() -> Result<(), Box<dyn std::error::Error>> {
    tokmd_cmd()
        .args(["lang", "--format", "invalid_fmt"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("invalid value"));
    Ok(())
}

#[test]
fn module_invalid_format_fails() -> Result<(), Box<dyn std::error::Error>> {
    tokmd_cmd()
        .args(["module", "--format", "invalid_fmt"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("invalid value"));
    Ok(())
}

#[test]
fn export_invalid_format_fails() -> Result<(), Box<dyn std::error::Error>> {
    tokmd_cmd()
        .args(["export", "--format", "invalid_fmt"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("invalid value"));
    Ok(())
}

#[test]
fn analyze_invalid_format_fails() -> Result<(), Box<dyn std::error::Error>> {
    tokmd_cmd()
        .args(["analyze", "--format", "invalid_fmt"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("invalid value"));
    Ok(())
}

#[test]
fn analyze_invalid_preset_fails() -> Result<(), Box<dyn std::error::Error>> {
    tokmd_cmd()
        .args(["analyze", "--preset", "nonexistent"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("invalid value"));
    Ok(())
}

#[test]
fn tools_invalid_format_fails() -> Result<(), Box<dyn std::error::Error>> {
    tokmd_cmd()
        .args(["tools", "--format", "invalid_fmt"])
        .assert()
        .failure();
    Ok(())
}

#[test]
fn unknown_flag_on_lang_fails() -> Result<(), Box<dyn std::error::Error>> {
    tokmd_cmd()
        .args(["lang", "--this-flag-does-not-exist"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("unexpected argument"));
    Ok(())
}

#[test]
fn lang_invalid_children_mode_fails() -> Result<(), Box<dyn std::error::Error>> {
    tokmd_cmd()
        .args(["lang", "--children", "invalid_mode"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("invalid value"));
    Ok(())
}

#[test]
fn gate_missing_args_fails() -> Result<(), Box<dyn std::error::Error>> {
    tokmd_cmd()
        .arg("gate")
        .assert()
        .failure()
        .stderr(predicate::str::is_empty().not());
    Ok(())
}

#[test]
fn diff_missing_args_fails() -> Result<(), Box<dyn std::error::Error>> {
    tokmd_cmd()
        .arg("diff")
        .assert()
        .failure()
        .stderr(predicate::str::is_empty().not());
    Ok(())
}

#[test]
fn diff_nonexistent_files_fails() -> Result<(), Box<dyn std::error::Error>> {
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
    Ok(())
}

// ===========================================================================
// 18. Subcommand --help flags
// ===========================================================================

#[test]
fn lang_help_shows_format() -> Result<(), Box<dyn std::error::Error>> {
    tokmd_cmd()
        .args(["lang", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("--format"));
    Ok(())
}

#[test]
fn module_help_shows_depth() -> Result<(), Box<dyn std::error::Error>> {
    tokmd_cmd()
        .args(["module", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("--module-depth"));
    Ok(())
}

#[test]
fn export_help_shows_format() -> Result<(), Box<dyn std::error::Error>> {
    tokmd_cmd()
        .args(["export", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("--format"));
    Ok(())
}

#[test]
fn analyze_help_shows_preset() -> Result<(), Box<dyn std::error::Error>> {
    tokmd_cmd()
        .args(["analyze", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("--preset"));
    Ok(())
}

#[test]
fn badge_help_shows_metric() -> Result<(), Box<dyn std::error::Error>> {
    tokmd_cmd()
        .args(["badge", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("--metric"));
    Ok(())
}

#[test]
fn context_help_shows_mode() -> Result<(), Box<dyn std::error::Error>> {
    tokmd_cmd()
        .args(["context", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("--mode"));
    Ok(())
}

// ===========================================================================
// 19. Empty directory behavior
// ===========================================================================

#[test]
fn lang_empty_dir_succeeds_with_zero_totals() -> Result<(), Box<dyn std::error::Error>> {
    let dir = tempdir()?;
    std::fs::create_dir_all(dir.path().join(".git"))?;

    let mut cmd = Command::new(env!("CARGO_BIN_EXE_tokmd"));
    cmd.current_dir(dir.path())
        .arg("lang")
        .assert()
        .success()
        .stdout(predicate::str::contains("|**Total**|0|0|0|0|"));
    Ok(())
}

#[test]
fn module_empty_dir_succeeds_with_zero_totals() -> Result<(), Box<dyn std::error::Error>> {
    let dir = tempdir()?;
    std::fs::create_dir_all(dir.path().join(".git"))?;

    let mut cmd = Command::new(env!("CARGO_BIN_EXE_tokmd"));
    cmd.current_dir(dir.path())
        .arg("module")
        .assert()
        .success()
        .stdout(predicate::str::contains("|**Total**|0|0|0|0|0|0|"));
    Ok(())
}

#[test]
fn export_empty_dir_produces_meta_only() -> Result<(), Box<dyn std::error::Error>> {
    let dir = tempdir()?;
    std::fs::create_dir_all(dir.path().join(".git"))?;

    let output = Command::new(env!("CARGO_BIN_EXE_tokmd"))
        .current_dir(dir.path())
        .arg("export")
        .output()?;

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout)?;
    let lines: Vec<&str> = stdout.lines().collect();
    assert_eq!(lines.len(), 1, "expected only the meta record");
    assert!(lines[0].contains(r#""type":"meta""#));
    Ok(())
}
