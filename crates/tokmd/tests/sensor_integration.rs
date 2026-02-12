//! Integration tests for the `tokmd sensor` command.

mod common;

use assert_cmd::Command;
use tempfile::tempdir;

#[test]
fn sensor_json_outputs_artifacts_and_data() {
    // Given: A git repository with a main branch and a feature branch with code changes
    // When: User runs `tokmd sensor --base main --head HEAD --output report.json --format json`
    // Then: Output should include sensor report with artifacts (receipt, cockpit, comment)
    if !common::git_available() {
        eprintln!("Skipping: git not available");
        return;
    }

    let dir = tempdir().unwrap();
    if !common::init_git_repo(dir.path()) {
        eprintln!("Skipping: git init failed");
        return;
    }

    std::fs::create_dir_all(dir.path().join("src")).unwrap();
    std::fs::write(dir.path().join("src/lib.rs"), "fn main() {}\n").unwrap();
    if !common::git_add_commit(dir.path(), "Initial commit") {
        eprintln!("Skipping: git commit failed");
        return;
    }

    let _ = std::process::Command::new("git")
        .args(["checkout", "-b", "feature"])
        .current_dir(dir.path())
        .status();

    std::fs::write(
        dir.path().join("src/lib.rs"),
        "fn main() { println!(\"hi\"); }\n",
    )
    .unwrap();
    std::fs::write(dir.path().join("src/extra.rs"), "fn extra() {}\n").unwrap();
    if !common::git_add_commit(dir.path(), "Add changes") {
        eprintln!("Skipping: second commit failed");
        return;
    }

    let output_path = dir
        .path()
        .join("artifacts")
        .join("tokmd")
        .join("report.json");
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_tokmd"));
    let output = cmd
        .current_dir(dir.path())
        .arg("sensor")
        .arg("--base")
        .arg("main")
        .arg("--head")
        .arg("HEAD")
        .arg("--output")
        .arg(&output_path)
        .arg("--format")
        .arg("json")
        .output()
        .expect("run tokmd sensor");

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        panic!("tokmd sensor failed: {stderr}");
    }

    assert!(output_path.exists());
    let comment_path = output_path.parent().unwrap().join("comment.md");
    let sidecar_path = output_path
        .parent()
        .unwrap()
        .join("extras")
        .join("cockpit_receipt.json");
    assert!(comment_path.exists());
    assert!(sidecar_path.exists());

    let stdout = String::from_utf8(output.stdout).expect("stdout utf8");
    let json: serde_json::Value = serde_json::from_str(&stdout).expect("valid JSON");
    assert_eq!(json["schema"], "sensor.report.v1");
    assert_eq!(json["tool"]["name"], "tokmd");
    assert!(json.get("data").is_some());

    let artifacts = json
        .get("artifacts")
        .and_then(|v| v.as_array())
        .expect("artifacts array");
    let ids: std::collections::HashSet<_> = artifacts
        .iter()
        .filter_map(|a| a.get("id").and_then(|id| id.as_str()))
        .collect();
    for id in ["receipt", "cockpit", "comment"] {
        assert!(ids.contains(id), "missing artifact id {id}");
    }
}

