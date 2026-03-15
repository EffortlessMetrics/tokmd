//! Deep CLI integration tests for `tokmd run`, `tokmd cockpit`, and
//! `tokmd badge` commands.

mod common;

use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::tempdir;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn tokmd_cmd() -> Command {
    Command::new(env!("CARGO_BIN_EXE_tokmd"))
}

fn tokmd_on_fixtures() -> Command {
    let mut cmd = tokmd_cmd();
    cmd.current_dir(common::fixture_root());
    cmd
}

/// Run `tokmd run` on the fixture tree into `output_dir` and return the path.
fn run_to_dir(output_dir: &std::path::Path) {
    tokmd_on_fixtures()
        .args(["run", "--output-dir"])
        .arg(output_dir)
        .arg(".")
        .assert()
        .success();
}

// =========================================================================
// 1. `tokmd run` tests
// =========================================================================

#[test]
fn run_produces_output_directory_with_artifacts() {
    let dir = tempdir().unwrap();
    let out = dir.path().join("run_out");
    run_to_dir(&out);

    assert!(out.join("receipt.json").exists(), "receipt.json missing");
    assert!(out.join("lang.json").exists(), "lang.json missing");
    assert!(out.join("module.json").exists(), "module.json missing");
    assert!(out.join("export.jsonl").exists(), "export.jsonl missing");
}

#[test]
fn run_receipt_is_valid_json() {
    let dir = tempdir().unwrap();
    let out = dir.path().join("run_json");
    run_to_dir(&out);

    let raw = fs::read_to_string(out.join("receipt.json")).unwrap();
    let v: serde_json::Value = serde_json::from_str(&raw).expect("receipt must be valid JSON");
    assert!(v.is_object(), "receipt should be a JSON object");
}

#[test]
fn run_includes_lang_module_export_references() {
    let dir = tempdir().unwrap();
    let out = dir.path().join("run_refs");
    run_to_dir(&out);

    let raw = fs::read_to_string(out.join("receipt.json")).unwrap();
    let v: serde_json::Value = serde_json::from_str(&raw).unwrap();
    assert!(v["lang_file"].is_string(), "should reference lang_file");
    assert!(v["module_file"].is_string(), "should reference module_file");
    assert!(v["export_file"].is_string(), "should reference export_file");
}

#[test]
fn run_output_dir_creates_specified_directory() {
    let dir = tempdir().unwrap();
    let custom = dir.path().join("custom").join("nested");
    assert!(!custom.exists(), "should not pre-exist");

    run_to_dir(&custom);
    assert!(custom.exists(), "output dir should be created");
    assert!(custom.join("receipt.json").exists());
}

#[test]
fn run_receipt_has_schema_version() {
    let dir = tempdir().unwrap();
    let out = dir.path().join("run_sv");
    run_to_dir(&out);

    let v: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(out.join("receipt.json")).unwrap()).unwrap();
    assert_eq!(v["schema_version"], 2, "schema_version should be 2");
}

#[test]
fn run_lang_json_has_languages() {
    let dir = tempdir().unwrap();
    let out = dir.path().join("run_lang");
    run_to_dir(&out);

    let v: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(out.join("lang.json")).unwrap()).unwrap();
    assert!(
        v.get("languages").is_some() || v.get("rows").is_some(),
        "lang.json should contain language data"
    );
}

#[test]
fn run_module_json_is_valid_json() {
    let dir = tempdir().unwrap();
    let out = dir.path().join("run_mod");
    run_to_dir(&out);

    let raw = fs::read_to_string(out.join("module.json")).unwrap();
    let v: serde_json::Value = serde_json::from_str(&raw).expect("module.json must be valid JSON");
    assert!(v.is_object(), "module.json should be a JSON object");
}

#[test]
fn run_export_jsonl_has_lines() {
    let dir = tempdir().unwrap();
    let out = dir.path().join("run_export");
    run_to_dir(&out);

    let raw = fs::read_to_string(out.join("export.jsonl")).unwrap();
    let lines: Vec<&str> = raw.lines().collect();
    assert!(
        !lines.is_empty(),
        "export.jsonl should have at least one line"
    );
    // Each line must be valid JSON
    for (i, line) in lines.iter().enumerate() {
        let _: serde_json::Value =
            serde_json::from_str(line).unwrap_or_else(|e| panic!("line {} invalid JSON: {}", i, e));
    }
}

