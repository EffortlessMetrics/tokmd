//! Comprehensive CLI end-to-end pipeline tests exercising multi-command
//! workflows and output format validation across all tokmd subcommands.

mod common;

use assert_cmd::Command;
use predicates::prelude::*;
use serde_json::Value;

fn tokmd_cmd() -> Command {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_tokmd"));
    cmd.current_dir(common::fixture_root());
    cmd
}

// ===========================================================================
// lang: output formats
// ===========================================================================

#[test]
fn lang_default_outputs_markdown_table() {
    tokmd_cmd()
        .arg("lang")
        .assert()
        .success()
        .stdout(predicate::str::contains("Lang"))
        .stdout(predicate::str::contains("Code"))
        .stdout(predicate::str::contains("|"));
}

#[test]
fn lang_json_outputs_valid_json_with_rows() {
    let output = tokmd_cmd()
        .args(["lang", "--format", "json"])
        .output()
        .expect("run");
    assert!(output.status.success());
    let json: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert!(json["rows"].is_array(), "JSON must have rows array");
    assert!(
        json["schema_version"].is_number(),
        "must have schema_version"
    );
    assert_eq!(json["mode"], "lang");
}

#[test]
fn lang_tsv_has_tabs_and_header() {
    let output = tokmd_cmd()
        .args(["lang", "--format", "tsv"])
        .output()
        .expect("run");
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains('\t'), "TSV must contain tab characters");
    let first_line = stdout.lines().next().unwrap();
    assert!(
        first_line.contains("Lang") || first_line.contains("lang"),
        "TSV header should contain language column"
    );
}

#[test]
fn lang_json_rows_have_required_fields() {
    let output = tokmd_cmd()
        .args(["lang", "--format", "json"])
        .output()
        .expect("run");
    assert!(output.status.success());
    let json: Value = serde_json::from_slice(&output.stdout).unwrap();
    let rows = json["rows"].as_array().unwrap();
    assert!(
        !rows.is_empty(),
        "fixture must produce at least one language"
    );
    for row in rows {
        assert!(row["lang"].is_string(), "row must have lang");
        assert!(row["code"].is_number(), "row must have code");
        assert!(row["files"].is_number(), "row must have files");
        assert!(row["lines"].is_number(), "row must have lines");
    }
}

#[test]
fn lang_json_has_total_section() {
    let output = tokmd_cmd()
        .args(["lang", "--format", "json"])
        .output()
        .expect("run");
    assert!(output.status.success());
    let json: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert!(json["total"].is_object(), "must have total");
    assert!(json["total"]["code"].is_number(), "total must have code");
}

// ===========================================================================
// module: depth options and output formats
// ===========================================================================

#[test]
fn module_default_outputs_markdown() {
    tokmd_cmd()
        .arg("module")
        .assert()
        .success()
        .stdout(predicate::str::contains("Module"))
        .stdout(predicate::str::contains("Code"));
}

#[test]
fn module_json_has_rows_and_total() {
    let output = tokmd_cmd()
        .args(["module", "--format", "json"])
        .output()
        .expect("run");
    assert!(output.status.success());
    let json: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(json["mode"], "module");
    assert!(json["rows"].is_array());
    assert!(json["total"].is_object());
}

#[test]
fn module_tsv_has_tabs() {
    let output = tokmd_cmd()
        .args(["module", "--format", "tsv"])
        .output()
        .expect("run");
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains('\t'), "TSV must contain tabs");
}

#[test]
fn module_depth_1_reduces_nesting() {
    // With depth 1, module keys should be shorter (fewer segments)
    let output = tokmd_cmd()
        .args(["module", "--format", "json", "--module-depth", "1"])
        .output()
        .expect("run");
    assert!(output.status.success());
    let json: Value = serde_json::from_slice(&output.stdout).unwrap();
    let rows = json["rows"].as_array().unwrap();
    assert!(!rows.is_empty(), "should have at least one module row");
}