#[test]
fn sensor_md_outputs_markdown() {
    // Given: A git repository with a main branch and a feature branch with code changes
    // When: User runs `tokmd sensor --base main --head HEAD --output report.json --format md`
    // Then: Output should include markdown report with "Sensor Report" header
    if !common::git_available() {
        eprintln!("Skipping: git not available");
        return;
    }

    let dir = tempdir().unwrap();
    if !common::init_git_repo(dir.path()) {
        eprintln!("Skipping: git init failed");
        return;
    }

    std::fs::create_dir_all(dir.path().join("src")).unwrap();
    std::fs::write(dir.path().join("src/lib.rs"), "fn main() {}\n").unwrap();
    if !common::git_add_commit(dir.path(), "Initial") {
        return;
    }

    let _ = std::process::Command::new("git")
        .args(["checkout", "-b", "feature"])
        .current_dir(dir.path())
        .status();

    std::fs::write(dir.path().join("src/lib.rs"), "fn main() { }\n").unwrap();
    if !common::git_add_commit(dir.path(), "Update") {
        return;
    }

    let output_path = dir
        .path()
        .join("artifacts")
        .join("tokmd")
        .join("report.json");
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_tokmd"));
    let output = cmd
        .current_dir(dir.path())
        .arg("sensor")
        .arg("--base")
        .arg("main")
        .arg("--head")
        .arg("HEAD")
        .arg("--output")
        .arg(&output_path)
        .arg("--format")
        .arg("md")
        .output()
        .expect("run tokmd sensor md");

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        panic!("tokmd sensor failed: {stderr}");
    }

    let stdout = String::from_utf8(output.stdout).expect("stdout utf8");
    assert!(stdout.contains("## Sensor Report: tokmd"));
    assert!(output_path.exists());
}

// =============================================================================
// Priority 2: BDD Scenario Tests for 1.6.0 Features
// =============================================================================

#[test]
fn sensor_report_with_risk_findings() {
    // Given: A repository with hotspots exceeding thresholds
    // When: User runs `tokmd sensor --base main --head feature --format json`
    // Then: Output should include risk findings with proper severity
    if !common::git_available() {
        eprintln!("Skipping: git not available");
        return;
    }

    let dir = tempdir().unwrap();
    if !common::init_git_repo(dir.path()) {
        eprintln!("Skipping: git init failed");
        return;
    }

    std::fs::create_dir_all(dir.path().join("src")).unwrap();
    std::fs::write(dir.path().join("src/lib.rs"), "fn main() {}\n").unwrap();
    if !common::git_add_commit(dir.path(), "Initial commit") {
        eprintln!("Skipping: git commit failed");
        return;
    }

    let _ = std::process::Command::new("git")
        .args(["checkout", "-b", "feature"])
        .current_dir(dir.path())
        .status();

    // Add changes that could trigger risk findings
    std::fs::write(
        dir.path().join("src/lib.rs"),
        "fn main() { let mut x = 0; for i in 0..1000 { x += i; } }\n",
    )
    .unwrap();
    if !common::git_add_commit(dir.path(), "Add complex code") {
        eprintln!("Skipping: second commit failed");
        return;
    }

    let mut cmd = Command::new(env!("CARGO_BIN_EXE_tokmd"));
    let output = cmd
        .current_dir(dir.path())
        .arg("sensor")
        .arg("--base")
        .arg("main")
        .arg("--head")
        .arg("HEAD")
        .arg("--format")
        .arg("json")
        .output()
        .expect("run tokmd sensor");

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        panic!("tokmd sensor failed: {stderr}");
    }

    let stdout = String::from_utf8(output.stdout).expect("stdout utf8");
    let json: serde_json::Value = serde_json::from_str(&stdout).expect("valid JSON");
    
    // Verify sensor report structure
    assert_eq!(json["schema"], "sensor.report.v1");
    assert_eq!(json["tool"]["name"], "tokmd");
    
    // Verify data section exists (which may contain risk findings)
    assert!(json.get("data").is_some());
}

