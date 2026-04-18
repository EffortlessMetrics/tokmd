//! Extended end-to-end CLI tests covering every subcommand, format
//! variation, determinism, and error handling.

mod common;

use assert_cmd::Command;
use predicates::prelude::*;
use serde_json::Value;

fn tokmd_cmd() -> Command {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_tokmd"));
    cmd.current_dir(common::fixture_root());
    cmd
}

// ---------------------------------------------------------------------------
// 1. Default invocation (no subcommand → lang)
// ---------------------------------------------------------------------------

#[cfg(feature = "analysis")]
#[test]
fn default_invocation_runs_successfully_with_output() {
    tokmd_cmd()
        .assert()
        .success()
        .stdout(predicate::str::is_empty().not());
}

// ---------------------------------------------------------------------------
// 2. tokmd lang — markdown output
// ---------------------------------------------------------------------------

#[test]
fn lang_produces_markdown_output() {
    tokmd_cmd()
        .arg("lang")
        .assert()
        .success()
        .stdout(predicate::str::contains("Lang"))
        .stdout(predicate::str::contains("|"));
}

// ---------------------------------------------------------------------------
// 3. tokmd lang --format json — valid JSON
// ---------------------------------------------------------------------------

#[test]
fn lang_format_json_produces_valid_json() {
    let output = tokmd_cmd()
        .args(["lang", "--format", "json"])
        .output()
        .expect("failed to run");

    assert!(output.status.success());
    let json: Value = serde_json::from_slice(&output.stdout)
        .expect("lang --format json should produce valid JSON");
    assert!(json.is_object());
    assert!(json["rows"].is_array());
}

// ---------------------------------------------------------------------------
// 4. tokmd lang --format tsv — tab-separated
// ---------------------------------------------------------------------------

#[test]
fn lang_format_tsv_contains_tabs() {
    let output = tokmd_cmd()
        .args(["lang", "--format", "tsv"])
        .output()
        .expect("failed to run");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains('\t'), "TSV output must contain tabs");
    assert!(!stdout.is_empty());
}

// ---------------------------------------------------------------------------
// 5. tokmd module — module breakdown
// ---------------------------------------------------------------------------

#[test]
fn module_produces_output() {
    tokmd_cmd()
        .arg("module")
        .assert()
        .success()
        .stdout(predicate::str::is_empty().not())
        .stdout(predicate::str::contains("Module"));
}

// ---------------------------------------------------------------------------
// 6. tokmd module --format json — valid JSON
// ---------------------------------------------------------------------------

#[test]
fn module_format_json_produces_valid_json() {
    let output = tokmd_cmd()
        .args(["module", "--format", "json"])
        .output()
        .expect("failed to run");

    assert!(output.status.success());
    let json: Value = serde_json::from_slice(&output.stdout)
        .expect("module --format json should produce valid JSON");
    assert!(json["rows"].is_array());
}

// ---------------------------------------------------------------------------
// 7. tokmd export --format jsonl — JSONL output
// ---------------------------------------------------------------------------

#[test]
fn export_format_jsonl_produces_valid_jsonl() {
    let output = tokmd_cmd()
        .args(["export", "--format", "jsonl"])
        .output()
        .expect("failed to run");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    let lines: Vec<&str> = stdout.lines().filter(|l| !l.trim().is_empty()).collect();
    assert!(
        lines.len() >= 2,
        "JSONL should have meta + at least one data row"
    );
    for (i, line) in lines.iter().enumerate() {
        let _: Value = serde_json::from_str(line)
            .unwrap_or_else(|e| panic!("JSONL line {} is invalid JSON: {e}", i + 1));
    }
}

// ---------------------------------------------------------------------------
// 8. tokmd export --format csv — CSV output
// ---------------------------------------------------------------------------

#[test]
fn export_format_csv_has_header_and_rows() {
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
    assert!(
        lines[0].contains(','),
        "CSV header should contain commas: {}",
        lines[0]
    );
}

// ---------------------------------------------------------------------------
// 9. tokmd --help — shows help
// ---------------------------------------------------------------------------

#[test]
fn help_flag_shows_help_text() {
    tokmd_cmd()
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Usage"))
        .stdout(predicate::str::contains("tokmd"));
}

// ---------------------------------------------------------------------------
// 10. tokmd lang --help — shows lang help
// ---------------------------------------------------------------------------

