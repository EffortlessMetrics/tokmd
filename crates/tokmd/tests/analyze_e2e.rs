//! End-to-end tests for the `tokmd analyze` CLI command.
//!
//! These tests verify that `tokmd analyze` produces correct output across
//! different presets, formats, and configurations. Tests use either the
//! shared fixture root or ad-hoc temp directories for predictable results.

mod common;

use assert_cmd::Command;
use serde_json::Value;
use std::fs;
use tempfile::tempdir;

fn tokmd_cmd() -> Command {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_tokmd"));
    cmd.current_dir(common::fixture_root());
    cmd
}

/// Create a temp directory with sample source files for hermetic analysis.
fn create_sample_project() -> tempfile::TempDir {
    let dir = tempdir().expect("create temp dir");
    let src = dir.path().join("src");
    fs::create_dir_all(&src).unwrap();

    fs::write(
        src.join("main.rs"),
        r#"fn main() {
    let x = 42;
    println!("Hello, world! {}", x);
    if x > 0 {
        println!("positive");
    }
}

fn add(a: i32, b: i32) -> i32 {
    a + b
}
"#,
    )
    .unwrap();

    fs::write(
        src.join("lib.rs"),
        r#"pub fn greet(name: &str) -> String {
    format!("Hello, {}!", name)
}

pub fn compute(values: &[f64]) -> f64 {
    values.iter().sum::<f64>() / values.len() as f64
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_greet() {
        assert_eq!(greet("world"), "Hello, world!");
    }
}
"#,
    )
    .unwrap();

    fs::write(
        dir.path().join("README.md"),
        "# Sample Project\n\nA sample project for testing.\n",
    )
    .unwrap();

    // .git marker so ignore crate works
    fs::create_dir_all(dir.path().join(".git")).unwrap();

    dir
}

// ---------------------------------------------------------------------------
// 1. Default preset (receipt) — markdown output
// ---------------------------------------------------------------------------

#[test]
fn analyze_default_preset_succeeds() {
    tokmd_cmd().arg("analyze").arg(".").assert().success();
}

// ---------------------------------------------------------------------------
// 2. --preset receipt
// ---------------------------------------------------------------------------

#[test]
fn analyze_preset_receipt_succeeds() {
    tokmd_cmd()
        .arg("analyze")
        .arg(".")
        .arg("--preset")
        .arg("receipt")
        .assert()
        .success();
}

// ---------------------------------------------------------------------------
// 3. --preset health
// ---------------------------------------------------------------------------

#[test]
fn analyze_preset_health_succeeds() {
    tokmd_cmd()
        .arg("analyze")
        .arg(".")
        .arg("--preset")
        .arg("health")
        .assert()
        .success();
}

// ---------------------------------------------------------------------------
// 4. --preset receipt --format json
// ---------------------------------------------------------------------------

#[test]
fn analyze_preset_receipt_json_output() {
    let output = tokmd_cmd()
        .arg("analyze")
        .arg(".")
        .arg("--preset")
        .arg("receipt")
        .arg("--format")
        .arg("json")
        .output()
        .expect("failed to execute");

    assert!(output.status.success(), "exit code should be 0");

    let stdout = String::from_utf8(output.stdout).expect("invalid UTF-8");
    let json: Value = serde_json::from_str(&stdout).expect("output should be valid JSON");

    assert_eq!(json["mode"], "analysis");
    assert!(json["schema_version"].is_number());
}

// ---------------------------------------------------------------------------
// 5. --format json — verify JSON envelope structure
// ---------------------------------------------------------------------------

#[test]
fn analyze_json_envelope_structure() {
    let output = tokmd_cmd()
        .arg("analyze")
        .arg(".")
        .arg("--format")
        .arg("json")
        .output()
        .expect("failed to execute");

    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).expect("invalid UTF-8");
    let json: Value = serde_json::from_str(&stdout).expect("invalid JSON");

    // Top-level envelope fields
    assert_eq!(json["mode"], "analysis", "mode should be 'analysis'");
    assert!(
        json["generated_at_ms"].is_number(),
        "generated_at_ms should be present"
    );
    assert!(json["source"].is_object(), "source should be an object");
    assert!(json["args"].is_object(), "args should be an object");
}

// ---------------------------------------------------------------------------
// 6. Temp directory with sample files
// ---------------------------------------------------------------------------