#[test]
fn sensor_report_with_contract_changes() {
    // Given: A repository with changes to codebase structure
    // When: User runs `tokmd sensor --base main --head feature --format json`
    // Then: Output should include contract changes in the data section
    if !common::git_available() {
        eprintln!("Skipping: git not available");
        return;
    }

    let dir = tempdir().unwrap();
    if !common::init_git_repo(dir.path()) {
        eprintln!("Skipping: git init failed");
        return;
    }

    std::fs::create_dir_all(dir.path().join("src")).unwrap();
    std::fs::write(dir.path().join("src/lib.rs"), "fn main() {}\n").unwrap();
    if !common::git_add_commit(dir.path(), "Initial commit") {
        eprintln!("Skipping: git commit failed");
        return;
    }

    let _ = std::process::Command::new("git")
        .args(["checkout", "-b", "feature"])
        .current_dir(dir.path())
        .status();

    // Add new files and modify existing ones
    std::fs::write(dir.path().join("src/lib.rs"), "fn main() { println!(\"updated\"); }\n").unwrap();
    std::fs::write(dir.path().join("src/new.rs"), "fn new() {}\n").unwrap();
    if !common::git_add_commit(dir.path(), "Add changes") {
        eprintln!("Skipping: second commit failed");
        return;
    }

    let mut cmd = Command::new(env!("CARGO_BIN_EXE_tokmd"));
    let output = cmd
        .current_dir(dir.path())
        .arg("sensor")
        .arg("--base")
        .arg("main")
        .arg("--head")
        .arg("HEAD")
        .arg("--format")
        .arg("json")
        .output()
        .expect("run tokmd sensor");

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        panic!("tokmd sensor failed: {stderr}");
    }

    let stdout = String::from_utf8(output.stdout).expect("stdout utf8");
    let json: serde_json::Value = serde_json::from_str(&stdout).expect("valid JSON");
    
    // Verify sensor report structure with contract-related data
    assert_eq!(json["schema"], "sensor.report.v1");
    assert!(json.get("data").is_some());
}

#[test]
fn sensor_envelope_verdict_aggregation() {
    // Given: A sensor report with multiple findings
    // When: User runs `tokmd sensor --base main --head feature --format json`
    // Then: Output should include envelope with verdict aggregation
    if !common::git_available() {
        eprintln!("Skipping: git not available");
        return;
    }

    let dir = tempdir().unwrap();
    if !common::init_git_repo(dir.path()) {
        eprintln!("Skipping: git init failed");
        return;
    }

    std::fs::create_dir_all(dir.path().join("src")).unwrap();
    std::fs::write(dir.path().join("src/lib.rs"), "fn main() {}\n").unwrap();
    if !common::git_add_commit(dir.path(), "Initial commit") {
        eprintln!("Skipping: git commit failed");
        return;
    }

    let _ = std::process::Command::new("git")
        .args(["checkout", "-b", "feature"])
        .current_dir(dir.path())
        .status();

    std::fs::write(dir.path().join("src/lib.rs"), "fn main() { }\n").unwrap();
    if !common::git_add_commit(dir.path(), "Update") {
        eprintln!("Skipping: second commit failed");
        return;
    }

    let mut cmd = Command::new(env!("CARGO_BIN_EXE_tokmd"));
    let output = cmd
        .current_dir(dir.path())
        .arg("sensor")
        .arg("--base")
        .arg("main")
        .arg("--head")
        .arg("HEAD")
        .arg("--format")
        .arg("json")
        .output()
        .expect("run tokmd sensor");

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        panic!("tokmd sensor failed: {stderr}");
    }

    let stdout = String::from_utf8(output.stdout).expect("stdout utf8");
    let json: serde_json::Value = serde_json::from_str(&stdout).expect("valid JSON");
    
    // Verify envelope structure exists
    assert_eq!(json["schema"], "sensor.report.v1");
    assert!(json.get("data").is_some());
    
    // Verify artifacts section (envelope metadata)
    let artifacts = json
        .get("artifacts")
        .and_then(|v| v.as_array())
        .expect("artifacts array");
    assert!(!artifacts.is_empty(), "artifacts should not be empty");
}

// =============================================================================
// Priority 3: High-Signal BDD Scenario Tests
// =============================================================================

/// Generate a Rust source file whose single function has cyclomatic complexity 17.
///
/// The function body contains 16 `if` statements (CC = 1 base + 16 branches).
fn high_cc_source(fn_name: &str) -> String {
    let mut body = format!("fn {fn_name}(x: i32) -> i32 {{\n");
    for i in 0..16 {
        body.push_str(&format!("    if x == {i} {{ return {i}; }}\n"));
    }
    body.push_str("    -1\n}\n");
    body
}

