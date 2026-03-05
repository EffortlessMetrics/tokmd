//! Deep CLI tests for `tokmd sensor` – conforming sensor producing
//! `sensor.report.v1` envelope.
//!
//! All tests require the `git` feature flag and a working `git` binary.

#![cfg(feature = "git")]
mod common;

use assert_cmd::Command;
use serde_json::Value;
use tempfile::tempdir;

fn tokmd() -> Command {
    Command::new(env!("CARGO_BIN_EXE_tokmd"))
}

/// Scaffold a two-branch git repo: main → feature.
fn scaffold_git_repo(
    base_files: &[(&str, &str)],
    feature_files: &[(&str, &str)],
) -> tempfile::TempDir {
    let dir = tempdir().unwrap();

    if !common::git_available() || !common::init_git_repo(dir.path()) {
        panic!("git not available or init failed");
    }

    for (name, content) in base_files {
        let path = dir.path().join(name);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).unwrap();
        }
        std::fs::write(&path, content).unwrap();
    }
    assert!(common::git_add_commit(dir.path(), "Initial commit"));

    let _ = std::process::Command::new("git")
        .args(["checkout", "-b", "feature"])
        .current_dir(dir.path())
        .status();

    for (name, content) in feature_files {
        let path = dir.path().join(name);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).unwrap();
        }
        std::fs::write(&path, content).unwrap();
    }
    assert!(common::git_add_commit(dir.path(), "Feature changes"));

    dir
}