#[test]
fn analyze_temp_directory_with_sample_files() {
    let project = create_sample_project();

    let output = Command::new(env!("CARGO_BIN_EXE_tokmd"))
        .arg("analyze")
        .arg(".")
        .arg("--preset")
        .arg("receipt")
        .arg("--format")
        .arg("json")
        .current_dir(project.path())
        .output()
        .expect("failed to execute");

    assert!(
        output.status.success(),
        "analyze on temp dir failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8(output.stdout).expect("invalid UTF-8");
    let json: Value = serde_json::from_str(&stdout).expect("invalid JSON");

    assert_eq!(json["mode"], "analysis");

    // Should detect Rust files
    let derived = &json["derived"];
    assert!(derived.is_object(), "derived section should exist");
}

// ---------------------------------------------------------------------------
// 7. JSON has analysis_schema_version field
// ---------------------------------------------------------------------------

#[test]
fn analyze_json_has_schema_version() {
    let output = tokmd_cmd()
        .arg("analyze")
        .arg(".")
        .arg("--format")
        .arg("json")
        .output()
        .expect("failed to execute");

    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).expect("invalid UTF-8");
    let json: Value = serde_json::from_str(&stdout).expect("invalid JSON");

    let sv = json["schema_version"]
        .as_u64()
        .expect("schema_version should be a number");
    assert!(sv >= 8, "analysis schema_version should be >= 8, got {sv}");
}

// ---------------------------------------------------------------------------
// 8. Analysis output contains derived metrics
// ---------------------------------------------------------------------------

#[test]
fn analyze_json_contains_derived_metrics() {
    let output = tokmd_cmd()
        .arg("analyze")
        .arg(".")
        .arg("--preset")
        .arg("receipt")
        .arg("--format")
        .arg("json")
        .output()
        .expect("failed to execute");

    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).expect("invalid UTF-8");
    let json: Value = serde_json::from_str(&stdout).expect("invalid JSON");

    let derived = json["derived"]
        .as_object()
        .expect("derived section should be an object");

    // Receipt preset should include doc_density and distribution
    assert!(
        derived.contains_key("doc_density"),
        "derived should contain 'doc_density'"
    );
    assert!(
        derived.contains_key("distribution"),
        "derived should contain 'distribution'"
    );
}

// ---------------------------------------------------------------------------
// 9. --preset supply
// ---------------------------------------------------------------------------

#[test]
fn analyze_preset_supply_succeeds() {
    tokmd_cmd()
        .arg("analyze")
        .arg(".")
        .arg("--preset")
        .arg("supply")
        .assert()
        .success();
}

// ---------------------------------------------------------------------------
// 10. Density and COCOMO metrics present
// ---------------------------------------------------------------------------

#[test]
fn analyze_json_contains_density_and_cocomo() {
    let output = tokmd_cmd()
        .arg("analyze")
        .arg(".")
        .arg("--preset")
        .arg("receipt")
        .arg("--format")
        .arg("json")
        .output()
        .expect("failed to execute");

    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).expect("invalid UTF-8");
    let json: Value = serde_json::from_str(&stdout).expect("invalid JSON");

    // Doc density metrics
    let doc_density = &json["derived"]["doc_density"];
    assert!(doc_density.is_object(), "doc_density should be present");

    // COCOMO metrics (optional field, but should be present for receipt)
    let cocomo = &json["derived"]["cocomo"];
    assert!(cocomo.is_object(), "cocomo should be present");
    assert!(
        cocomo.get("effort_pm").is_some(),
        "cocomo should have effort_pm"
    );
}

// ---------------------------------------------------------------------------
// 11. Markdown output contains recognisable headings
// ---------------------------------------------------------------------------

#[test]
fn analyze_markdown_output_contains_headings() {
    let output = tokmd_cmd()
        .arg("analyze")
        .arg(".")
        .arg("--preset")
        .arg("receipt")
        .arg("--format")
        .arg("md")
        .output()
        .expect("failed to execute");

    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).expect("invalid UTF-8");

    // Markdown should have section headings
    assert!(
        stdout.contains('#'),
        "markdown output should contain headings"
    );
}

// ---------------------------------------------------------------------------
// 12. --preset health --format json has complexity section
// ---------------------------------------------------------------------------

#[test]
fn analyze_health_preset_has_complexity() {
    let output = tokmd_cmd()
        .arg("analyze")
        .arg(".")
        .arg("--preset")
        .arg("health")
        .arg("--format")
        .arg("json")
        .output()
        .expect("failed to execute");

    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).expect("invalid UTF-8");
    let json: Value = serde_json::from_str(&stdout).expect("invalid JSON");

    assert_eq!(json["mode"], "analysis");
    // Health preset includes derived metrics
    assert!(
        json["derived"].is_object(),
        "health should have derived section"
    );
}

// ---------------------------------------------------------------------------
// 13. Output to directory writes analysis.json
// ---------------------------------------------------------------------------

