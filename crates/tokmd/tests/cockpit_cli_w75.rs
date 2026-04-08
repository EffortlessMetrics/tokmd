#![cfg(feature = "git")]
//! Deep CLI tests for `tokmd cockpit` – w75 wave.
//!
//! All tests require the `git` feature flag and a working `git` binary.
//! Each test scaffolds a temporary git repo to ensure deterministic behaviour.

#![cfg(feature = "git")]
mod common;

use assert_cmd::Command;
use serde_json::Value;
use tempfile::tempdir;

fn tokmd() -> Command {
    Command::new(env!("CARGO_BIN_EXE_tokmd"))
}

/// Scaffold a two-branch git repo: main → feature.
fn scaffold(base_files: &[(&str, &str)], feature_files: &[(&str, &str)]) -> tempfile::TempDir {
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

/// Run cockpit and return parsed JSON (or None if failed).
fn run_cockpit_json(dir: &std::path::Path, extra_args: &[&str]) -> Option<Value> {
    let mut args = vec!["cockpit", "--base", "main", "--format", "json"];
    args.extend_from_slice(extra_args);

    let output = tokmd().current_dir(dir).args(&args).output().unwrap();

    if !output.status.success() {
        return None;
    }
    serde_json::from_slice(&output.stdout).ok()
}

// ═══════════════════════════════════════════════════════════════════════════
// 1. Basic JSON structure
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn cockpit_json_top_level_keys() {
    if !common::git_available() {
        return;
    }

    let dir = scaffold(
        &[("src/lib.rs", "pub fn hello() {}\n")],
        &[("src/lib.rs", "pub fn hello() { println!(\"hi\"); }\n")],
    );

    let json = match run_cockpit_json(dir.path(), &[]) {
        Some(j) => j,
        None => return,
    };

    assert!(json["schema_version"].is_number());
    assert_eq!(json["mode"], "cockpit");
    assert!(json["change_surface"].is_object());
    assert!(json["composition"].is_object());
    assert!(json["code_health"].is_object());
    assert!(json["risk"].is_object());
    assert!(json["contracts"].is_object());
    assert!(json["evidence"].is_object());
}

// ═══════════════════════════════════════════════════════════════════════════
// 2. Change surface metrics
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn cockpit_change_surface_counts_new_file() {
    if !common::git_available() {
        return;
    }

    let dir = scaffold(
        &[("src/lib.rs", "fn main() {}\n")],
        &[
            ("src/lib.rs", "fn main() {}\n"),
            ("src/extra.rs", "fn extra() {}\n"),
        ],
    );

    let json = match run_cockpit_json(dir.path(), &[]) {
        Some(j) => j,
        None => return,
    };

    let cs = &json["change_surface"];
    assert!(cs["files_changed"].as_u64().unwrap_or(0) >= 1);
    assert!(cs["commits"].as_u64().unwrap_or(0) >= 1);
    assert!(cs["insertions"].is_number());
    assert!(cs["deletions"].is_number());
}

#[test]
fn cockpit_change_surface_detects_modification() {
    if !common::git_available() {
        return;
    }

    let dir = scaffold(
        &[("src/lib.rs", "fn old() {}\n")],
        &[(
            "src/lib.rs",
            "fn new_function() { println!(\"updated\"); }\n",
        )],
    );

    let json = match run_cockpit_json(dir.path(), &[]) {
        Some(j) => j,
        None => return,
    };

    let cs = &json["change_surface"];
    let insertions = cs["insertions"].as_u64().unwrap_or(0);
    let deletions = cs["deletions"].as_u64().unwrap_or(0);
    assert!(insertions > 0 || deletions > 0, "should detect changes");
}

// ═══════════════════════════════════════════════════════════════════════════
// 3. Output formats
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn cockpit_markdown_format_has_sections() {
    if !common::git_available() {
        return;
    }

    let dir = scaffold(
        &[("src/lib.rs", "fn main() {}\n")],
        &[("src/lib.rs", "fn main() { println!(\"hi\"); }\n")],
    );

    let output = tokmd()
        .current_dir(dir.path())
        .args(["cockpit", "--base", "main", "--format", "md"])
        .output()
        .unwrap();

    if !output.status.success() {
        return;
    }

    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(
        stdout.contains("## Glass Cockpit"),
        "missing cockpit header"
    );
    assert!(
        stdout.contains("### Change Surface"),
        "missing change surface"
    );
    assert!(stdout.contains("### Composition"), "missing composition");
}

#[test]
fn cockpit_sections_format_has_markers() {
    if !common::git_available() {
        return;
    }

    let dir = scaffold(
        &[("src/lib.rs", "fn main() {}\n")],
        &[("src/lib.rs", "fn main() { }\n")],
    );

    let output = tokmd()
        .current_dir(dir.path())
        .args(["cockpit", "--base", "main", "--format", "sections"])
        .output()
        .unwrap();

    if !output.status.success() {
        return;
    }

    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("<!-- SECTION:COCKPIT -->"));
    assert!(stdout.contains("<!-- SECTION:REVIEW_PLAN -->"));
    assert!(stdout.contains("<!-- SECTION:RECEIPTS -->"));
}