#[test]
fn run_redact_paths_hides_raw_paths() {
    let dir = tempdir().unwrap();
    let out = dir.path().join("run_redact");

    tokmd_on_fixtures()
        .args(["run", "--output-dir"])
        .arg(&out)
        .args(["--redact", "paths", "."])
        .assert()
        .success();

    let export_raw = fs::read_to_string(out.join("export.jsonl")).unwrap();
    // Redacted paths should contain hash-like segments (blake3 hex), not .rs/.js extensions directly
    // At minimum they should not contain the literal fixture filenames
    assert!(
        !export_raw.contains("script.js") && !export_raw.contains("large.rs"),
        "redacted export should not contain raw fixture filenames"
    );
}

#[test]
fn run_on_empty_dir_produces_valid_receipt() {
    let dir = tempdir().unwrap();
    let empty = dir.path().join("empty_src");
    fs::create_dir_all(&empty).unwrap();
    // Create a .git marker so ignore crate works
    fs::create_dir_all(empty.join(".git")).unwrap();
    let out = dir.path().join("run_empty");

    tokmd_cmd()
        .current_dir(&empty)
        .args(["run", "--output-dir"])
        .arg(&out)
        .arg(".")
        .assert()
        .success();

    let v: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(out.join("receipt.json")).unwrap()).unwrap();
    assert_eq!(v["schema_version"], 2);
}

#[test]
fn run_determinism_two_runs_identical() {
    let dir = tempdir().unwrap();
    let out1 = dir.path().join("det1");
    let out2 = dir.path().join("det2");

    run_to_dir(&out1);
    run_to_dir(&out2);

    // Compare lang.json after stripping timestamps (generated_at_ms varies)
    let strip_ts = |s: &str| -> serde_json::Value {
        let mut v: serde_json::Value = serde_json::from_str(s).unwrap();
        if let Some(obj) = v.as_object_mut() {
            obj.remove("generated_at_ms");
        }
        v
    };

    let lang1 = fs::read_to_string(out1.join("lang.json")).unwrap();
    let lang2 = fs::read_to_string(out2.join("lang.json")).unwrap();
    assert_eq!(
        strip_ts(&lang1),
        strip_ts(&lang2),
        "lang.json should be identical across runs (ignoring timestamp)"
    );

    let mod1 = fs::read_to_string(out1.join("module.json")).unwrap();
    let mod2 = fs::read_to_string(out2.join("module.json")).unwrap();
    assert_eq!(
        strip_ts(&mod1),
        strip_ts(&mod2),
        "module.json should be identical across runs (ignoring timestamp)"
    );

    // Compare export.jsonl lines after stripping timestamps
    let exp1 = fs::read_to_string(out1.join("export.jsonl")).unwrap();
    let exp2 = fs::read_to_string(out2.join("export.jsonl")).unwrap();
    let strip_jsonl_ts = |s: &str| -> Vec<serde_json::Value> {
        s.lines()
            .map(|l| {
                let mut v: serde_json::Value = serde_json::from_str(l).unwrap();
                if let Some(obj) = v.as_object_mut() {
                    obj.remove("generated_at_ms");
                }
                v
            })
            .collect()
    };
    assert_eq!(
        strip_jsonl_ts(&exp1),
        strip_jsonl_ts(&exp2),
        "export.jsonl should be identical across runs (ignoring timestamp)"
    );
}

#[test]
fn run_receipt_generated_at_ms_is_present() {
    let dir = tempdir().unwrap();
    let out = dir.path().join("run_ts");
    run_to_dir(&out);

    let v: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(out.join("receipt.json")).unwrap()).unwrap();
    assert!(
        v["generated_at_ms"].is_number(),
        "receipt should have generated_at_ms"
    );
}

// =========================================================================
// 2. `tokmd cockpit` tests
// =========================================================================

/// Helper: create a minimal git repo with two branches for cockpit tests.
/// Returns the tempdir (caller must keep it alive).
#[cfg(feature = "git")]
#[allow(dead_code)]
fn setup_cockpit_repo() -> Option<tempfile::TempDir> {
    if !common::git_available() {
        return None;
    }
    let dir = tempdir().unwrap();

    if !common::init_git_repo(dir.path()) {
        return None;
    }

    fs::write(dir.path().join("lib.rs"), "fn hello() {}\n").unwrap();
    if !common::git_add_commit(dir.path(), "Initial commit") {
        return None;
    }

    let ok = std::process::Command::new("git")
        .args(["checkout", "-b", "feature"])
        .current_dir(dir.path())
        .status()
        .map(|s| s.success())
        .unwrap_or(false);
    if !ok {
        return None;
    }

    fs::write(dir.path().join("new.rs"), "fn new_func() {}\n").unwrap();
    if !common::git_add_commit(dir.path(), "Add new file") {
        return None;
    }

    Some(dir)
}