#[test]
fn lang_help_shows_subcommand_help() {
    tokmd_cmd()
        .args(["lang", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("--format"));
}

// ---------------------------------------------------------------------------
// 11–14. Shell completions
// ---------------------------------------------------------------------------

#[test]
fn completions_bash_generates_output() {
    tokmd_cmd()
        .args(["completions", "bash"])
        .assert()
        .success()
        .stdout(predicate::str::is_empty().not());
}

#[test]
fn completions_zsh_generates_output() {
    tokmd_cmd()
        .args(["completions", "zsh"])
        .assert()
        .success()
        .stdout(predicate::str::is_empty().not());
}

#[test]
fn completions_fish_generates_output() {
    tokmd_cmd()
        .args(["completions", "fish"])
        .assert()
        .success()
        .stdout(predicate::str::is_empty().not());
}

#[test]
fn completions_powershell_generates_output() {
    tokmd_cmd()
        .args(["completions", "powershell"])
        .assert()
        .success()
        .stdout(predicate::str::is_empty().not());
}

// ---------------------------------------------------------------------------
// 15. tokmd badge — SVG output
// ---------------------------------------------------------------------------

#[cfg(feature = "analysis")]
#[test]
fn badge_outputs_svg() {
    tokmd_cmd()
        .args(["badge", "--metric", "lines"])
        .assert()
        .success()
        .stdout(predicate::str::contains("<svg"));
}

// ---------------------------------------------------------------------------
// 16. tokmd init — tokeignore output
// ---------------------------------------------------------------------------

#[test]
fn init_print_generates_tokeignore_template() {
    tokmd_cmd()
        .args(["init", "--print", "--non-interactive"])
        .assert()
        .success()
        .stdout(predicate::str::is_empty().not());
}

// ---------------------------------------------------------------------------
// 17. tokmd tools — tool schema output
// ---------------------------------------------------------------------------

#[test]
fn tools_default_produces_json_schema() {
    let output = tokmd_cmd().arg("tools").output().expect("failed to run");

    assert!(output.status.success());
    let json: Value = serde_json::from_slice(&output.stdout)
        .expect("tools default format should produce valid JSON");
    assert!(json.is_object());
}

// ---------------------------------------------------------------------------
// 18. Nonexistent subcommand — error exit code
// ---------------------------------------------------------------------------

#[test]
fn nonexistent_subcommand_fails() {
    tokmd_cmd()
        .arg("this-subcommand-does-not-exist")
        .assert()
        .failure();
}

// ---------------------------------------------------------------------------
// 19. tokmd lang <PATH> — explicit path as positional arg
// ---------------------------------------------------------------------------

#[test]
fn lang_with_explicit_path_works() {
    tokmd_cmd()
        .args(["lang", "."])
        .assert()
        .success()
        .stdout(predicate::str::is_empty().not());
}

// ---------------------------------------------------------------------------
// 20. tokmd lang --children collapse
// ---------------------------------------------------------------------------

#[test]
fn lang_children_collapse_works() {
    tokmd_cmd()
        .args(["lang", "--children", "collapse"])
        .assert()
        .success()
        .stdout(predicate::str::is_empty().not());
}

// ---------------------------------------------------------------------------
// 21. tokmd lang --children separate
// ---------------------------------------------------------------------------

#[test]
fn lang_children_separate_works() {
    tokmd_cmd()
        .args(["lang", "--children", "separate"])
        .assert()
        .success()
        .stdout(predicate::str::is_empty().not());
}

// ---------------------------------------------------------------------------
// 22. Determinism — same command twice gives identical output
// ---------------------------------------------------------------------------

#[test]
fn deterministic_lang_json_output() {
    let run = || {
        tokmd_cmd()
            .args(["lang", "--format", "json"])
            .output()
            .expect("failed to run")
    };

    let out1 = run();
    let out2 = run();

    assert!(out1.status.success());
    assert!(out2.status.success());

    // Compare parsed JSON (ignoring potential timestamp differences in envelope)
    let json1: Value = serde_json::from_slice(&out1.stdout).unwrap();
    let json2: Value = serde_json::from_slice(&out2.stdout).unwrap();
    assert_eq!(
        json1["rows"], json2["rows"],
        "rows must be identical across runs"
    );
    assert_eq!(
        json1["total"], json2["total"],
        "total must be identical across runs"
    );
}

// ---------------------------------------------------------------------------
// 23. JSON output has schema_version field
// ---------------------------------------------------------------------------

#[test]
fn lang_json_has_schema_version() {
    let output = tokmd_cmd()
        .args(["lang", "--format", "json"])
        .output()
        .expect("failed to run");

    assert!(output.status.success());
    let json: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert!(
        json["schema_version"].is_number(),
        "JSON output must include schema_version"
    );
}

// ---------------------------------------------------------------------------
// 24. tokmd run — run receipt output
// ---------------------------------------------------------------------------

#[cfg(feature = "analysis")]
#[test]
fn run_produces_artifacts() {
    let tmp = tempfile::tempdir().expect("create temp dir");
    let output_dir = tmp.path().join("run-output");

    let mut cmd = tokmd_cmd();
    cmd.args(["run", "--output-dir"])
        .arg(output_dir.as_os_str());

    cmd.assert().success();

    // The run command writes artifacts to the output directory.
    assert!(
        output_dir.exists(),
        "run should create the output directory"
    );
}

// ---------------------------------------------------------------------------
// Extra: module --format tsv
// ---------------------------------------------------------------------------

#[test]
fn module_format_tsv_contains_tabs() {
    let output = tokmd_cmd()
        .args(["module", "--format", "tsv"])
        .output()
        .expect("failed to run");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains('\t'), "TSV output must contain tabs");
}

// ---------------------------------------------------------------------------
// Extra: export --format json
// ---------------------------------------------------------------------------

