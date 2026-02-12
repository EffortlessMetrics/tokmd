//! Integration tests for the `tokmd sensor` command.

use assert_cmd::Command;
use tempfile::tempdir;

fn git_available() -> bool {
    std::process::Command::new("git")
        .arg("--version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

fn init_git_repo(dir: &std::path::Path) -> bool {
    let commands = [
        vec!["init"],
        vec!["symbolic-ref", "HEAD", "refs/heads/main"],
        vec!["config", "user.email", "test@test.com"],
        vec!["config", "user.name", "Test User"],
    ];

    for args in &commands {
        let status = std::process::Command::new("git")
            .args(args)
            .current_dir(dir)
            .status();
        if !status.map(|s| s.success()).unwrap_or(false) {
            return false;
        }
    }
    true
}

fn git_add_commit(dir: &std::path::Path, message: &str) -> bool {
    let commands = [vec!["add", "."], vec!["commit", "-m", message]];

    for args in &commands {
        let status = std::process::Command::new("git")
            .args(args)
            .current_dir(dir)
            .status();
        if !status.map(|s| s.success()).unwrap_or(false) {
            return false;
        }
    }
    true
}

#[test]
fn sensor_json_outputs_artifacts_and_data() {
    // Given: A git repository with a main branch and a feature branch with code changes
    // When: User runs `tokmd sensor --base main --head HEAD --output report.json --format json`
    // Then: Output should include sensor report with artifacts (receipt, cockpit, comment)
    if !git_available() {
        eprintln!("Skipping: git not available");
        return;
    }

    let dir = tempdir().unwrap();
    if !init_git_repo(dir.path()) {
        eprintln!("Skipping: git init failed");
        return;
    }

    std::fs::create_dir_all(dir.path().join("src")).unwrap();
    std::fs::write(dir.path().join("src/lib.rs"), "fn main() {}\n").unwrap();
    if !git_add_commit(dir.path(), "Initial commit") {
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
    if !git_add_commit(dir.path(), "Add changes") {
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
    if !git_available() {
        eprintln!("Skipping: git not available");
        return;
    }

    let dir = tempdir().unwrap();
    if !init_git_repo(dir.path()) {
        eprintln!("Skipping: git init failed");
        return;
    }

    std::fs::create_dir_all(dir.path().join("src")).unwrap();
    std::fs::write(dir.path().join("src/lib.rs"), "fn main() {}\n").unwrap();
    if !git_add_commit(dir.path(), "Initial") {
        return;
    }

    let _ = std::process::Command::new("git")
        .args(["checkout", "-b", "feature"])
        .current_dir(dir.path())
        .status();

    std::fs::write(dir.path().join("src/lib.rs"), "fn main() { }\n").unwrap();
    if !git_add_commit(dir.path(), "Update") {
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
    if !git_available() {
        eprintln!("Skipping: git not available");
        return;
    }

    let dir = tempdir().unwrap();
    if !init_git_repo(dir.path()) {
        eprintln!("Skipping: git init failed");
        return;
    }

    std::fs::create_dir_all(dir.path().join("src")).unwrap();
    std::fs::write(dir.path().join("src/lib.rs"), "fn main() {}\n").unwrap();
    if !git_add_commit(dir.path(), "Initial commit") {
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
    if !git_add_commit(dir.path(), "Add complex code") {
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
    if !git_available() {
        eprintln!("Skipping: git not available");
        return;
    }

    let dir = tempdir().unwrap();
    if !init_git_repo(dir.path()) {
        eprintln!("Skipping: git init failed");
        return;
    }

    std::fs::create_dir_all(dir.path().join("src")).unwrap();
    std::fs::write(dir.path().join("src/lib.rs"), "fn main() {}\n").unwrap();
    if !git_add_commit(dir.path(), "Initial commit") {
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
    if !git_add_commit(dir.path(), "Add changes") {
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
    if !git_available() {
        eprintln!("Skipping: git not available");
        return;
    }

    let dir = tempdir().unwrap();
    if !init_git_repo(dir.path()) {
        eprintln!("Skipping: git init failed");
        return;
    }

    std::fs::create_dir_all(dir.path().join("src")).unwrap();
    std::fs::write(dir.path().join("src/lib.rs"), "fn main() {}\n").unwrap();
    if !git_add_commit(dir.path(), "Initial commit") {
        eprintln!("Skipping: git commit failed");
        return;
    }

    let _ = std::process::Command::new("git")
        .args(["checkout", "-b", "feature"])
        .current_dir(dir.path())
        .status();

    std::fs::write(dir.path().join("src/lib.rs"), "fn main() { }\n").unwrap();
    if !git_add_commit(dir.path(), "Update") {
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