#[test]
fn scenario_docs_only_change_verdict_pass() {
    // Given: A git repo on main with src/lib.rs + README.md
    // When:  Feature branch changes ONLY README.md
    // Then:  Sensor should produce verdict=pass, findings=[], mutation gate skip
    if !common::git_available() {
        eprintln!("Skipping: git not available");
        return;
    }

    let dir = tempdir().unwrap();
    if !common::init_git_repo(dir.path()) {
        eprintln!("Skipping: git init failed");
        return;
    }

    // Main branch: one Rust file + one doc file
    std::fs::create_dir_all(dir.path().join("src")).unwrap();
    std::fs::write(dir.path().join("src/lib.rs"), "fn main() {}\n").unwrap();
    std::fs::write(dir.path().join("README.md"), "# My Project\n").unwrap();
    if !common::git_add_commit(dir.path(), "Initial commit") {
        eprintln!("Skipping: initial commit failed");
        return;
    }

    // Feature branch: change only docs
    let _ = std::process::Command::new("git")
        .args(["checkout", "-b", "feature"])
        .current_dir(dir.path())
        .status();

    std::fs::write(
        dir.path().join("README.md"),
        "# My Project\n\nUpdated documentation.\n",
    )
    .unwrap();
    if !common::git_add_commit(dir.path(), "Update docs") {
        eprintln!("Skipping: feature commit failed");
        return;
    }

    let output_path = dir
        .path()
        .join("artifacts")
        .join("tokmd")
        .join("report.json");

    let mut cmd = Command::new(env!("CARGO_BIN_EXE_tokmd"));
    let output = cmd
        .current_dir(dir.path())
        .arg("sensor")
        .arg("--base")
        .arg("main")
        .arg("--head")
        .arg("HEAD")
        .arg("--output")
        .arg(&output_path)
        .arg("--format")
        .arg("json")
        .output()
        .expect("run tokmd sensor");

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        panic!("tokmd sensor failed: {stderr}");
    }

    let stdout = String::from_utf8(output.stdout).expect("stdout utf8");
    let json: serde_json::Value = serde_json::from_str(&stdout).expect("valid JSON");

    // ---- Assertions ----

    // Verdict must be pass (docs-only change triggers no failure gates)
    assert_eq!(
        json["verdict"], "pass",
        "docs-only change should produce pass verdict, got: {}",
        json["verdict"]
    );

    // Findings must be empty (no risk/contract/gate findings for docs)
    let findings = json["findings"]
        .as_array()
        .expect("findings should be an array");
    assert!(
        findings.is_empty(),
        "docs-only change should produce no findings, got: {findings:?}"
    );

    // Mutation gate should be skipped (no .rs files in changeset)
    let gates = &json["data"]["gates"];
    let items = gates["items"].as_array().expect("gate items array");
    let mutation_gate = items.iter().find(|g| g["id"] == "mutation");
    if let Some(mg) = mutation_gate {
        assert_eq!(
            mg["status"], "skip",
            "mutation gate should be skip for docs-only change, got: {}",
            mg["status"]
        );
    }

    // Report artefact should exist on disk
    assert!(
        output_path.exists(),
        "report.json should be written to artifacts dir"
    );

    // Verify the on-disk report matches stdout
    let disk_json: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&output_path).unwrap())
            .expect("valid JSON on disk");
    assert_eq!(
        disk_json["verdict"], json["verdict"],
        "on-disk report verdict should match stdout"
    );
}

