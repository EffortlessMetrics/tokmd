//! End-to-end CLI integration tests exercising core commands and flag
//! combinations.  Each test invokes the real `tokmd` binary against a
//! hermetic fixture directory.

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
// badge
// ---------------------------------------------------------------------------

#[cfg(feature = "analysis")]
#[test]
fn given_repo_when_badge_then_outputs_svg() {
    tokmd_cmd()
        .args(["badge", "--metric", "lines"])
        .assert()
        .success()
        .stdout(predicate::str::contains("<svg"));
}

#[cfg(feature = "analysis")]
#[test]
fn given_repo_when_badge_tokens_metric_then_svg_contains_tokens() {
    tokmd_cmd()
        .args(["badge", "--metric", "tokens"])
        .assert()
        .success()
        .stdout(predicate::str::contains("<svg"))
        .stdout(predicate::str::contains("tokens"));
}

#[cfg(feature = "analysis")]
#[test]
fn given_repo_when_badge_bytes_metric_then_svg_contains_bytes() {
    tokmd_cmd()
        .args(["badge", "--metric", "bytes"])
        .assert()
        .success()
        .stdout(predicate::str::contains("<svg"))
        .stdout(predicate::str::contains("bytes"));
}

// ---------------------------------------------------------------------------
// completions
// ---------------------------------------------------------------------------

#[test]
fn given_bash_when_completions_then_outputs_script() {
    tokmd_cmd()
        .args(["completions", "bash"])
        .assert()
        .success()
        .stdout(predicate::str::is_empty().not());
}

#[test]
fn given_powershell_when_completions_then_outputs_script() {
    tokmd_cmd()
        .args(["completions", "powershell"])
        .assert()
        .success()
        .stdout(predicate::str::is_empty().not());
}

// ---------------------------------------------------------------------------
// lang --format json
// ---------------------------------------------------------------------------

#[test]
fn given_repo_when_lang_json_then_valid_json_with_schema() {
    let output = tokmd_cmd()
        .args(["lang", "--format", "json"])
        .output()
        .expect("failed to run");

    assert!(output.status.success());
    let json: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert!(json["schema_version"].is_number());
    assert_eq!(json["mode"], "lang");
}

#[test]
fn given_repo_when_lang_json_then_rows_have_code_field() {
    let output = tokmd_cmd()
        .args(["lang", "--format", "json"])
        .output()
        .expect("failed to run");

    assert!(output.status.success());
    let json: Value = serde_json::from_slice(&output.stdout).unwrap();
    let rows = json["rows"].as_array().expect("rows should be an array");
    assert!(!rows.is_empty(), "should detect at least one language");
    for row in rows {
        assert!(row["code"].is_number(), "each row should have a code count");
    }
}

#[test]
fn given_repo_when_lang_tsv_then_tab_separated() {
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
}

// ---------------------------------------------------------------------------
// module --format json
// ---------------------------------------------------------------------------

#[test]
fn given_repo_when_module_json_then_has_rows() {
    let output = tokmd_cmd()
        .args(["module", "--format", "json"])
        .output()
        .expect("failed to run");

    assert!(output.status.success());
    let json: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert!(json["rows"].is_array());
    let rows = json["rows"].as_array().unwrap();
    assert!(!rows.is_empty(), "should have at least one module row");
}

#[test]
fn given_repo_when_module_json_then_total_present() {
    let output = tokmd_cmd()
        .args(["module", "--format", "json"])
        .output()
        .expect("failed to run");

    assert!(output.status.success());
    let json: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert!(json["total"].is_object(), "total should be present");
    assert!(
        json["total"]["code"].is_number(),
        "total should have code count"
    );
}

// ---------------------------------------------------------------------------
// export --format jsonl
// ---------------------------------------------------------------------------

#[test]
fn given_repo_when_export_jsonl_then_each_line_valid() {
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
fn given_repo_when_export_csv_then_has_header_and_rows() {
    let output = tokmd_cmd()
        .args(["export", "--format", "csv"])
        .output()
        .expect("failed to run");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    let lines: Vec<&str> = stdout.lines().collect();
    assert!(lines.len() >= 2, "should have header + at least one row");
    let header = lines[0];
    assert!(
        header.contains("path") || header.contains("language"),
        "CSV header should contain column names"
    );
}

// ---------------------------------------------------------------------------
// init
// ---------------------------------------------------------------------------

#[test]
fn given_when_init_print_then_generates_tokeignore() {
    tokmd_cmd()
        .args(["init", "--print", "--non-interactive"])
        .assert()
        .success()
        .stdout(predicate::str::is_empty().not());
}

// ---------------------------------------------------------------------------
// tools
// ---------------------------------------------------------------------------

#[test]
fn given_openai_when_tools_then_valid_json() {
    let output = tokmd_cmd()
        .args(["tools", "--format", "openai"])
        .output()
        .expect("failed to run");

    assert!(output.status.success());
    let json: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert!(
        json.get("functions").is_some(),
        "OpenAI format should have 'functions' key"
    );
}

#[test]
fn given_anthropic_when_tools_then_valid_json() {
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

// ---------------------------------------------------------------------------
// --version
// ---------------------------------------------------------------------------

#[test]
fn given_version_flag_then_shows_semver() {
    tokmd_cmd()
        .arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::is_match(r"\d+\.\d+\.\d+").unwrap());
}

// ---------------------------------------------------------------------------
// analyze
// ---------------------------------------------------------------------------

#[cfg(feature = "analysis")]
#[test]
fn given_repo_when_analyze_receipt_json_then_has_derived() {
    let output = tokmd_cmd()
        .args(["analyze", "--preset", "receipt", "--format", "json"])
        .output()
        .expect("failed to run");

    assert!(output.status.success());
    let json: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert!(json["schema_version"].is_number());
    assert!(json["derived"].is_object(), "should have derived metrics");
}

// ---------------------------------------------------------------------------
// lang markdown (default format)
// ---------------------------------------------------------------------------

#[test]
fn given_repo_when_lang_default_then_markdown_table() {
    tokmd_cmd()
        .arg("lang")
        .assert()
        .success()
        .stdout(predicate::str::contains("Lang"))
        .stdout(predicate::str::contains("Code"));
}

// ---------------------------------------------------------------------------
// module markdown (default format)
// ---------------------------------------------------------------------------

#[test]
fn given_repo_when_module_default_then_markdown_table() {
    tokmd_cmd()
        .arg("module")
        .assert()
        .success()
        .stdout(predicate::str::contains("Module"))
        .stdout(predicate::str::contains("Code"));
}