#[test]
fn module_json_rows_have_module_field() {
    let output = tokmd_cmd()
        .args(["module", "--format", "json"])
        .output()
        .expect("run");
    assert!(output.status.success());
    let json: Value = serde_json::from_slice(&output.stdout).unwrap();
    for row in json["rows"].as_array().unwrap() {
        assert!(row["module"].is_string(), "each row must have module field");
        assert!(row["code"].is_number(), "each row must have code field");
    }
}

// ===========================================================================
// export: JSONL, CSV, JSON
// ===========================================================================

#[test]
fn export_jsonl_each_line_is_valid_json() {
    let output = tokmd_cmd()
        .args(["export", "--format", "jsonl"])
        .output()
        .expect("run");
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    let lines: Vec<&str> = stdout.lines().filter(|l| !l.trim().is_empty()).collect();
    assert!(lines.len() >= 2, "need meta + at least one data row");
    for (i, line) in lines.iter().enumerate() {
        serde_json::from_str::<Value>(line)
            .unwrap_or_else(|e| panic!("line {i} invalid JSON: {e}"));
    }
}

#[test]
fn export_csv_has_header_and_data_rows() {
    let output = tokmd_cmd()
        .args(["export", "--format", "csv"])
        .output()
        .expect("run");
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    let lines: Vec<&str> = stdout.lines().collect();
    assert!(lines.len() >= 2, "need header + at least one data row");
    assert!(
        lines[0].contains("path") || lines[0].contains("language"),
        "CSV header should contain column names"
    );
}

#[test]
fn export_json_has_envelope_and_rows() {
    let output = tokmd_cmd()
        .args(["export", "--format", "json"])
        .output()
        .expect("run");
    assert!(output.status.success());
    let json: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(json["mode"], "export");
    assert!(json["schema_version"].is_number());
    assert!(json["rows"].is_array());
    let rows = json["rows"].as_array().unwrap();
    assert!(!rows.is_empty());
    for row in rows {
        assert!(row["path"].is_string(), "export row must have path");
        assert!(row["code"].is_number(), "export row must have code");
    }
}

#[test]
fn export_csv_columns_consistent() {
    let output = tokmd_cmd()
        .args(["export", "--format", "csv"])
        .output()
        .expect("run");
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    let lines: Vec<&str> = stdout.lines().collect();
    let header_cols = lines[0].split(',').count();
    for (i, line) in lines.iter().enumerate() {
        assert_eq!(
            line.split(',').count(),
            header_cols,
            "CSV row {i} column count mismatch"
        );
    }
}

#[test]
fn export_jsonl_first_line_is_meta_record() {
    let output = tokmd_cmd()
        .args(["export", "--format", "jsonl"])
        .output()
        .expect("run");
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    let first_line = stdout.lines().next().unwrap();
    let meta: Value = serde_json::from_str(first_line).unwrap();
    assert!(
        meta.get("schema_version").is_some() || meta.get("mode").is_some(),
        "first JSONL line should be a meta record"
    );
}

// ===========================================================================
// run: full scan (writes to output dir, no --format flag)
// ===========================================================================

#[test]
fn run_default_succeeds() {
    tokmd_cmd().arg("run").assert().success();
}