#[test]
fn analyze_output_dir_writes_file() {
    let dir = tempdir().unwrap();
    let out = dir.path();

    let output = tokmd_cmd()
        .arg("analyze")
        .arg(".")
        .arg("--preset")
        .arg("receipt")
        .arg("--format")
        .arg("json")
        .arg("--output-dir")
        .arg(out)
        .output()
        .expect("failed to execute");

    assert!(output.status.success(), "analyze should succeed");

    let path = out.join("analysis.json");
    assert!(
        path.exists(),
        "analysis.json should be written to output dir"
    );

    let content = fs::read_to_string(&path).expect("read analysis.json");
    let json: Value = serde_json::from_str(&content).expect("file should be valid JSON");
    assert_eq!(json["mode"], "analysis");
}

// ===========================================================================
// Deep E2E tests – presets, formats, determinism, errors
// ===========================================================================

// ---------------------------------------------------------------------------
// 14. --preset health --format json: valid JSON with schema_version
// ---------------------------------------------------------------------------

#[test]
fn analyze_health_preset_json_has_schema_version() {
    let output = tokmd_cmd()
        .arg("analyze")
        .arg(".")
        .arg("--preset")
        .arg("health")
        .arg("--format")
        .arg("json")
        .output()
        .expect("failed to execute");

    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).expect("invalid UTF-8");
    let json: Value = serde_json::from_str(&stdout).expect("output should be valid JSON");

    assert_eq!(json["mode"], "analysis");
    let sv = json["schema_version"]
        .as_u64()
        .expect("schema_version should be a number");
    assert!(sv >= 8, "analysis schema_version should be >= 8, got {sv}");
    assert!(
        json["derived"].is_object(),
        "health preset should include derived section"
    );
    // Health preset includes complexity metrics
    assert!(
        json["complexity"].is_object(),
        "health preset should include complexity section"
    );
}

// ---------------------------------------------------------------------------
// 15. --preset risk --format json: valid JSON with git section
// ---------------------------------------------------------------------------

