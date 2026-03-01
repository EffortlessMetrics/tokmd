//! E2E integration tests for `tokmd run` CLI command.
//!
//! These tests exercise the run subcommand end-to-end, verifying artifact
//! creation, JSON schema compliance, output content, and various flag
//! combinations.

mod common;

use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::tempdir;

fn tokmd_cmd() -> Command {
    Command::new(env!("CARGO_BIN_EXE_tokmd"))
}

/// Helper: run `tokmd run` on `fixture_root()` writing to `out_dir`.
fn run_on_fixtures(out_dir: &std::path::Path) {
    tokmd_cmd()
        .current_dir(common::fixture_root())
        .arg("run")
        .arg("--output-dir")
        .arg(out_dir)
        .arg(".")
        .assert()
        .success();
}

// ---------------------------------------------------------------------------
// 1. Basic: `tokmd run` in current directory creates all artifacts
// ---------------------------------------------------------------------------

#[test]
fn run_creates_all_four_artifacts() {
    let dir = tempdir().unwrap();
    let out_dir = dir.path().join("run_basic");

    run_on_fixtures(&out_dir);

    assert!(
        out_dir.join("receipt.json").exists(),
        "receipt.json missing"
    );
    assert!(out_dir.join("lang.json").exists(), "lang.json missing");
    assert!(out_dir.join("module.json").exists(), "module.json missing");
    assert!(
        out_dir.join("export.jsonl").exists(),
        "export.jsonl missing"
    );
}

#[test]
fn run_prints_output_directory_path() {
    let dir = tempdir().unwrap();
    let out_dir = dir.path().join("run_stdout");

    tokmd_cmd()
        .current_dir(common::fixture_root())
        .arg("run")
        .arg("--output-dir")
        .arg(&out_dir)
        .arg(".")
        .assert()
        .success()
        .stdout(predicate::str::contains("Writing run artifacts to:"));
}

// ---------------------------------------------------------------------------
// 2. JSON output: receipt.json is valid JSON
// ---------------------------------------------------------------------------

#[test]
fn run_receipt_is_valid_json() {
    let dir = tempdir().unwrap();
    let out_dir = dir.path().join("run_json");

    run_on_fixtures(&out_dir);

    let content = fs::read_to_string(out_dir.join("receipt.json")).unwrap();
    let _: serde_json::Value =
        serde_json::from_str(&content).expect("receipt.json should be valid JSON");
}

#[test]
fn run_lang_json_is_valid_json() {
    let dir = tempdir().unwrap();
    let out_dir = dir.path().join("run_lang_json");

    run_on_fixtures(&out_dir);

    let content = fs::read_to_string(out_dir.join("lang.json")).unwrap();
    let _: serde_json::Value =
        serde_json::from_str(&content).expect("lang.json should be valid JSON");
}

#[test]
fn run_module_json_is_valid_json() {
    let dir = tempdir().unwrap();
    let out_dir = dir.path().join("run_module_json");

    run_on_fixtures(&out_dir);

    let content = fs::read_to_string(out_dir.join("module.json")).unwrap();
    let _: serde_json::Value =
        serde_json::from_str(&content).expect("module.json should be valid JSON");
}

// ---------------------------------------------------------------------------
// 3. Empty directory produces artifacts with zero counts
// ---------------------------------------------------------------------------

#[test]
fn run_empty_directory_produces_artifacts() {
    let dir = tempdir().unwrap();
    let scan_dir = dir.path().join("empty_src");
    let out_dir = dir.path().join("run_empty");
    fs::create_dir_all(&scan_dir).unwrap();
    // Create a .git marker so ignore crate works correctly
    fs::create_dir_all(scan_dir.join(".git")).unwrap();

    tokmd_cmd()
        .current_dir(&scan_dir)
        .arg("run")
        .arg("--output-dir")
        .arg(&out_dir)
        .arg(".")
        .assert()
        .success();

    assert!(out_dir.join("receipt.json").exists());
    assert!(out_dir.join("lang.json").exists());
    assert!(out_dir.join("module.json").exists());
    assert!(out_dir.join("export.jsonl").exists());

    // lang.json should have an empty or minimal languages array
    let lang: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(out_dir.join("lang.json")).unwrap()).unwrap();
    let rows = lang["rows"].as_array().expect("lang.rows should be array");
    assert!(rows.is_empty(), "empty dir should have no language rows");
}

// ---------------------------------------------------------------------------
// 4. Directory with sample files produces expected language output
// ---------------------------------------------------------------------------

