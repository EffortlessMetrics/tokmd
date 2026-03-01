//! E2E tests for the `tokmd badge` CLI command.
//!
//! Tests exercise badge generation with different metrics and verify
//! SVG output structure.

mod common;

use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::tempdir;

fn tokmd_cmd() -> Command {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_tokmd"));
    cmd.current_dir(common::fixture_root());
    cmd
}

// ---------------------------------------------------------------------------
// SVG output basics
// ---------------------------------------------------------------------------

#[test]
fn badge_lines_produces_valid_svg() {
    tokmd_cmd()
        .arg("badge")
        .arg("--metric")
        .arg("lines")
        .assert()
        .success()
        .stdout(predicate::str::contains("<svg"))
        .stdout(predicate::str::contains("</svg>"))
        .stdout(predicate::str::contains("xmlns"));
}

#[test]
fn badge_tokens_produces_svg() {
    tokmd_cmd()
        .arg("badge")
        .arg("--metric")
        .arg("tokens")
        .assert()
        .success()
        .stdout(predicate::str::contains("<svg"))
        .stdout(predicate::str::contains("tokens"));
}

#[test]
fn badge_bytes_produces_svg() {
    tokmd_cmd()
        .arg("badge")
        .arg("--metric")
        .arg("bytes")
        .assert()
        .success()
        .stdout(predicate::str::contains("<svg"))
        .stdout(predicate::str::contains("bytes"));
}

#[test]
fn badge_doc_metric_produces_svg() {
    tokmd_cmd()
        .arg("badge")
        .arg("--metric")
        .arg("doc")
        .assert()
        .success()
        .stdout(predicate::str::contains("<svg"))
        .stdout(predicate::str::contains("doc"));
}

#[test]
fn badge_blank_metric_produces_svg() {
    tokmd_cmd()
        .arg("badge")
        .arg("--metric")
        .arg("blank")
        .assert()
        .success()
        .stdout(predicate::str::contains("<svg"))
        .stdout(predicate::str::contains("blank"));
}

// ---------------------------------------------------------------------------
// --out file output
// ---------------------------------------------------------------------------

#[test]
fn badge_writes_svg_to_file() {
    let dir = tempdir().unwrap();
    let out_file = dir.path().join("badge.svg");

    tokmd_cmd()
        .arg("badge")
        .arg("--metric")
        .arg("lines")
        .arg("--out")
        .arg(&out_file)
        .assert()
        .success()
        .stdout("");

    let content = fs::read_to_string(&out_file).unwrap();
    assert!(
        content.contains("<svg"),
        "file should contain SVG opening tag"
    );
    assert!(
        content.contains("</svg>"),
        "file should contain SVG closing tag"
    );
    assert!(content.contains("xmlns"), "SVG should have xmlns attribute");
}

// ---------------------------------------------------------------------------
// SVG structure validation
// ---------------------------------------------------------------------------

#[test]
fn badge_svg_has_expected_structure() {
    let output = tokmd_cmd()
        .arg("badge")
        .arg("--metric")
        .arg("lines")
        .output()
        .expect("failed to execute tokmd badge");

    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).unwrap();

    // SVG must have proper XML structure
    assert!(stdout.contains("<svg"), "must contain svg open tag");
    assert!(stdout.contains("</svg>"), "must contain svg close tag");
    assert!(
        stdout.contains("xmlns=\"http://www.w3.org/2000/svg\"")
            || stdout.contains("xmlns='http://www.w3.org/2000/svg'"),
        "SVG must have proper xmlns"
    );

    // Badge should contain label and value text elements
    assert!(stdout.contains("<text"), "SVG should contain text elements");
    assert!(stdout.contains("<rect"), "SVG should contain rect elements");
}

#[test]
fn badge_svg_contains_metric_label() {
    let output = tokmd_cmd()
        .arg("badge")
        .arg("--metric")
        .arg("bytes")
        .output()
        .unwrap();

    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(
        stdout.contains("bytes"),
        "badge should contain the metric label 'bytes'"
    );
}

// ---------------------------------------------------------------------------
// explicit --path input
// ---------------------------------------------------------------------------

#[test]
fn badge_with_explicit_path() {
    tokmd_cmd()
        .arg("badge")
        .arg("--metric")
        .arg("lines")
        .arg(common::fixture_root())
        .assert()
        .success()
        .stdout(predicate::str::contains("<svg"));
}

// ---------------------------------------------------------------------------
// error cases
// ---------------------------------------------------------------------------

#[test]
fn badge_missing_metric_fails() {
    tokmd_cmd().arg("badge").assert().failure();
}
