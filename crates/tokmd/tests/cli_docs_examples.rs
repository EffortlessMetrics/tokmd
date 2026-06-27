#![cfg(feature = "analysis")]

//! Executable coverage for examples in docs/reference-cli.md.
//! Ensures that the documented examples actually run successfully.

use assert_cmd::Command;
use std::env;
use tempfile::tempdir;

fn tokmd_cmd() -> Command {
    Command::cargo_bin("tokmd").expect("binary exists")
}

// We need the fixture root, similar to other integration tests.
fn fixture_root() -> std::path::PathBuf {
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    std::path::Path::new(&manifest_dir)
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .to_path_buf()
}

#[test]
fn docs_context_example_1() {
    let dir = tempdir().unwrap();
    let output = dir.path().join("context.txt");

    // tokmd context --budget 128k --mode bundle --output context.txt
    tokmd_cmd()
        .current_dir(dir.path())
        .arg("context")
        .arg("--budget")
        .arg("128k")
        .arg("--mode")
        .arg("bundle")
        .arg("--output")
        .arg(&output)
        .assert()
        .success();

    assert!(output.exists());
}

#[test]
fn docs_context_example_2() {
    // tokmd context crates/tokmd xtask --strategy spread --budget 200k
    tokmd_cmd()
        .current_dir(fixture_root())
        .arg("context")
        .arg("crates/tokmd")
        .arg("xtask")
        .arg("--strategy")
        .arg("spread")
        .arg("--budget")
        .arg("200k")
        .assert()
        .success();
}

#[test]
fn docs_analyze_example_1() {
    let dir = tempdir().unwrap();

    // tokmd analyze --preset receipt --format md
    tokmd_cmd()
        .current_dir(dir.path())
        .arg("analyze")
        .arg("--preset")
        .arg("receipt")
        .arg("--format")
        .arg("md")
        .assert()
        .success();
}

#[test]
fn docs_analyze_example_2() {
    let dir = tempdir().unwrap();
    let out_dir = dir.path().join(".runs/analysis");

    // tokmd analyze . --preset risk --output-dir .runs/analysis
    tokmd_cmd()
        .current_dir(dir.path())
        .arg("analyze")
        .arg(".")
        .arg("--preset")
        .arg("risk")
        .arg("--output-dir")
        .arg(&out_dir)
        .assert()
        .success();

    assert!(out_dir.exists());
}

#[test]
fn docs_gate_example_2() {
    let dir = tempdir().unwrap();

    // Note: We skip Gate Example 1 (tokmd gate analysis.json --policy policy.toml)
    // because it requires specific fixture files to exist and be valid.

    // tokmd gate . --preset health --format json
    tokmd_cmd()
        .current_dir(dir.path())
        .arg("gate")
        .arg(".")
        .arg("--preset")
        .arg("health")
        .arg("--format")
        .arg("json")
        .assert()
        // Health preset might fail the gate (exit code 1), but the command itself should run
        // We just care that it executes without CLI parse errors (exit code 2).
        .code(predicates::prelude::predicate::in_hash(vec![0, 1]));
}