#[test]
fn run_sample_files_contain_expected_language() {
    let dir = tempdir().unwrap();
    let scan_dir = dir.path().join("sample_src");
    let out_dir = dir.path().join("run_sample");
    fs::create_dir_all(&scan_dir).unwrap();
    fs::create_dir_all(scan_dir.join(".git")).unwrap();

    // Write a small Rust file
    fs::write(
        scan_dir.join("main.rs"),
        "fn main() {\n    println!(\"hello\");\n}\n",
    )
    .unwrap();

    // Write a small Python file
    fs::write(
        scan_dir.join("script.py"),
        "def hello():\n    print(\"hello\")\n",
    )
    .unwrap();

    tokmd_cmd()
        .current_dir(&scan_dir)
        .arg("run")
        .arg("--output-dir")
        .arg(&out_dir)
        .arg(".")
        .assert()
        .success();

    let lang: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(out_dir.join("lang.json")).unwrap()).unwrap();

    let rows = lang["rows"].as_array().expect("rows should be array");
    assert!(rows.len() >= 2, "should have at least Rust and Python rows");

    let languages: Vec<&str> = rows.iter().filter_map(|r| r["lang"].as_str()).collect();
    assert!(languages.contains(&"Rust"), "should contain Rust");
    assert!(languages.contains(&"Python"), "should contain Python");
}

// ---------------------------------------------------------------------------
// 5. Schema field verification on receipt.json
// ---------------------------------------------------------------------------

#[test]
fn run_receipt_has_expected_schema_fields() {
    let dir = tempdir().unwrap();
    let out_dir = dir.path().join("run_schema");

    run_on_fixtures(&out_dir);

    let content = fs::read_to_string(out_dir.join("receipt.json")).unwrap();
    let receipt: serde_json::Value = serde_json::from_str(&content).unwrap();

    assert!(
        receipt["schema_version"].is_number(),
        "receipt should have schema_version"
    );
    assert_eq!(
        receipt["schema_version"].as_u64(),
        Some(2),
        "schema_version should be 2"
    );
    assert!(
        receipt["generated_at_ms"].is_number(),
        "receipt should have generated_at_ms"
    );
    assert_eq!(
        receipt["lang_file"].as_str(),
        Some("lang.json"),
        "lang_file should be 'lang.json'"
    );
    assert_eq!(
        receipt["module_file"].as_str(),
        Some("module.json"),
        "module_file should be 'module.json'"
    );
    assert_eq!(
        receipt["export_file"].as_str(),
        Some("export.jsonl"),
        "export_file should be 'export.jsonl'"
    );
}

#[test]
fn run_lang_json_has_schema_version_and_scan() {
    let dir = tempdir().unwrap();
    let out_dir = dir.path().join("run_lang_schema");

    run_on_fixtures(&out_dir);

    let lang: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(out_dir.join("lang.json")).unwrap()).unwrap();

    assert!(
        lang["schema_version"].is_number(),
        "lang.json should have schema_version"
    );
    assert!(lang["scan"].is_object(), "lang.json should have scan block");
    assert!(
        lang["scan"]["paths"].is_array(),
        "lang.json scan should have paths"
    );
    assert!(lang["rows"].is_array(), "lang.json should have rows array");
    assert!(
        lang["total"].is_object(),
        "lang.json should have total object"
    );
}

// ---------------------------------------------------------------------------
// 6. Scan options: --name flag produces named output directory
// ---------------------------------------------------------------------------

#[test]
fn run_with_name_flag_creates_named_dir() {
    let dir = tempdir().unwrap();
    let work_dir = dir.path().join("workdir");
    fs::create_dir_all(&work_dir).unwrap();
    fs::create_dir_all(work_dir.join(".git")).unwrap();
    fs::create_dir_all(work_dir.join("src")).unwrap();
    fs::write(work_dir.join("src/lib.rs"), "fn hello() {}\n").unwrap();

    tokmd_cmd()
        .current_dir(&work_dir)
        .arg("run")
        .arg("--name")
        .arg("my-run")
        .arg(".")
        .assert()
        .success();

    let expected = work_dir.join(".runs/tokmd/my-run");
    assert!(
        expected.exists(),
        ".runs/tokmd/my-run should be created at {:?}",
        expected
    );
    assert!(expected.join("receipt.json").exists());
    assert!(expected.join("lang.json").exists());
}

// ---------------------------------------------------------------------------
// 7. Verify output contains language and module data
// ---------------------------------------------------------------------------

#[test]
fn run_module_json_contains_module_rows() {
    let dir = tempdir().unwrap();
    let out_dir = dir.path().join("run_module_rows");

    run_on_fixtures(&out_dir);

    let module: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(out_dir.join("module.json")).unwrap()).unwrap();

    assert!(
        module["rows"].is_array(),
        "module.json should have rows array"
    );
    assert!(
        module["scan"].is_object(),
        "module.json should have scan block"
    );
    assert!(
        module["schema_version"].is_number(),
        "module.json should have schema_version"
    );
}