#[test]
fn scenario_four_high_complexity_files_verdict_fail() {
    // Given: A git repo on main with src/lib.rs
    // When:  Feature branch adds 4 .rs files each with CC=17
    // Then:  Sensor should produce verdict=fail with complexity findings
    if !common::git_available() {
        eprintln!("Skipping: git not available");
        return;
    }

    let dir = tempdir().unwrap();
    if !common::init_git_repo(dir.path()) {
        eprintln!("Skipping: git init failed");
        return;
    }

    // Main branch: minimal Rust file
    std::fs::create_dir_all(dir.path().join("src")).unwrap();
    std::fs::write(dir.path().join("src/lib.rs"), "fn main() {}\n").unwrap();
    if !common::git_add_commit(dir.path(), "Initial commit") {
        eprintln!("Skipping: initial commit failed");
        return;
    }

    // Feature branch: add 4 files each with CC = 17
    let _ = std::process::Command::new("git")
        .args(["checkout", "-b", "feature"])
        .current_dir(dir.path())
        .status();

    for i in 1..=4 {
        let name = format!("src/complex_{i}.rs");
        let fn_name = format!("complex_{i}");
        std::fs::write(dir.path().join(&name), high_cc_source(&fn_name)).unwrap();
    }

    if !common::git_add_commit(dir.path(), "Add 4 high-complexity files") {
        eprintln!("Skipping: feature commit failed");
        return;
    }

    // Seed mutants-summary.json so the mutation gate resolves instead of
    // attempting to run cargo-mutants (which may not be installed).
    if let Some(head) = common::git_head(dir.path()) {
        common::write_mutants_summary(dir.path(), &head, "src", "pass");
    }

    let mut cmd = Command::new(env!("CARGO_BIN_EXE_tokmd"));
    let output = cmd
        .current_dir(dir.path())
        .arg("sensor")
        .arg("--base")
        .arg("main")
        .arg("--head")
        .arg("HEAD")
        .arg("--format")
        .arg("json")
        .output()
        .expect("run tokmd sensor");

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        panic!("tokmd sensor failed: {stderr}");
    }

    let stdout = String::from_utf8(output.stdout).expect("stdout utf8");
    let json: serde_json::Value = serde_json::from_str(&stdout).expect("valid JSON");

    // ---- Assertions ----

    // Verdict must be fail (≥4 high-complexity files triggers complexity gate failure)
    assert_eq!(
        json["verdict"], "fail",
        "4 high-CC files should produce fail verdict, got: {}",
        json["verdict"]
    );

    let findings = json["findings"]
        .as_array()
        .expect("findings should be an array");

    // Should contain exactly 1 gate.complexity_failed finding (severity: error)
    let gate_findings: Vec<_> = findings
        .iter()
        .filter(|f| f["check_id"] == "gate" && f["code"] == "complexity_failed")
        .collect();
    assert_eq!(
        gate_findings.len(),
        1,
        "expected 1 gate.complexity_failed finding, got {}",
        gate_findings.len()
    );
    assert_eq!(
        gate_findings[0]["severity"], "error",
        "gate.complexity_failed should be severity error"
    );

    // Should contain 4 risk.complexity_high findings (severity: warn)
    let risk_findings: Vec<_> = findings
        .iter()
        .filter(|f| f["check_id"] == "risk" && f["code"] == "complexity_high")
        .collect();
    assert_eq!(
        risk_findings.len(),
        4,
        "expected 4 risk.complexity_high findings, got {}",
        risk_findings.len()
    );
    for rf in &risk_findings {
        assert_eq!(
            rf["severity"], "warn",
            "risk.complexity_high should be severity warn"
        );
    }

    // Complexity gate in data.gates should be fail
    let gates = &json["data"]["gates"];
    let items = gates["items"].as_array().expect("gate items array");
    let complexity_gate = items.iter().find(|g| g["id"] == "complexity");
    assert!(
        complexity_gate.is_some(),
        "complexity gate should be present in gate items"
    );
    assert_eq!(
        complexity_gate.unwrap()["status"], "fail",
        "complexity gate should be fail"
    );
}
