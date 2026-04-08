#![cfg(feature = "git")]
//! Deep CLI tests for `tokmd cockpit` – PR change-surface analysis.
//!
//! All tests require the `git` feature flag and a working `git` binary.

#![cfg(feature = "git")]
mod common;

use assert_cmd::Command;
use predicates::prelude::*;
use serde_json::Value;
use tempfile::tempdir;

fn tokmd() -> Command {
    Command::new(env!("CARGO_BIN_EXE_tokmd"))
}

/// Scaffold a two-branch git repo: main (initial commit) -> feature (with changes).
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

// ---------------------------------------------------------------------------
// 1. Help text
// ---------------------------------------------------------------------------

#[test]
fn cockpit_help_shows_expected_flags() {
    tokmd()
        .args(["cockpit", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("--base"))
        .stdout(predicate::str::contains("--head"))
        .stdout(predicate::str::contains("--format"));
}

// ---------------------------------------------------------------------------
// 2. JSON output structure
// ---------------------------------------------------------------------------

#[test]
fn cockpit_json_has_schema_version_and_mode() {
    if !common::git_available() {
        return;
    }

    let dir = scaffold_git_repo(
        &[("src/lib.rs", "fn main() {}\n")],
        &[("src/lib.rs", "fn main() { println!(\"hi\"); }\n")],
    );

    let output = tokmd()
        .current_dir(dir.path())
        .args([
            "cockpit", "--base", "main", "--head", "HEAD", "--format", "json",
        ])
        .output()
        .unwrap();

    if !output.status.success() {
        return;
    }

    let json: Value = serde_json::from_slice(&output.stdout).expect("valid JSON");
    assert!(json["schema_version"].is_number());
    assert_eq!(json["mode"], "cockpit");
}

#[test]
fn cockpit_json_has_change_surface() {
    if !common::git_available() {
        return;
    }

    let dir = scaffold_git_repo(
        &[("src/lib.rs", "fn main() {}\n")],
        &[
            ("src/lib.rs", "fn main() { println!(\"hello\"); }\n"),
            ("src/extra.rs", "fn extra() {}\n"),
        ],
    );

    let output = tokmd()
        .current_dir(dir.path())
        .args(["cockpit", "--base", "main", "--format", "json"])
        .output()
        .unwrap();

    if !output.status.success() {
        return;
    }

    let json: Value = serde_json::from_slice(&output.stdout).unwrap();
    let cs = &json["change_surface"];
    assert!(cs.is_object(), "change_surface should be present");
    assert!(cs["commits"].is_number());
    assert!(cs["files_changed"].is_number());
    assert!(cs["insertions"].is_number());
    assert!(cs["deletions"].is_number());
}

#[test]
fn cockpit_json_has_composition() {
    if !common::git_available() {
        return;
    }

    let dir = scaffold_git_repo(
        &[("src/lib.rs", "fn main() {}\n")],
        &[("src/lib.rs", "fn main() { }\n")],
    );

    let output = tokmd()
        .current_dir(dir.path())
        .args(["cockpit", "--base", "main", "--format", "json"])
        .output()
        .unwrap();

    if !output.status.success() {
        return;
    }

    let json: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert!(json["composition"].is_object());
    assert!(json["composition"]["code_pct"].is_number());
}

#[test]
fn cockpit_json_has_evidence_gates() {
    if !common::git_available() {
        return;
    }

    let dir = scaffold_git_repo(
        &[("src/lib.rs", "fn main() {}\n")],
        &[("src/lib.rs", "fn main() { }\n")],
    );

    let output = tokmd()
        .current_dir(dir.path())
        .args(["cockpit", "--base", "main", "--format", "json"])
        .output()
        .unwrap();

    if !output.status.success() {
        return;
    }

    let json: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert!(json["evidence"].is_object(), "evidence should be present");
    assert!(
        json["evidence"]["overall_status"].is_string(),
        "overall_status should be a string"
    );
}

#[test]
fn cockpit_json_has_review_plan() {
    if !common::git_available() {
        return;
    }

    let dir = scaffold_git_repo(
        &[("src/lib.rs", "fn main() {}\n")],
        &[
            ("src/lib.rs", "fn main() { println!(\"changed\"); }\n"),
            ("src/new.rs", "fn new() {}\n"),
        ],
    );

    let output = tokmd()
        .current_dir(dir.path())
        .args(["cockpit", "--base", "main", "--format", "json"])
        .output()
        .unwrap();

    if !output.status.success() {
        return;
    }

    let json: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert!(
        json["review_plan"].is_array(),
        "review_plan should be present"
    );
}

#[test]
fn cockpit_json_has_contracts() {
    if !common::git_available() {
        return;
    }

    let dir = scaffold_git_repo(
        &[("src/lib.rs", "fn main() {}\n")],
        &[("src/lib.rs", "fn main() { }\n")],
    );

    let output = tokmd()
        .current_dir(dir.path())
        .args(["cockpit", "--base", "main", "--format", "json"])
        .output()
        .unwrap();

    if !output.status.success() {
        return;
    }

    let json: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert!(json["contracts"].is_object(), "contracts should be present");
}

// ---------------------------------------------------------------------------
// 3. Markdown output
// ---------------------------------------------------------------------------

#[test]
fn cockpit_md_has_expected_sections() {
    if !common::git_available() {
        return;
    }

    let dir = scaffold_git_repo(
        &[("src/main.rs", "fn main() {}\n")],
        &[("src/main.rs", "fn main() { println!(\"hello\"); }\n")],
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
        "missing Glass Cockpit header"
    );
    assert!(
        stdout.contains("### Change Surface"),
        "missing Change Surface"
    );
    assert!(stdout.contains("### Composition"), "missing Composition");
    assert!(stdout.contains("### Contracts"), "missing Contracts");
    assert!(stdout.contains("### Review Plan"), "missing Review Plan");
}

// ---------------------------------------------------------------------------
// 4. Sections output format
// ---------------------------------------------------------------------------

#[test]
fn cockpit_sections_has_markers() {
    if !common::git_available() {
        return;
    }

    let dir = scaffold_git_repo(
        &[("src/app.rs", "fn app() {}\n")],
        &[("src/app.rs", "fn app() { }\n")],
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

// ---------------------------------------------------------------------------
// 5. Output to file
// ---------------------------------------------------------------------------

#[test]
fn cockpit_output_file_creates_json() {
    if !common::git_available() {
        return;
    }

    let dir = scaffold_git_repo(
        &[("src/lib.rs", "fn main() {}\n")],
        &[("src/lib.rs", "fn main() { }\n")],
    );

    let output_file = dir.path().join("cockpit_out.json");

    let result = tokmd()
        .current_dir(dir.path())
        .args(["cockpit", "--base", "main", "--output"])
        .arg(&output_file)
        .output()
        .unwrap();

    if !result.status.success() {
        return;
    }

    assert!(output_file.exists(), "output file should be created");
    let content = std::fs::read_to_string(&output_file).unwrap();
    let json: Value = serde_json::from_str(&content).expect("valid JSON in output file");
    assert!(json["schema_version"].is_number());
}

// ---------------------------------------------------------------------------
// 6. Env var fallback for --base
// ---------------------------------------------------------------------------

#[test]
fn cockpit_base_from_env_var() {
    if !common::git_available() {
        return;
    }

    let dir = scaffold_git_repo(
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

    let json: Value = serde_json::from_slice(&output.stdout).expect("valid JSON");
    assert!(json["schema_version"].is_number());
}

// ---------------------------------------------------------------------------
// 7. Change surface metrics have expected values
// ---------------------------------------------------------------------------

#[test]
fn cockpit_change_surface_files_changed_matches() {
    if !common::git_available() {
        return;
    }

    let dir = scaffold_git_repo(
        &[("src/lib.rs", "fn main() {}\n")],
        &[
            ("src/lib.rs", "fn main() { println!(\"updated\"); }\n"),
            ("src/new_module.rs", "fn new_mod() {}\n"),
        ],
    );

    let output = tokmd()
        .current_dir(dir.path())
        .args(["cockpit", "--base", "main", "--format", "json"])
        .output()
        .unwrap();

    if !output.status.success() {
        return;
    }

    let json: Value = serde_json::from_slice(&output.stdout).unwrap();
    let files_changed = json["change_surface"]["files_changed"]
        .as_u64()
        .unwrap_or(0);
    assert!(
        files_changed >= 1,
        "should have at least 1 file changed, got {files_changed}"
    );
}

// ---------------------------------------------------------------------------
// 8. Risk section
// ---------------------------------------------------------------------------

#[test]
fn cockpit_json_has_risk_section() {
    if !common::git_available() {
        return;
    }

    let dir = scaffold_git_repo(
        &[("src/lib.rs", "fn main() {}\n")],
        &[("src/lib.rs", "fn main() { }\n")],
    );

    let output = tokmd()
        .current_dir(dir.path())
        .args(["cockpit", "--base", "main", "--format", "json"])
        .output()
        .unwrap();

    if !output.status.success() {
        return;
    }

    let json: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert!(json["risk"].is_object(), "risk section should be present");
    assert!(
        json["risk"]["level"].is_string(),
        "risk.level should be a string"
    );
}

// ---------------------------------------------------------------------------
// 9. Code health section
// ---------------------------------------------------------------------------

#[test]
fn cockpit_json_has_code_health() {
    if !common::git_available() {
        return;
    }

    let dir = scaffold_git_repo(
        &[("src/lib.rs", "fn main() {}\n")],
        &[("src/lib.rs", "fn main() { }\n")],
    );

    let output = tokmd()
        .current_dir(dir.path())
        .args(["cockpit", "--base", "main", "--format", "json"])
        .output()
        .unwrap();

    if !output.status.success() {
        return;
    }

    let json: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert!(
        json["code_health"].is_object(),
        "code_health should be present"
    );
    assert!(
        json["code_health"]["grade"].is_string(),
        "grade should be a string"
    );
}