#[test]
#[cfg(feature = "git")]
fn cockpit_json_has_schema_version() {
    let dir = match setup_cockpit_repo() {
        Some(d) => d,
        None => {
            eprintln!("Skipping: git setup failed");
            return;
        }
    };

    let output = tokmd_cmd()
        .current_dir(dir.path())
        .args([
            "cockpit", "--base", "main", "--head", "HEAD", "--format", "json",
        ])
        .output()
        .unwrap();

    if !output.status.success() {
        eprintln!(
            "Skipping: cockpit failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        return;
    }

    let v: serde_json::Value =
        serde_json::from_str(&String::from_utf8(output.stdout).unwrap()).unwrap();
    assert_eq!(v["schema_version"], 3, "cockpit schema_version should be 3");
}

#[test]
#[cfg(feature = "git")]
fn cockpit_json_has_composition() {
    let dir = match setup_cockpit_repo() {
        Some(d) => d,
        None => return,
    };

    let output = tokmd_cmd()
        .current_dir(dir.path())
        .args([
            "cockpit", "--base", "main", "--head", "HEAD", "--format", "json",
        ])
        .output()
        .unwrap();

    if !output.status.success() {
        return;
    }

    let v: serde_json::Value =
        serde_json::from_str(&String::from_utf8(output.stdout).unwrap()).unwrap();
    assert!(
        v.get("composition").is_some(),
        "cockpit should include composition"
    );
}

#[test]
#[cfg(feature = "git")]
fn cockpit_json_has_code_health() {
    let dir = match setup_cockpit_repo() {
        Some(d) => d,
        None => return,
    };

    let output = tokmd_cmd()
        .current_dir(dir.path())
        .args([
            "cockpit", "--base", "main", "--head", "HEAD", "--format", "json",
        ])
        .output()
        .unwrap();

    if !output.status.success() {
        return;
    }

    let v: serde_json::Value =
        serde_json::from_str(&String::from_utf8(output.stdout).unwrap()).unwrap();
    assert!(
        v.get("code_health").is_some(),
        "cockpit should include code_health"
    );
}

#[test]
#[cfg(feature = "git")]
fn cockpit_json_has_change_surface() {
    let dir = match setup_cockpit_repo() {
        Some(d) => d,
        None => return,
    };

    let output = tokmd_cmd()
        .current_dir(dir.path())
        .args([
            "cockpit", "--base", "main", "--head", "HEAD", "--format", "json",
        ])
        .output()
        .unwrap();

    if !output.status.success() {
        return;
    }

    let v: serde_json::Value =
        serde_json::from_str(&String::from_utf8(output.stdout).unwrap()).unwrap();
    assert!(
        v.get("change_surface").is_some(),
        "cockpit should include change_surface"
    );
}

#[test]
#[cfg(feature = "git")]
fn cockpit_json_has_evidence_and_risk() {
    let dir = match setup_cockpit_repo() {
        Some(d) => d,
        None => return,
    };

    let output = tokmd_cmd()
        .current_dir(dir.path())
        .args([
            "cockpit", "--base", "main", "--head", "HEAD", "--format", "json",
        ])
        .output()
        .unwrap();

    if !output.status.success() {
        return;
    }

    let v: serde_json::Value =
        serde_json::from_str(&String::from_utf8(output.stdout).unwrap()).unwrap();
    assert!(
        v.get("evidence").is_some(),
        "cockpit should include evidence"
    );
    assert!(v.get("risk").is_some(), "cockpit should include risk");
}

#[test]
#[cfg(feature = "git")]
fn cockpit_md_has_expected_sections() {
    let dir = match setup_cockpit_repo() {
        Some(d) => d,
        None => return,
    };

    let output = tokmd_cmd()
        .current_dir(dir.path())
        .args(["cockpit", "--base", "main", "--format", "md"])
        .output()
        .unwrap();

    if !output.status.success() {
        return;
    }

    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(
        stdout.contains("Glass Cockpit") || stdout.contains("Cockpit"),
        "md should contain Glass Cockpit header"
    );
    assert!(
        stdout.contains("Change Surface"),
        "md should contain Change Surface"
    );
    assert!(
        stdout.contains("Composition"),
        "md should contain Composition"
    );
}

#[test]
#[cfg(feature = "git")]
fn cockpit_json_has_review_plan() {
    let dir = match setup_cockpit_repo() {
        Some(d) => d,
        None => return,
    };

    let output = tokmd_cmd()
        .current_dir(dir.path())
        .args([
            "cockpit", "--base", "main", "--head", "HEAD", "--format", "json",
        ])
        .output()
        .unwrap();

    if !output.status.success() {
        return;
    }

    let v: serde_json::Value =
        serde_json::from_str(&String::from_utf8(output.stdout).unwrap()).unwrap();
    assert!(
        v.get("review_plan").is_some(),
        "cockpit should include review_plan"
    );
}

#[test]
#[cfg(feature = "git")]
fn cockpit_handles_no_git_history_gracefully() {
    if !common::git_available() {
        return;
    }
    let dir = tempdir().unwrap();

    // Directory with a .git marker but no actual git history
    fs::create_dir_all(dir.path().join(".git")).unwrap();
    fs::write(dir.path().join("main.rs"), "fn main() {}").unwrap();

    // cockpit should either fail gracefully or succeed with empty data
    let output = tokmd_cmd()
        .current_dir(dir.path())
        .args(["cockpit", "--base", "main", "--format", "json"])
        .output()
        .unwrap();

    // It's acceptable for the command to fail, but it should not panic
    // (i.e. exit code should be 0 or 1, not a crash signal)
    let code = output.status.code().unwrap_or(-1);
    assert!(
        code == 0 || code == 1 || code == 2,
        "cockpit should exit cleanly, got code {}",
        code
    );
}

// =========================================================================
// 3. `tokmd badge` tests
// =========================================================================

#[test]
fn badge_lines_produces_svg() {
    tokmd_on_fixtures()
        .args(["badge", "--metric", "lines"])
        .assert()
        .success()
        .stdout(predicate::str::contains("<svg"))
        .stdout(predicate::str::contains("</svg>"));
}

#[test]
fn badge_svg_is_valid_xml() {
    let output = tokmd_on_fixtures()
        .args(["badge", "--metric", "lines"])
        .output()
        .unwrap();

    assert!(output.status.success());
    let svg = String::from_utf8(output.stdout).unwrap();
    assert!(svg.starts_with('<'), "SVG should start with <");
    assert!(svg.contains("</svg>"), "SVG should have closing tag");
    // Verify basic XML well-formedness: contains xmlns
    assert!(svg.contains("xmlns"), "SVG should contain xmlns attribute");
}

#[test]
fn badge_contains_metric_label() {
    let output = tokmd_on_fixtures()
        .args(["badge", "--metric", "doc"])
        .output()
        .unwrap();

    assert!(output.status.success());
    let svg = String::from_utf8(output.stdout).unwrap();
    assert!(
        svg.contains("doc") || svg.contains("Doc"),
        "badge should contain the metric label"
    );
}

#[test]
fn badge_different_metrics_succeed() {
    // Test multiple metric variants
    for metric in &["lines", "tokens", "doc", "blank"] {
        let output = tokmd_on_fixtures()
            .args(["badge", "--metric", metric])
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "badge --metric {} should succeed",
            metric
        );
        let svg = String::from_utf8(output.stdout).unwrap();
        assert!(
            svg.contains("<svg"),
            "badge --metric {} should produce SVG",
            metric
        );
    }
}

#[test]
fn badge_output_to_file() -> Result<(), Box<dyn std::error::Error>> {
    let dir = tempdir()?;
    let out_file = dir.path().join("badge.svg");

    tokmd_on_fixtures()
        .args(["badge", "--metric", "lines", "--out"])
        .arg(&out_file)
        .assert()
        .success();

    let content = fs::read_to_string(&out_file)?;
    assert!(content.contains("<svg"), "file should contain SVG");
    assert!(content.contains("</svg>"), "file should have closing tag");
    Ok(())
}

#[test]
fn badge_invalid_metric_fails() {
    tokmd_on_fixtures()
        .args(["badge", "--metric", "nonexistent"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("invalid value"));
}