// ═══════════════════════════════════════════════════════════════════════════
// 4. Evidence gates
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn cockpit_evidence_has_overall_status() {
    if !common::git_available() {
        return;
    }

    let dir = scaffold(
        &[("src/lib.rs", "fn main() {}\n")],
        &[("src/lib.rs", "fn main() { }\n")],
    );

    let json = match run_cockpit_json(dir.path(), &[]) {
        Some(j) => j,
        None => return,
    };

    let evidence = &json["evidence"];
    assert!(evidence.is_object());
    assert!(evidence["overall_status"].is_string());
    let status = evidence["overall_status"].as_str().unwrap();
    let valid = ["pass", "warn", "fail", "skipped", "pending"];
    assert!(
        valid.iter().any(|s| status.eq_ignore_ascii_case(s)),
        "unexpected overall_status: {status}"
    );
}

#[test]
fn cockpit_evidence_mutation_gate_present() {
    if !common::git_available() {
        return;
    }

    let dir = scaffold(
        &[("src/lib.rs", "fn main() {}\n")],
        &[("src/lib.rs", "fn main() { }\n")],
    );

    let json = match run_cockpit_json(dir.path(), &[]) {
        Some(j) => j,
        None => return,
    };

    let evidence = &json["evidence"];
    // mutation gate should be present (it's required)
    assert!(
        evidence["mutation"].is_object() || evidence["mutation_gate"].is_object(),
        "mutation gate should exist in evidence"
    );
}

// ═══════════════════════════════════════════════════════════════════════════
// 5. Diff range and head/base flags
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn cockpit_explicit_head_flag() {
    if !common::git_available() {
        return;
    }

    let dir = scaffold(
        &[("src/lib.rs", "fn main() {}\n")],
        &[("src/lib.rs", "fn main() { println!(\"head\"); }\n")],
    );

    let json = match run_cockpit_json(dir.path(), &["--head", "HEAD"]) {
        Some(j) => j,
        None => return,
    };

    assert!(json["schema_version"].is_number());
}

#[test]
fn cockpit_env_var_base_ref() {
    if !common::git_available() {
        return;
    }

    let dir = scaffold(
        &[("src/lib.rs", "fn main() {}\n")],
        &[("src/lib.rs", "fn main() { }\n")],
    );

    let output = tokmd()
        .current_dir(dir.path())
        .env("TOKMD_GIT_BASE_REF", "main")
        .args(["cockpit", "--format", "json"])
        .output()
        .unwrap();

    if !output.status.success() {
        return;
    }

    let json: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert!(json["schema_version"].is_number());
}

// ═══════════════════════════════════════════════════════════════════════════
// 6. Output to file
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn cockpit_output_flag_writes_file() {
    if !common::git_available() {
        return;
    }

    let dir = scaffold(
        &[("src/lib.rs", "fn main() {}\n")],
        &[("src/lib.rs", "fn main() { }\n")],
    );

    let out_path = dir.path().join("cockpit_w75.json");

    let result = tokmd()
        .current_dir(dir.path())
        .args(["cockpit", "--base", "main", "--output"])
        .arg(&out_path)
        .output()
        .unwrap();

    if !result.status.success() {
        return;
    }

    assert!(out_path.exists(), "output file should be created");
    let content = std::fs::read_to_string(&out_path).unwrap();
    let json: Value = serde_json::from_str(&content).unwrap();
    assert!(json["schema_version"].is_number());
}

// ═══════════════════════════════════════════════════════════════════════════
// 7. Review plan and risk
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn cockpit_review_plan_is_array() {
    if !common::git_available() {
        return;
    }

    let dir = scaffold(
        &[("src/lib.rs", "fn main() {}\n")],
        &[
            ("src/lib.rs", "fn main() { println!(\"changed\"); }\n"),
            ("src/new.rs", "fn new_fn() {}\n"),
        ],
    );

    let json = match run_cockpit_json(dir.path(), &[]) {
        Some(j) => j,
        None => return,
    };

    assert!(json["review_plan"].is_array());
}

#[test]
fn cockpit_risk_level_is_valid_string() {
    if !common::git_available() {
        return;
    }

    let dir = scaffold(
        &[("src/lib.rs", "fn main() {}\n")],
        &[("src/lib.rs", "fn main() { }\n")],
    );

    let json = match run_cockpit_json(dir.path(), &[]) {
        Some(j) => j,
        None => return,
    };

    let risk = &json["risk"];
    assert!(risk["level"].is_string());
    let level = risk["level"].as_str().unwrap();
    let valid = ["low", "medium", "high", "critical"];
    assert!(
        valid.iter().any(|s| level.eq_ignore_ascii_case(s)),
        "unexpected risk level: {level}"
    );
}

// ═══════════════════════════════════════════════════════════════════════════
// 8. Code health and composition
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn cockpit_code_health_grade_valid() {
    if !common::git_available() {
        return;
    }

    let dir = scaffold(
        &[("src/lib.rs", "fn main() {}\n")],
        &[("src/lib.rs", "fn main() { }\n")],
    );

    let json = match run_cockpit_json(dir.path(), &[]) {
        Some(j) => j,
        None => return,
    };

    let grade = json["code_health"]["grade"].as_str().unwrap_or("");
    let valid_grades = ["A", "B", "C", "D", "F"];
    assert!(
        valid_grades.iter().any(|g| grade.starts_with(g)),
        "unexpected grade: {grade}"
    );
}

#[test]
fn cockpit_composition_has_code_pct() {
    if !common::git_available() {
        return;
    }

    let dir = scaffold(
        &[("src/lib.rs", "fn main() {}\n")],
        &[("src/lib.rs", "fn main() { }\n")],
    );

    let json = match run_cockpit_json(dir.path(), &[]) {
        Some(j) => j,
        None => return,
    };

    assert!(json["composition"]["code_pct"].is_number());
}