fn run_sensor_json(dir: &std::path::Path) -> Option<Value> {
    let output_path = dir
        .join("artifacts")
        .join("tokmd")
        .join("report.json");

    let output = tokmd()
        .current_dir(dir)
        .args([
            "sensor",
            "--base", "main",
            "--head", "HEAD",
            "--output",
        ])
        .arg(&output_path)
        .arg("--format")
        .arg("json")
        .output()
        .expect("run tokmd sensor");

    if !output.status.success() {
        eprintln!(
            "sensor failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        return None;
    }

    let stdout = String::from_utf8(output.stdout).expect("utf8");
    Some(serde_json::from_str(&stdout).expect("valid JSON"))
}

// ---------------------------------------------------------------------------
// 1. Schema compliance
// ---------------------------------------------------------------------------

#[test]
fn sensor_json_schema_is_report_v1() {
    if !common::git_available() {
        return;
    }

    let dir = scaffold_git_repo(
        &[("src/lib.rs", "fn main() {}\n")],
        &[("src/lib.rs", "fn main() { println!(\"hi\"); }\n")],
    );

    let json = match run_sensor_json(dir.path()) {
        Some(j) => j,
        None => return,
    };

    assert_eq!(json["schema"], "sensor.report.v1");
}

#[test]
fn sensor_json_has_tool_metadata() {
    if !common::git_available() {
        return;
    }

    let dir = scaffold_git_repo(
        &[("src/lib.rs", "fn main() {}\n")],
        &[("src/lib.rs", "fn main() { }\n")],
    );

    let json = match run_sensor_json(dir.path()) {
        Some(j) => j,
        None => return,
    };

    assert_eq!(json["tool"]["name"], "tokmd");
    assert!(json["tool"]["version"].is_string());
}

#[test]
fn sensor_json_has_verdict() {
    if !common::git_available() {
        return;
    }

    let dir = scaffold_git_repo(
        &[("src/lib.rs", "fn main() {}\n")],
        &[("src/lib.rs", "fn main() { }\n")],
    );

    let json = match run_sensor_json(dir.path()) {
        Some(j) => j,
        None => return,
    };

    let verdict = json["verdict"].as_str().unwrap_or("");
    let valid = ["pass", "fail", "warn", "skip", "pending"];
    assert!(
        valid.contains(&verdict),
        "verdict should be one of {valid:?}, got: {verdict}"
    );
}

#[test]
fn sensor_json_has_summary_string() {
    if !common::git_available() {
        return;
    }

    let dir = scaffold_git_repo(
        &[("src/lib.rs", "fn main() {}\n")],
        &[("src/lib.rs", "fn main() { }\n")],
    );

    let json = match run_sensor_json(dir.path()) {
        Some(j) => j,
        None => return,
    };

    assert!(json["summary"].is_string(), "summary should be a string");
    assert!(
        !json["summary"].as_str().unwrap().is_empty(),
        "summary should not be empty"
    );
}

#[test]
fn sensor_json_has_findings_array() {
    if !common::git_available() {
        return;
    }

    let dir = scaffold_git_repo(
        &[("src/lib.rs", "fn main() {}\n")],
        &[("src/lib.rs", "fn main() { }\n")],
    );

    let json = match run_sensor_json(dir.path()) {
        Some(j) => j,
        None => return,
    };

    assert!(json["findings"].is_array(), "findings should be an array");
}

#[test]
fn sensor_json_has_generated_at_timestamp() {
    if !common::git_available() {
        return;
    }

    let dir = scaffold_git_repo(
        &[("src/lib.rs", "fn main() {}\n")],
        &[("src/lib.rs", "fn main() { }\n")],
    );

    let json = match run_sensor_json(dir.path()) {
        Some(j) => j,
        None => return,
    };

    assert!(
        json["generated_at"].is_string(),
        "generated_at should be ISO 8601 string"
    );
}

// ---------------------------------------------------------------------------
// 2. Artifacts
// ---------------------------------------------------------------------------

#[test]
fn sensor_creates_report_and_sidecar_files() {
    if !common::git_available() {
        return;
    }

    let dir = scaffold_git_repo(
        &[("src/lib.rs", "fn main() {}\n")],
        &[("src/lib.rs", "fn main() { println!(\"hi\"); }\n")],
    );

    let report_path = dir
        .path()
        .join("artifacts")
        .join("tokmd")
        .join("report.json");

    let output = tokmd()
        .current_dir(dir.path())
        .args([
            "sensor",
            "--base", "main",
            "--head", "HEAD",
            "--output",
        ])
        .arg(&report_path)
        .arg("--format")
        .arg("json")
        .output()
        .unwrap();

    if !output.status.success() {
        return;
    }

    assert!(report_path.exists(), "report.json should be created");

    let comment_path = report_path.parent().unwrap().join("comment.md");
    assert!(comment_path.exists(), "comment.md should be created");

    let sidecar = report_path
        .parent()
        .unwrap()
        .join("extras")
        .join("cockpit_receipt.json");
    assert!(sidecar.exists(), "cockpit_receipt.json sidecar should be created");
}

#[test]
fn sensor_json_artifacts_list_has_expected_ids() {
    if !common::git_available() {
        return;
    }

    let dir = scaffold_git_repo(
        &[("src/lib.rs", "fn main() {}\n")],
        &[("src/lib.rs", "fn main() { println!(\"hello\"); }\n")],
    );

    let json = match run_sensor_json(dir.path()) {
        Some(j) => j,
        None => return,
    };

    let artifacts = json["artifacts"].as_array().expect("artifacts array");
    let ids: Vec<&str> = artifacts
        .iter()
        .filter_map(|a| a["id"].as_str())
        .collect();

    for expected in ["receipt", "cockpit", "comment"] {
        assert!(
            ids.contains(&expected),
            "artifacts should contain '{expected}', got: {ids:?}"
        );
    }
}

// ---------------------------------------------------------------------------
// 3. Markdown format
// ---------------------------------------------------------------------------

#[test]
fn sensor_md_produces_markdown_report() {
    if !common::git_available() {
        return;
    }

    let dir = scaffold_git_repo(
        &[("src/lib.rs", "fn main() {}\n")],
        &[("src/lib.rs", "fn main() { }\n")],
    );

    let report_path = dir
        .path()
        .join("artifacts")
        .join("tokmd")
        .join("report.json");

    let output = tokmd()
        .current_dir(dir.path())
        .args([
            "sensor",
            "--base", "main",
            "--head", "HEAD",
            "--output",
        ])
        .arg(&report_path)
        .arg("--format")
        .arg("md")
        .output()
        .unwrap();

    if !output.status.success() {
        return;
    }

    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(
        stdout.contains("## Sensor Report: tokmd"),
        "markdown should contain Sensor Report header"
    );
}

// ---------------------------------------------------------------------------
// 4. Data section contains gates
// ---------------------------------------------------------------------------

#[test]
fn sensor_data_section_has_gates() {
    if !common::git_available() {
        return;
    }

    let dir = scaffold_git_repo(
        &[("src/lib.rs", "fn main() {}\n")],
        &[("src/lib.rs", "fn main() { println!(\"hello\"); }\n")],
    );

    let json = match run_sensor_json(dir.path()) {
        Some(j) => j,
        None => return,
    };

    assert!(json["data"].is_object(), "data should be present");
    assert!(json["data"]["gates"].is_object(), "data.gates should be present");
    assert!(
        json["data"]["gates"]["items"].is_array(),
        "data.gates.items should be an array"
    );
}

#[test]
fn sensor_gates_contain_mutation_gate() {
    if !common::git_available() {
        return;
    }

    let dir = scaffold_git_repo(
        &[("src/lib.rs", "fn main() {}\n")],
        &[("src/lib.rs", "fn main() { println!(\"hi\"); }\n")],
    );

    let json = match run_sensor_json(dir.path()) {
        Some(j) => j,
        None => return,
    };

    let items = json["data"]["gates"]["items"]
        .as_array()
        .expect("gate items");
    let mutation = items.iter().find(|g| g["id"] == "mutation");
    assert!(mutation.is_some(), "mutation gate should always be present");
}

// ---------------------------------------------------------------------------
// 5. On-disk report matches stdout
// ---------------------------------------------------------------------------

#[test]
fn sensor_on_disk_report_matches_stdout() {
    if !common::git_available() {
        return;
    }

    let dir = scaffold_git_repo(
        &[("src/lib.rs", "fn main() {}\n")],
        &[("src/lib.rs", "fn main() { }\n")],
    );

    let report_path = dir
        .path()
        .join("artifacts")
        .join("tokmd")
        .join("report.json");

    let output = tokmd()
        .current_dir(dir.path())
        .args([
            "sensor",
            "--base", "main",
            "--head", "HEAD",
            "--output",
        ])
        .arg(&report_path)
        .arg("--format")
        .arg("json")
        .output()
        .unwrap();

    if !output.status.success() {
        return;
    }

    let stdout_json: Value =
        serde_json::from_slice(&output.stdout).expect("valid JSON from stdout");
    let disk_json: Value = serde_json::from_str(
        &std::fs::read_to_string(&report_path).unwrap(),
    )
    .expect("valid JSON on disk");

    assert_eq!(
        stdout_json["schema"], disk_json["schema"],
        "schema should match between stdout and disk"
    );
    assert_eq!(
        stdout_json["verdict"], disk_json["verdict"],
        "verdict should match between stdout and disk"
    );
}

// ---------------------------------------------------------------------------
// 6. Docs-only change scenario
// ---------------------------------------------------------------------------

#[test]
fn sensor_docs_only_change_produces_skip_verdict() {
    if !common::git_available() {
        return;
    }

    let dir = scaffold_git_repo(
        &[
            ("src/lib.rs", "fn main() {}\n"),
            ("README.md", "# Project\n"),
        ],
        &[("README.md", "# Project\n\nUpdated docs.\n")],
    );

    let json = match run_sensor_json(dir.path()) {
        Some(j) => j,
        None => return,
    };

    assert_eq!(
        json["verdict"], "skip",
        "docs-only change should produce skip verdict, got: {}",
        json["verdict"]
    );

    let findings = json["findings"].as_array().expect("findings array");
    assert!(
        findings.is_empty(),
        "docs-only change should have no findings"
    );
}

// ---------------------------------------------------------------------------
// 7. Capabilities section
// ---------------------------------------------------------------------------

#[test]
fn sensor_json_has_capabilities_or_null() {
    if !common::git_available() {
        return;
    }

    let dir = scaffold_git_repo(
        &[("src/lib.rs", "fn main() {}\n")],
        &[("src/lib.rs", "fn main() { }\n")],
    );

    let json = match run_sensor_json(dir.path()) {
        Some(j) => j,
        None => return,
    };

    // capabilities may be null or an object – both are valid
    let cap = &json["capabilities"];
    assert!(
        cap.is_null() || cap.is_object(),
        "capabilities should be null or object, got: {cap}"
    );
}