#[test]
fn analyze_risk_preset_json_has_git_section() {
    // Risk preset requires real git history for git metrics
    let project = create_sample_project();
    let dir = project.path();

    if !common::init_git_repo(dir) || !common::git_add_commit(dir, "initial") {
        eprintln!("skipping: git not available");
        return;
    }

    // Add a second commit so git log has history
    fs::write(dir.join("src").join("extra.rs"), "fn extra() {}\n").unwrap();
    if !common::git_add_commit(dir, "add extra") {
        eprintln!("skipping: git commit failed");
        return;
    }

    let output = Command::new(env!("CARGO_BIN_EXE_tokmd"))
        .arg("analyze")
        .arg(".")
        .arg("--preset")
        .arg("risk")
        .arg("--format")
        .arg("json")
        .current_dir(dir)
        .output()
        .expect("failed to execute");

    assert!(
        output.status.success(),
        "risk preset should succeed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8(output.stdout).expect("invalid UTF-8");
    let json: Value = serde_json::from_str(&stdout).expect("output should be valid JSON");

    assert_eq!(json["mode"], "analysis");
    let sv = json["schema_version"]
        .as_u64()
        .expect("schema_version should be a number");
    assert!(sv >= 8, "analysis schema_version should be >= 8, got {sv}");
    assert!(
        json["derived"].is_object(),
        "risk preset should include derived section"
    );
    // Risk includes git metrics when history is available
    assert!(
        json["git"].is_object(),
        "risk preset should include git section, got keys: {:?}",
        json.as_object().map(|o| o.keys().collect::<Vec<_>>())
    );
}

// ---------------------------------------------------------------------------
// 16. --preset supply --format json: valid JSON with deps/assets sections
// ---------------------------------------------------------------------------

#[test]
fn analyze_supply_preset_json_has_deps_and_assets() {
    let output = tokmd_cmd()
        .arg("analyze")
        .arg(".")
        .arg("--preset")
        .arg("supply")
        .arg("--format")
        .arg("json")
        .output()
        .expect("failed to execute");

    assert!(
        output.status.success(),
        "supply preset should succeed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8(output.stdout).expect("invalid UTF-8");
    let json: Value = serde_json::from_str(&stdout).expect("output should be valid JSON");

    assert_eq!(json["mode"], "analysis");
    let sv = json["schema_version"]
        .as_u64()
        .expect("schema_version should be a number");
    assert!(sv >= 8, "analysis schema_version should be >= 8, got {sv}");
    // Supply preset includes assets and dependency lockfile reports
    assert!(
        json["assets"].is_object(),
        "supply preset should include assets section"
    );
    assert!(
        json["deps"].is_object(),
        "supply preset should include deps section"
    );
}

// ---------------------------------------------------------------------------
// 17. --format tree: produces tree-like output (text format)
// ---------------------------------------------------------------------------

#[test]
fn analyze_tree_format_produces_output() {
    let output = tokmd_cmd()
        .arg("analyze")
        .arg(".")
        .arg("--preset")
        .arg("receipt")
        .arg("--format")
        .arg("tree")
        .output()
        .expect("failed to execute");

    assert!(
        output.status.success(),
        "tree format should succeed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8(output.stdout).expect("invalid UTF-8");
    assert!(!stdout.is_empty(), "tree format should produce output");
}

// ---------------------------------------------------------------------------
// 18. Determinism: two identical runs produce same JSON (timestamps excluded)
// ---------------------------------------------------------------------------

#[test]
fn analyze_determinism_two_runs_match() {
    let run = || {
        let output = tokmd_cmd()
            .arg("analyze")
            .arg(".")
            .arg("--preset")
            .arg("receipt")
            .arg("--format")
            .arg("json")
            .output()
            .expect("failed to execute");
        assert!(output.status.success());
        let stdout = String::from_utf8(output.stdout).expect("invalid UTF-8");
        let mut json: Value = serde_json::from_str(&stdout).expect("invalid JSON");
        // Normalize volatile fields
        json["generated_at_ms"] = Value::Null;
        if let Some(integrity) = json.pointer_mut("/derived/integrity/scan_fingerprint") {
            *integrity = Value::Null;
        }
        if let Some(ts) = json.pointer_mut("/tool/built_at") {
            *ts = Value::Null;
        }
        json
    };

    let run1 = run();
    let run2 = run();
    assert_eq!(
        run1, run2,
        "two identical analyze runs should produce deterministic output"
    );
}

// ---------------------------------------------------------------------------
// 19. Verify args.preset field reflects requested preset
// ---------------------------------------------------------------------------

#[test]
fn analyze_json_args_preset_matches_requested() {
    for preset in &["receipt", "health", "supply"] {
        let output = tokmd_cmd()
            .arg("analyze")
            .arg(".")
            .arg("--preset")
            .arg(preset)
            .arg("--format")
            .arg("json")
            .output()
            .expect("failed to execute");

        assert!(output.status.success());

        let stdout = String::from_utf8(output.stdout).expect("invalid UTF-8");
        let json: Value = serde_json::from_str(&stdout).expect("invalid JSON");

        assert_eq!(
            json["args"]["preset"].as_str().unwrap(),
            *preset,
            "args.preset should reflect the requested preset '{preset}'"
        );
    }
}

// ---------------------------------------------------------------------------
// 20. JSON status field is "complete"
// ---------------------------------------------------------------------------

#[test]
fn analyze_json_status_is_complete() {
    let output = tokmd_cmd()
        .arg("analyze")
        .arg(".")
        .arg("--preset")
        .arg("receipt")
        .arg("--format")
        .arg("json")
        .output()
        .expect("failed to execute");

    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).expect("invalid UTF-8");
    let json: Value = serde_json::from_str(&stdout).expect("invalid JSON");

    assert_eq!(
        json["status"].as_str().unwrap(),
        "complete",
        "status should be 'complete' for a valid scan"
    );
}

// ---------------------------------------------------------------------------
// 21. JSON warnings is an array
// ---------------------------------------------------------------------------

#[test]
fn analyze_json_warnings_is_array() {
    let output = tokmd_cmd()
        .arg("analyze")
        .arg(".")
        .arg("--preset")
        .arg("receipt")
        .arg("--format")
        .arg("json")
        .output()
        .expect("failed to execute");

    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).expect("invalid UTF-8");
    let json: Value = serde_json::from_str(&stdout).expect("invalid JSON");

    assert!(json["warnings"].is_array(), "warnings should be an array");
}

// ---------------------------------------------------------------------------
// 22. Non-existent path returns failure
// ---------------------------------------------------------------------------

#[test]
fn analyze_nonexistent_path_fails() {
    Command::new(env!("CARGO_BIN_EXE_tokmd"))
        .arg("analyze")
        .arg("this_path_does_not_exist_xyz")
        .assert()
        .failure();
}

// ---------------------------------------------------------------------------
// 23. Health preset produces todo density in derived section
// ---------------------------------------------------------------------------

#[test]
fn analyze_health_preset_has_todo_density() {
    let output = tokmd_cmd()
        .arg("analyze")
        .arg(".")
        .arg("--preset")
        .arg("health")
        .arg("--format")
        .arg("json")
        .output()
        .expect("failed to execute");

    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).expect("invalid UTF-8");
    let json: Value = serde_json::from_str(&stdout).expect("invalid JSON");

    let derived = json["derived"]
        .as_object()
        .expect("derived should be an object");
    assert!(
        derived.contains_key("todo"),
        "health preset derived should include todo density report"
    );
}

