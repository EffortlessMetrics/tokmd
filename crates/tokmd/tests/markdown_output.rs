mod common;

use assert_cmd::Command;
use predicates::prelude::*;

fn tokmd_cmd() -> Command {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_tokmd"));
    cmd.current_dir(common::fixture_root());
    cmd
}

// ---------------------------------------------------------------------------
// tokmd lang (default Markdown output)
// ---------------------------------------------------------------------------

#[test]
fn lang_markdown_has_table_header() {
    tokmd_cmd()
        .arg("lang")
        .arg(".")
        .assert()
        .success()
        .stdout(predicate::str::contains("|Lang|"))
        .stdout(predicate::str::contains("|Code|"))
        .stdout(predicate::str::contains("|Tokens|"));
}

#[test]
fn lang_markdown_has_separator_line() {
    let output = tokmd_cmd()
        .arg("lang")
        .arg(".")
        .output()
        .expect("failed to run tokmd lang");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("invalid UTF-8");

    // Markdown table separators use |---|
    let has_separator = stdout
        .lines()
        .any(|line| line.contains("|---") || line.contains("|:--"));
    assert!(
        has_separator,
        "expected Markdown table separator line in output:\n{stdout}"
    );
}

#[test]
fn lang_markdown_has_data_rows() {
    // The fixture contains .rs files so Rust must appear as a data row
    tokmd_cmd()
        .arg("lang")
        .arg(".")
        .assert()
        .success()
        .stdout(predicate::str::contains("|Rust|"));
}

#[test]
fn lang_markdown_has_total_row() {
    tokmd_cmd()
        .arg("lang")
        .arg(".")
        .assert()
        .success()
        .stdout(predicate::str::contains("|**Total**|"));
}

// ---------------------------------------------------------------------------
// tokmd module (default Markdown output)
// ---------------------------------------------------------------------------

#[test]
fn module_markdown_has_table_header() {
    tokmd_cmd()
        .arg("module")
        .arg(".")
        .assert()
        .success()
        .stdout(predicate::str::contains("|Module|"))
        .stdout(predicate::str::contains("|Code|"));
}

#[test]
fn module_markdown_has_separator_line() {
    let output = tokmd_cmd()
        .arg("module")
        .arg(".")
        .output()
        .expect("failed to run tokmd module");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("invalid UTF-8");

    let has_separator = stdout
        .lines()
        .any(|line| line.contains("|---") || line.contains("|:--"));
    assert!(
        has_separator,
        "expected Markdown table separator line in module output:\n{stdout}"
    );
}

#[test]
fn module_markdown_has_root_row() {
    tokmd_cmd()
        .arg("module")
        .arg(".")
        .assert()
        .success()
        .stdout(predicate::str::contains("|(root)|"));
}

#[test]
fn module_markdown_has_src_row() {
    tokmd_cmd()
        .arg("module")
        .arg(".")
        .assert()
        .success()
        .stdout(predicate::str::contains("|src|"));
}

// ---------------------------------------------------------------------------
// tokmd analyze --preset receipt (default Markdown output)
// ---------------------------------------------------------------------------

#[test]
fn analyze_receipt_markdown_has_sections() {
    tokmd_cmd()
        .arg("analyze")
        .arg("--preset")
        .arg("receipt")
        .arg(".")
        .assert()
        .success()
        // Markdown sections start with ## or ###
        .stdout(predicate::str::contains("##"));
}

#[test]
fn analyze_receipt_markdown_has_derived_metrics() {
    let output = tokmd_cmd()
        .arg("analyze")
        .arg("--preset")
        .arg("receipt")
        .arg(".")
        .output()
        .expect("failed to run tokmd analyze");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("invalid UTF-8");

    // Receipt preset produces derived metrics section with density/distribution
    let has_table = stdout
        .lines()
        .any(|line| line.contains("|---") || line.contains("|:--"));
    assert!(
        has_table,
        "expected at least one Markdown table in analyze output:\n{stdout}"
    );
}

// ---------------------------------------------------------------------------
// tokmd badge --metric code (SVG output)
// ---------------------------------------------------------------------------

#[test]
fn badge_code_metric_produces_svg() {
    tokmd_cmd()
        .arg("badge")
        .arg("--metric")
        .arg("lines")
        .assert()
        .success()
        .stdout(predicate::str::contains("<svg"))
        .stdout(predicate::str::contains("</svg>"));
}

#[test]
fn badge_code_metric_svg_has_content() {
    tokmd_cmd()
        .arg("badge")
        .arg("--metric")
        .arg("lines")
        .assert()
        .success()
        .stdout(predicate::str::contains("lines"));
}