#[test]
fn run_export_jsonl_has_valid_lines() {
    let dir = tempdir().unwrap();
    let out_dir = dir.path().join("run_export_jsonl");

    run_on_fixtures(&out_dir);

    let content = fs::read_to_string(out_dir.join("export.jsonl")).unwrap();
    let mut line_count = 0;
    for (i, line) in content.lines().enumerate() {
        if line.trim().is_empty() {
            continue;
        }
        let _: serde_json::Value = serde_json::from_str(line)
            .unwrap_or_else(|e| panic!("invalid JSON on line {}: {}\n  {}", i + 1, e, line));
        line_count += 1;
    }
    assert!(line_count > 0, "export.jsonl should have at least one line");
}

#[test]
fn run_export_jsonl_first_line_is_meta() {
    let dir = tempdir().unwrap();
    let out_dir = dir.path().join("run_export_meta");

    run_on_fixtures(&out_dir);

    let content = fs::read_to_string(out_dir.join("export.jsonl")).unwrap();
    let first_line = content
        .lines()
        .next()
        .expect("export.jsonl should have lines");
    let meta: serde_json::Value = serde_json::from_str(first_line).unwrap();

    assert_eq!(
        meta["type"].as_str(),
        Some("meta"),
        "first line should be type 'meta'"
    );
    assert!(
        meta["schema_version"].is_number(),
        "meta line should have schema_version"
    );
}

// ---------------------------------------------------------------------------
// 8. Redact flag
// ---------------------------------------------------------------------------

#[test]
fn run_redact_paths_hides_raw_paths_in_export() {
    let dir = tempdir().unwrap();
    let scan_dir = dir.path().join("secret_repo");
    let out_dir = dir.path().join("run_redacted");
    fs::create_dir_all(&scan_dir).unwrap();
    fs::create_dir_all(scan_dir.join(".git")).unwrap();
    fs::write(scan_dir.join("confidential.rs"), "fn secret() {}\n").unwrap();

    tokmd_cmd()
        .current_dir(&scan_dir)
        .arg("run")
        .arg("--output-dir")
        .arg(&out_dir)
        .arg("--redact")
        .arg("paths")
        .arg(".")
        .assert()
        .success();

    let export = fs::read_to_string(out_dir.join("export.jsonl")).unwrap();
    // Meta line should indicate redaction
    assert!(
        export.contains(r#""redact":"paths""#),
        "export.jsonl should indicate redact mode"
    );

    // Row lines should not contain the original filename
    for line in export.lines().skip(1) {
        if line.contains(r#""type":"row""#) {
            assert!(
                !line.contains("confidential"),
                "redacted export should not contain original filenames"
            );
        }
    }
}

// ---------------------------------------------------------------------------
// 9. Analysis preset: --analysis receipt
// ---------------------------------------------------------------------------

#[test]
fn run_with_analysis_preset_creates_analysis_artifacts() {
    let dir = tempdir().unwrap();
    let out_dir = dir.path().join("run_analysis");

    tokmd_cmd()
        .current_dir(common::fixture_root())
        .arg("run")
        .arg("--output-dir")
        .arg(&out_dir)
        .arg("--analysis")
        .arg("receipt")
        .arg(".")
        .assert()
        .success();

    // Core artifacts
    assert!(out_dir.join("receipt.json").exists());
    assert!(out_dir.join("lang.json").exists());

    // Analysis artifacts produced by the preset
    assert!(
        out_dir.join("analysis.json").exists(),
        "analysis.json should be created with --analysis"
    );
    assert!(
        out_dir.join("analysis.md").exists(),
        "analysis.md should be created with --analysis"
    );
}

// ---------------------------------------------------------------------------
// 10. Error handling: nonexistent path
// ---------------------------------------------------------------------------

#[test]
fn run_nonexistent_path_fails() {
    let dir = tempdir().unwrap();
    let out_dir = dir.path().join("run_noexist");

    tokmd_cmd()
        .arg("run")
        .arg("--output-dir")
        .arg(&out_dir)
        .arg("/this/path/definitely/does/not/exist")
        .assert()
        .failure();
}

// ---------------------------------------------------------------------------
// 11. Determinism: two runs on same data produce identical receipt content
// ---------------------------------------------------------------------------

#[test]
fn run_two_runs_produce_same_lang_data() {
    let dir = tempdir().unwrap();
    let out1 = dir.path().join("run_det1");
    let out2 = dir.path().join("run_det2");

    run_on_fixtures(&out1);
    run_on_fixtures(&out2);

    let lang1: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(out1.join("lang.json")).unwrap()).unwrap();
    let lang2: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(out2.join("lang.json")).unwrap()).unwrap();

    // Rows and totals should be identical (timestamps may differ)
    assert_eq!(lang1["rows"], lang2["rows"], "lang rows should match");
    assert_eq!(lang1["totals"], lang2["totals"], "lang totals should match");
}