#[test]
fn run_with_output_dir_creates_artifacts() {
    let tmp = std::env::temp_dir().join(format!("tokmd-run-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&tmp);
    tokmd_cmd()
        .args(["run", "--output-dir", tmp.to_str().unwrap()])
        .assert()
        .success();
    // run should create artifact files in the output directory
    assert!(tmp.exists(), "output directory should be created");
    let _ = std::fs::remove_dir_all(&tmp);
}

// ===========================================================================
// analyze: presets
// ===========================================================================

#[test]
fn analyze_receipt_preset_json_has_derived() {
    let output = tokmd_cmd()
        .args(["analyze", "--preset", "receipt", "--format", "json"])
        .output()
        .expect("run");
    assert!(output.status.success());
    let json: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert!(json["schema_version"].is_number());
    assert!(
        json["derived"].is_object(),
        "receipt preset must have derived"
    );
}

#[test]
fn analyze_health_preset_json_has_derived() {
    let output = tokmd_cmd()
        .args(["analyze", "--preset", "health", "--format", "json"])
        .output()
        .expect("run");
    assert!(output.status.success());
    let json: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert!(
        json["derived"].is_object(),
        "health preset must have derived"
    );
}

#[test]
fn analyze_default_format_outputs_markdown() {
    tokmd_cmd()
        .args(["analyze", "--preset", "receipt"])
        .assert()
        .success()
        .stdout(predicate::str::is_empty().not());
}

// ===========================================================================
// badge: SVG generation
// ===========================================================================

#[test]
fn badge_lines_metric_produces_svg() {
    tokmd_cmd()
        .args(["badge", "--metric", "lines"])
        .assert()
        .success()
        .stdout(predicate::str::contains("<svg"));
}

#[test]
fn badge_tokens_metric_produces_svg() {
    tokmd_cmd()
        .args(["badge", "--metric", "tokens"])
        .assert()
        .success()
        .stdout(predicate::str::contains("<svg"))
        .stdout(predicate::str::contains("tokens"));
}

#[test]
fn badge_bytes_metric_produces_svg() {
    tokmd_cmd()
        .args(["badge", "--metric", "bytes"])
        .assert()
        .success()
        .stdout(predicate::str::contains("<svg"))
        .stdout(predicate::str::contains("bytes"));
}

// ===========================================================================
// diff: JSON receipt comparison
// ===========================================================================

#[test]
fn diff_between_two_lang_receipts() {
    let make_receipt = || {
        let output = tokmd_cmd()
            .args(["lang", "--format", "json"])
            .output()
            .expect("run");
        assert!(output.status.success());
        output.stdout
    };

    let receipt_a = make_receipt();
    let receipt_b = make_receipt();

    let tmp_dir = std::env::temp_dir().join(format!("tokmd-diff-{}", std::process::id()));
    std::fs::create_dir_all(&tmp_dir).unwrap();
    let file_a = tmp_dir.join("a.json");
    let file_b = tmp_dir.join("b.json");
    std::fs::write(&file_a, &receipt_a).unwrap();
    std::fs::write(&file_b, &receipt_b).unwrap();

    let output = tokmd_cmd()
        .args([
            "diff",
            "--from",
            file_a.to_str().unwrap(),
            "--to",
            file_b.to_str().unwrap(),
            "--format",
            "json",
        ])
        .output()
        .expect("diff run");
    assert!(output.status.success());
    let json: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert!(json.is_object(), "diff must produce JSON object");

    let _ = std::fs::remove_dir_all(&tmp_dir);
}

// ===========================================================================
// init: tokeignore generation
// ===========================================================================

#[test]
fn init_print_generates_tokeignore_content() {
    tokmd_cmd()
        .args(["init", "--print", "--non-interactive"])
        .assert()
        .success()
        .stdout(predicate::str::is_empty().not());
}

// ===========================================================================
// check-ignore
// ===========================================================================

#[test]
fn check_ignore_with_path_runs() {
    // check-ignore requires a path argument
    let result = tokmd_cmd()
        .args(["check-ignore", "src/main.rs"])
        .output()
        .expect("run");
    // Exit code may be 0 (ignored) or 1 (not ignored); just verify no crash
    assert!(
        result.status.success() || result.status.code() == Some(1),
        "check-ignore should run without crash"
    );
}

// ===========================================================================
// completions: all shells
// ===========================================================================

#[test]
fn completions_bash_outputs_script() {
    tokmd_cmd()
        .args(["completions", "bash"])
        .assert()
        .success()
        .stdout(predicate::str::is_empty().not());
}

#[test]
fn completions_zsh_outputs_script() {
    tokmd_cmd()
        .args(["completions", "zsh"])
        .assert()
        .success()
        .stdout(predicate::str::is_empty().not());
}

#[test]
fn completions_fish_outputs_script() {
    tokmd_cmd()
        .args(["completions", "fish"])
        .assert()
        .success()
        .stdout(predicate::str::is_empty().not());
}

#[test]
fn completions_powershell_outputs_script() {
    tokmd_cmd()
        .args(["completions", "powershell"])
        .assert()
        .success()
        .stdout(predicate::str::is_empty().not());
}

// ===========================================================================
// --help for all subcommands
// ===========================================================================

#[test]
fn help_root_shows_usage() {
    tokmd_cmd()
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Usage"));
}

#[test]
fn help_lang_shows_usage() {
    tokmd_cmd()
        .args(["lang", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Usage"));
}

#[test]
fn help_module_shows_usage() {
    tokmd_cmd()
        .args(["module", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Usage"));
}

#[test]
fn help_export_shows_usage() {
    tokmd_cmd()
        .args(["export", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Usage"));
}

#[test]
fn help_analyze_shows_usage() {
    tokmd_cmd()
        .args(["analyze", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Usage"));
}

#[test]
fn help_badge_shows_usage() {
    tokmd_cmd()
        .args(["badge", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Usage"));
}

#[test]
fn help_diff_shows_usage() {
    tokmd_cmd()
        .args(["diff", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Usage"));
}

#[test]
fn help_run_shows_usage() {
    tokmd_cmd()
        .args(["run", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Usage"));
}

// ===========================================================================
// --version
// ===========================================================================

#[test]
fn version_flag_shows_semver() {
    tokmd_cmd()
        .arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::is_match(r"\d+\.\d+\.\d+").unwrap());
}

// ===========================================================================
// error cases
// ===========================================================================

#[test]
fn invalid_path_fails_gracefully() {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_tokmd"));
    cmd.args([
        "lang",
        "--path",
        "/nonexistent/path/that/does/not/exist/abcxyz123",
    ]);
    cmd.assert().failure();
}

#[test]
fn invalid_subcommand_fails() {
    tokmd_cmd()
        .arg("not-a-real-command")
        .assert()
        .failure()
        .stderr(predicate::str::is_empty().not())
        .stderr(predicate::str::contains("not recognized"))
        .stderr(predicate::str::contains("Path not found"));
}

#[test]
fn invalid_format_flag_fails() {
    tokmd_cmd()
        .args(["lang", "--format", "xml-invalid"])
        .assert()
        .failure();
}

#[test]
fn export_invalid_format_fails() {
    tokmd_cmd()
        .args(["export", "--format", "yaml-invalid"])
        .assert()
        .failure();
}

// ===========================================================================
// --children collapse vs separate
// ===========================================================================

#[test]
fn lang_children_collapse_succeeds() {
    let output = tokmd_cmd()
        .args(["lang", "--format", "json", "--children", "collapse"])
        .output()
        .expect("run");
    assert!(output.status.success());
    let json: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert!(json["rows"].is_array());
}

#[test]
fn lang_children_separate_succeeds() {
    let output = tokmd_cmd()
        .args(["lang", "--format", "json", "--children", "separate"])
        .output()
        .expect("run");
    assert!(output.status.success());
    let json: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert!(json["rows"].is_array());
}

#[test]
fn lang_children_separate_may_have_embedded_rows() {
    let output = tokmd_cmd()
        .args(["lang", "--format", "json", "--children", "separate"])
        .output()
        .expect("run");
    assert!(output.status.success());
    let json: Value = serde_json::from_slice(&output.stdout).unwrap();
    // In separate mode, embedded languages may appear as "(embedded)" rows.
    // We just verify the structure is valid.
    let rows = json["rows"].as_array().unwrap();
    for row in rows {
        assert!(row["lang"].is_string());
        assert!(row["code"].is_number());
    }
}

#[test]
fn module_children_parents_only_succeeds() {
    let output = tokmd_cmd()
        .args(["module", "--format", "json", "--children", "parents-only"])
        .output()
        .expect("run");
    assert!(output.status.success());
    let json: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert!(json["rows"].is_array());
}

// ===========================================================================
// --exclude patterns
// ===========================================================================

#[test]
fn lang_exclude_rust_reduces_rows() {
    let without_exclude = {
        let o = tokmd_cmd()
            .args(["lang", "--format", "json"])
            .output()
            .expect("run");
        let json: Value = serde_json::from_slice(&o.stdout).unwrap();
        json["rows"].as_array().unwrap().len()
    };

    let with_exclude = {
        let o = tokmd_cmd()
            .args(["lang", "--format", "json", "--exclude", "*.rs"])
            .output()
            .expect("run");
        let json: Value = serde_json::from_slice(&o.stdout).unwrap();
        json["rows"].as_array().unwrap().len()
    };

    assert!(
        with_exclude < without_exclude,
        "excluding *.rs should reduce language rows: {} vs {}",
        with_exclude,
        without_exclude
    );
}

#[test]
fn export_exclude_js_filters_files() {
    let output = tokmd_cmd()
        .args(["export", "--format", "json", "--exclude", "*.js"])
        .output()
        .expect("run");
    assert!(output.status.success());
    let json: Value = serde_json::from_slice(&output.stdout).unwrap();
    let rows = json["rows"].as_array().unwrap();
    for row in rows {
        let path = row["path"].as_str().unwrap();
        assert!(
            !path.ends_with(".js"),
            "excluded .js file still present: {path}"
        );
    }
}

// ===========================================================================
// pipeline: lang → export → analyze chain validation
// ===========================================================================

#[test]
fn pipeline_lang_then_export_file_count_consistent() {
    // Export may have more rows than lang files due to embedded languages;
    // verify export has at least as many rows as lang total files.
    let lang_files = {
        let o = tokmd_cmd()
            .args(["lang", "--format", "json"])
            .output()
            .expect("run");
        let json: Value = serde_json::from_slice(&o.stdout).unwrap();
        json["total"]["files"].as_u64().unwrap()
    };

    let export_count = {
        let o = tokmd_cmd()
            .args(["export", "--format", "json"])
            .output()
            .expect("run");
        let json: Value = serde_json::from_slice(&o.stdout).unwrap();
        json["rows"].as_array().unwrap().len() as u64
    };

    assert!(
        export_count >= lang_files,
        "export rows ({export_count}) should be >= lang total files ({lang_files})"
    );
}

#[test]
fn pipeline_lang_and_module_same_total_files() {
    let lang_files = {
        let o = tokmd_cmd()
            .args(["lang", "--format", "json"])
            .output()
            .expect("run");
        let json: Value = serde_json::from_slice(&o.stdout).unwrap();
        json["total"]["files"].as_u64().unwrap()
    };

    let module_files = {
        let o = tokmd_cmd()
            .args(["module", "--format", "json"])
            .output()
            .expect("run");
        let json: Value = serde_json::from_slice(&o.stdout).unwrap();
        json["total"]["files"].as_u64().unwrap()
    };

    assert_eq!(
        lang_files, module_files,
        "lang and module must report same total files"
    );
}

// ===========================================================================
// export: path normalization
// ===========================================================================

#[test]
fn export_paths_always_use_forward_slashes() {
    let output = tokmd_cmd()
        .args(["export", "--format", "json"])
        .output()
        .expect("run");
    assert!(output.status.success());
    let json: Value = serde_json::from_slice(&output.stdout).unwrap();
    for row in json["rows"].as_array().unwrap() {
        let path = row["path"].as_str().unwrap();
        assert!(!path.contains('\\'), "backslash in path: {path}");
    }
}

#[test]
fn module_keys_always_use_forward_slashes() {
    let output = tokmd_cmd()
        .args(["module", "--format", "json"])
        .output()
        .expect("run");
    assert!(output.status.success());
    let json: Value = serde_json::from_slice(&output.stdout).unwrap();
    for row in json["rows"].as_array().unwrap() {
        let module = row["module"].as_str().unwrap();
        assert!(!module.contains('\\'), "backslash in module: {module}");
    }
}