#[test]
fn export_format_json_produces_valid_json() {
    let output = tokmd_cmd()
        .args(["export", "--format", "json"])
        .output()
        .expect("failed to run");

    assert!(output.status.success());
    let json: Value = serde_json::from_slice(&output.stdout)
        .expect("export --format json should produce valid JSON");
    assert!(json.is_object());
}

// ---------------------------------------------------------------------------
// Extra: --version flag
// ---------------------------------------------------------------------------

#[test]
fn version_flag_shows_semver() {
    tokmd_cmd()
        .arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::is_match(r"\d+\.\d+\.\d+").unwrap());
}

// ---------------------------------------------------------------------------
// Extra: module --format json has schema_version
// ---------------------------------------------------------------------------

#[test]
fn module_json_has_schema_version() {
    let output = tokmd_cmd()
        .args(["module", "--format", "json"])
        .output()
        .expect("failed to run");

    assert!(output.status.success());
    let json: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert!(
        json["schema_version"].is_number(),
        "module JSON must include schema_version"
    );
}

// ---------------------------------------------------------------------------
// Extra: analyze --preset receipt --format json
// ---------------------------------------------------------------------------

#[cfg(feature = "analysis")]
#[test]
fn analyze_receipt_json_has_derived() {
    let output = tokmd_cmd()
        .args(["analyze", "--preset", "receipt", "--format", "json"])
        .output()
        .expect("failed to run");

    assert!(output.status.success());
    let json: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert!(
        json["schema_version"].is_number(),
        "analyze JSON must include schema_version"
    );
    assert!(json["derived"].is_object(), "should have derived metrics");
}

// ---------------------------------------------------------------------------
// Extra: tools --format openai
// ---------------------------------------------------------------------------

#[test]
fn tools_format_openai_produces_valid_json() {
    let output = tokmd_cmd()
        .args(["tools", "--format", "openai"])
        .output()
        .expect("failed to run");

    assert!(output.status.success());
    let json: Value = serde_json::from_slice(&output.stdout)
        .expect("tools --format openai should produce valid JSON");
    assert!(json.get("functions").is_some());
}

// ---------------------------------------------------------------------------
// Extra: tools --format anthropic
// ---------------------------------------------------------------------------

#[test]
fn tools_format_anthropic_produces_valid_json() {
    let output = tokmd_cmd()
        .args(["tools", "--format", "anthropic"])
        .output()
        .expect("failed to run");

    assert!(output.status.success());
    let json: Value = serde_json::from_slice(&output.stdout)
        .expect("tools --format anthropic should produce valid JSON");
    assert!(json.get("tools").is_some());
}

// ---------------------------------------------------------------------------
// Extra: deterministic module json output
// ---------------------------------------------------------------------------

#[test]
fn deterministic_module_json_output() {
    let run = || {
        tokmd_cmd()
            .args(["module", "--format", "json"])
            .output()
            .expect("failed to run")
    };

    let out1 = run();
    let out2 = run();

    assert!(out1.status.success());
    assert!(out2.status.success());

    let json1: Value = serde_json::from_slice(&out1.stdout).unwrap();
    let json2: Value = serde_json::from_slice(&out2.stdout).unwrap();
    assert_eq!(
        json1["rows"], json2["rows"],
        "module rows must be deterministic"
    );
    assert_eq!(
        json1["total"], json2["total"],
        "module total must be deterministic"
    );
}

// ---------------------------------------------------------------------------
// Extra: lang --top limits rows
// ---------------------------------------------------------------------------

#[test]
fn lang_top_limits_output_rows() {
    let output = tokmd_cmd()
        .args(["lang", "--format", "json", "--top", "1"])
        .output()
        .expect("failed to run");

    assert!(output.status.success());
    let json: Value = serde_json::from_slice(&output.stdout).unwrap();
    let rows = json["rows"].as_array().expect("rows should be array");
    // With --top 1, we get at most 1 real row + possibly an "Other" row
    assert!(
        rows.len() <= 2,
        "--top 1 should limit rows, got {}",
        rows.len()
    );
}

// ---------------------------------------------------------------------------
// Extra: export --format csv has commas (no tabs)
// ---------------------------------------------------------------------------

#[test]
fn export_csv_uses_commas_not_tabs() {
    let output = tokmd_cmd()
        .args(["export", "--format", "csv"])
        .output()
        .expect("failed to run");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    let header = stdout.lines().next().expect("should have header line");
    assert!(header.contains(','), "CSV header should contain commas");
}

// ---------------------------------------------------------------------------
// Extra: lang --files flag
// ---------------------------------------------------------------------------

#[test]
fn lang_files_flag_accepted() {
    tokmd_cmd()
        .args(["lang", "--files"])
        .assert()
        .success()
        .stdout(predicate::str::is_empty().not());
}

// ---------------------------------------------------------------------------
// Extra: completions elvish
// ---------------------------------------------------------------------------

#[test]
fn completions_elvish_generates_output() {
    tokmd_cmd()
        .args(["completions", "elvish"])
        .assert()
        .success()
        .stdout(predicate::str::is_empty().not());
}
