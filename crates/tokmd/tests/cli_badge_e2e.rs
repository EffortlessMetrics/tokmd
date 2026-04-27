#![cfg(feature = "analysis")]

//! End-to-end tests for `tokmd badge` — metric variants, SVG structure,
//! and file-output behaviour.

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
// Metric variants
// ---------------------------------------------------------------------------

#[test]
fn badge_doc_metric_produces_svg_with_label() {
    tokmd_cmd()
        .args(["badge", "--metric", "doc"])
        .assert()
        .success()
        .stdout(predicate::str::contains("<svg"))
        .stdout(predicate::str::contains("</svg>"))
        .stdout(predicate::str::contains("doc"));
}

#[test]
fn badge_blank_metric_produces_svg_with_label() {
    tokmd_cmd()
        .args(["badge", "--metric", "blank"])
        .assert()
        .success()
        .stdout(predicate::str::contains("<svg"))
        .stdout(predicate::str::contains("</svg>"))
        .stdout(predicate::str::contains("blank"));
}

#[test]
fn badge_lines_svg_contains_xmlns_attribute() {
    tokmd_cmd()
        .args(["badge", "--metric", "lines"])
        .assert()
        .success()
        .stdout(predicate::str::contains("xmlns"));
}

#[test]
fn badge_tokens_svg_is_well_formed() {
    let output = tokmd_cmd()
        .args(["badge", "--metric", "tokens"])
        .output()
        .expect("failed to run");

    assert!(output.status.success());
    let svg = String::from_utf8(output.stdout).expect("valid UTF-8");
    assert!(svg.starts_with("<svg"), "SVG should start with <svg tag");
    assert!(svg.contains("</svg>"), "SVG should have closing tag");
    assert!(svg.contains("tokens"), "SVG should contain metric label");
}

// ---------------------------------------------------------------------------
// File output
// ---------------------------------------------------------------------------

#[test]
fn badge_out_flag_writes_valid_svg_file() -> Result<(), Box<dyn std::error::Error>> {
    let dir = tempdir()?;
    let out_file = dir.path().join("out.svg");

    tokmd_cmd()
        .args(["badge", "--metric", "doc", "--out"])
        .arg(&out_file)
        .assert()
        .success()
        .stdout("");

    let content = std::fs::read_to_string(&out_file)?;
    assert!(content.contains("<svg"), "file should contain SVG");
    assert!(content.contains("doc"), "file should contain metric label");
    Ok(())
}

#[test]
fn badge_out_to_nested_dir_creates_file() -> Result<(), Box<dyn std::error::Error>> {
    let dir = tempdir()?;
    let nested = dir.path().join("sub").join("dir");
    std::fs::create_dir_all(&nested)?;
    let out_file = nested.join("badge.svg");

    tokmd_cmd()
        .args(["badge", "--metric", "bytes", "--out"])
        .arg(&out_file)
        .assert()
        .success();

    assert!(out_file.exists(), "badge file should exist");
    let content = std::fs::read_to_string(&out_file)?;
    assert!(content.contains("<svg"));
    Ok(())
}

// ---------------------------------------------------------------------------
// Error cases
// ---------------------------------------------------------------------------

#[test]
fn badge_missing_metric_flag_fails() {
    tokmd_cmd()
        .arg("badge")
        .assert()
        .failure()
        .stderr(predicate::str::contains("--metric"));
}

#[test]
fn badge_invalid_metric_fails() {
    tokmd_cmd()
        .args(["badge", "--metric", "nonexistent"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("invalid value"));
}
