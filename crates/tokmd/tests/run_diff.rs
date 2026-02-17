mod common;

use assert_cmd::Command;
use assert_cmd::cargo::cargo_bin_cmd;
use predicates::prelude::*;
use std::fs;
use std::process::Command as ProcessCommand;
use tempfile::tempdir;

#[test]
fn test_run_generates_artifacts() {
    let dir = tempdir().unwrap();
    let output_dir = dir.path().join("run1");

    let mut cmd: Command = cargo_bin_cmd!("tokmd");
    cmd.current_dir(common::fixture_root()) // Run on test data so it's small and predictable
        .arg("run")
        .arg("--output-dir")
        .arg(output_dir.to_str().unwrap())
        .arg(".") // path to scan
        .assert()
        .success();

    assert!(
        output_dir.join("receipt.json").exists(),
        "receipt.json missing"
    );
    assert!(output_dir.join("lang.json").exists(), "lang.json missing");
    assert!(
        output_dir.join("module.json").exists(),
        "module.json missing"
    );
    assert!(
        output_dir.join("export.jsonl").exists(),
        "export.jsonl missing"
    );

    // Check content of receipt.json
    let receipt_content = fs::read_to_string(output_dir.join("receipt.json")).unwrap();
    assert!(receipt_content.contains("lang.json"));
    assert!(receipt_content.contains("schema_version"));
}

#[test]
fn test_diff_identical_runs() {
    let dir = tempdir().unwrap();
    let run1_dir = dir.path().join("run1");
    let run2_dir = dir.path().join("run2");

    // Run 1
    let mut cmd1: Command = cargo_bin_cmd!("tokmd");
    cmd1.current_dir(common::fixture_root())
        .arg("run")
        .arg("--output-dir")
        .arg(run1_dir.to_str().unwrap())
        .arg(".")
        .assert()
        .success();

    // Run 2 (same data)
    let mut cmd2: Command = cargo_bin_cmd!("tokmd");
    cmd2.current_dir(common::fixture_root())
        .arg("run")
        .arg("--output-dir")
        .arg(run2_dir.to_str().unwrap())
        .arg(".")
        .assert()
        .success();

    // Diff
    let mut cmd_diff: Command = cargo_bin_cmd!("tokmd");
    cmd_diff
        .arg("diff")
        .arg("--from")
        .arg(run1_dir.join("receipt.json").to_str().unwrap())
        .arg("--to")
        .arg(run2_dir.join("receipt.json").to_str().unwrap())
        .assert()
        .success()
        .stdout(predicate::str::contains("## Diff:"));
    // Should produce empty diff table (header only) because counts are identical
    // But headers are always printed.
}

#[test]
fn test_diff_compact_mode() {
    let dir = tempdir().unwrap();
    let run1_dir = dir.path().join("run1");
    let run2_dir = dir.path().join("run2");

    let mut cmd1: Command = cargo_bin_cmd!("tokmd");
    cmd1.current_dir(common::fixture_root())
        .arg("run")
        .arg("--output-dir")
        .arg(run1_dir.to_str().unwrap())
        .arg(".")
        .assert()
        .success();

    let mut cmd2: Command = cargo_bin_cmd!("tokmd");
    cmd2.current_dir(common::fixture_root())
        .arg("run")
        .arg("--output-dir")
        .arg(run2_dir.to_str().unwrap())
        .arg(".")
        .assert()
        .success();

    let mut cmd_diff: Command = cargo_bin_cmd!("tokmd");
    cmd_diff
        .arg("diff")
        .arg("--compact")
        .arg("--from")
        .arg(run1_dir.join("receipt.json").to_str().unwrap())
        .arg("--to")
        .arg(run2_dir.join("receipt.json").to_str().unwrap())
        .assert()
        .success()
        .stdout(predicate::str::contains("|Metric|Value|"))
        .stdout(predicate::str::contains("Languages changed"))
        .stdout(predicate::str::contains("Language Breakdown").not());
}

#[test]
fn test_diff_color_always_emits_ansi() {
    let dir = tempdir().unwrap();
    let run1_dir = dir.path().join("run1");
    let run2_dir = dir.path().join("run2");

    let mut cmd1: Command = cargo_bin_cmd!("tokmd");
    cmd1.current_dir(common::fixture_root())
        .arg("run")
        .arg("--output-dir")
        .arg(run1_dir.to_str().unwrap())
        .arg(".")
        .assert()
        .success();

    let mut cmd2: Command = cargo_bin_cmd!("tokmd");
    cmd2.current_dir(common::fixture_root())
        .arg("run")
        .arg("--output-dir")
        .arg(run2_dir.to_str().unwrap())
        .arg(".")
        .assert()
        .success();

    let mut cmd_diff: Command = cargo_bin_cmd!("tokmd");
    cmd_diff
        .arg("diff")
        .arg("--color")
        .arg("always")
        .arg("--from")
        .arg(run1_dir.join("receipt.json").to_str().unwrap())
        .arg("--to")
        .arg(run2_dir.join("receipt.json").to_str().unwrap())
        .assert()
        .success()
        .stdout(predicate::str::contains("\u{1b}["));
}

#[test]
fn test_run_default_output_creates_local_runs_dir() {
    let dir = tempdir().unwrap();
    let work_dir = dir.path().join("workdir");
    fs::create_dir_all(&work_dir).unwrap();

    // Create a minimal source file to scan
    let src_dir = work_dir.join("src");
    fs::create_dir_all(&src_dir).unwrap();
    fs::write(src_dir.join("lib.rs"), "fn main() {}\n").unwrap();

    // Run without --output-dir (should use .runs/tokmd/<run-id>)
    let mut cmd: Command = cargo_bin_cmd!("tokmd");
    cmd.current_dir(&work_dir)
        .arg("run")
        .arg("--name")
        .arg("test-run")
        .arg(".")
        .assert()
        .success();

    // Verify .runs/tokmd/test-run directory was created
    let expected_dir = work_dir.join(".runs/tokmd/test-run");
    assert!(
        expected_dir.exists(),
        ".runs/tokmd/test-run directory should be created at {:?}",
        expected_dir
    );
    assert!(
        expected_dir.join("receipt.json").exists(),
        "receipt.json should exist in default location"
    );
    assert!(
        expected_dir.join("lang.json").exists(),
        "lang.json should exist in default location"
    );
    assert!(
        expected_dir.join("export.jsonl").exists(),
        "export.jsonl should exist in default location"
    );
}

#[test]
fn test_run_with_redact_flag() {
    let dir = tempdir().unwrap();
    let output_dir = dir.path().join("run-redacted");

    let mut cmd: Command = cargo_bin_cmd!("tokmd");
    cmd.current_dir(common::fixture_root())
        .arg("run")
        .arg("--output-dir")
        .arg(output_dir.to_str().unwrap())
        .arg("--redact")
        .arg("paths")
        .arg(".")
        .assert()
        .success();

    // Check that export.jsonl was created and contains redacted paths
    let export_content = fs::read_to_string(output_dir.join("export.jsonl")).unwrap();

    // The meta line should contain "redact": "paths"
    assert!(
        export_content.contains(r#""redact":"paths""#),
        "export.jsonl should indicate redact mode is 'paths'"
    );

    // Paths should be hashed (16 hex chars followed by extension)
    // Check that we don't have the original .rs extension preceded by a recognizable path
    let lines: Vec<&str> = export_content.lines().collect();
    for line in lines.iter().skip(1) {
        // Skip meta line
        if line.contains(r#""type":"row""#) {
            // Paths should be hashed - they should be 16 hex chars followed by extension
            assert!(
                !line.contains("src/") && !line.contains("src\\"),
                "Redacted export should not contain original path segments"
            );
        }
    }
}

#[cfg(feature = "git")]
fn git_available() -> bool {
    ProcessCommand::new("git")
        .arg("--version")
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .map(|status| status.success())
        .unwrap_or(false)
}

#[cfg(feature = "git")]
fn git_cmd(dir: &std::path::Path, args: &[&str]) {
    let status = ProcessCommand::new("git")
        .args(args)
        .current_dir(dir)
        .status()
        .expect("git command failed to run");
    assert!(status.success(), "git command failed");
}

// =============================================================================
// Comprehensive Redaction Leak Tests for `tokmd run` Command
// =============================================================================
// These tests verify that when --redact is used, sensitive paths and patterns
// don't leak into any of the run artifacts (lang.json, module.json, export.jsonl)

#[test]
fn test_run_redact_lang_json_paths_redacted() {
    // Given: A directory with known path names
    // When: We run `tokmd run --redact paths`
    // Then: lang.json should have redacted scan.paths (no raw paths visible)
    let dir = tempdir().unwrap();
    let output_dir = dir.path().join("run-redact-lang");

    let secret_dir = dir.path().join("secret_project");
    fs::create_dir_all(&secret_dir).unwrap();
    fs::write(secret_dir.join("confidential.rs"), "fn secret() {}\n").unwrap();

    let mut cmd: Command = cargo_bin_cmd!("tokmd");
    cmd.current_dir(dir.path())
        .arg("run")
        .arg("--output-dir")
        .arg(output_dir.to_str().unwrap())
        .arg("--redact")
        .arg("paths")
        .arg("secret_project")
        .assert()
        .success();

    // Read lang.json and verify paths are redacted
    let lang_content = fs::read_to_string(output_dir.join("lang.json")).unwrap();

    // The input path "secret_project" should NOT appear in raw form
    assert!(
        !lang_content.contains("secret_project"),
        "lang.json should not contain raw path 'secret_project' when redacted.\nContent: {}",
        lang_content
    );

    // Should have scan.paths field with hashed values (16 hex chars)
    assert!(
        lang_content.contains(r#""paths":["#),
        "lang.json should contain paths array"
    );
}

#[test]
fn test_run_redact_module_json_scan_paths_redacted() {
    // Given: A directory with known path names
    // When: We run `tokmd run --redact paths`
    // Then: module.json should have redacted scan.paths (no raw paths in scan metadata)
    let dir = tempdir().unwrap();
    let output_dir = dir.path().join("run-redact-module");

    let proprietary_dir = dir.path().join("proprietary_code");
    fs::create_dir_all(&proprietary_dir).unwrap();
    fs::write(proprietary_dir.join("internal.rs"), "fn internal() {}\n").unwrap();

    let mut cmd: Command = cargo_bin_cmd!("tokmd");
    cmd.current_dir(dir.path())
        .arg("run")
        .arg("--output-dir")
        .arg(output_dir.to_str().unwrap())
        .arg("--redact")
        .arg("paths")
        .arg("proprietary_code")
        .assert()
        .success();

    // Read module.json and verify scan.paths are redacted
    let module_content = fs::read_to_string(output_dir.join("module.json")).unwrap();

    // Should have scan.paths field with hashed values (16 hex chars)
    assert!(
        module_content.contains(r#""paths":["#),
        "module.json should contain paths array"
    );

    // Parse JSON to check scan.paths specifically
    let json: serde_json::Value = serde_json::from_str(&module_content).unwrap();
    let scan_paths = json["scan"]["paths"]
        .as_array()
        .expect("scan.paths should be array");

    // Each path should be a 16-char hex hash (no raw path like "proprietary_code")
    for path in scan_paths {
        let path_str = path.as_str().unwrap();
        assert!(
            !path_str.contains("proprietary_code"),
            "scan.paths should not contain raw path 'proprietary_code'. Found: {}",
            path_str
        );
        // Hash is 16 hex chars
        assert!(
            path_str.len() >= 16 && path_str.chars().take(16).all(|c| c.is_ascii_hexdigit()),
            "scan.paths should contain hashed values. Found: {}",
            path_str
        );
    }

    // Note: With --redact paths, module names in rows may still be visible
    // This is expected behavior - use --redact all to also hash module names
}

#[test]
fn test_run_redact_excluded_patterns_in_lang_json() {
    // Given: Sensitive exclude patterns
    // When: We run `tokmd run --exclude <pattern> --redact paths`
    // Then: lang.json should have excluded_redacted: true and hashed patterns
    let dir = tempdir().unwrap();
    let output_dir = dir.path().join("run-redact-excluded-lang");

    let src_dir = dir.path().join("src");
    fs::create_dir_all(&src_dir).unwrap();
    fs::write(src_dir.join("app.rs"), "fn main() {}\n").unwrap();

    let sensitive_dir = dir.path().join("sensitive_data");
    fs::create_dir_all(&sensitive_dir).unwrap();
    fs::write(
        sensitive_dir.join("secrets.rs"),
        "const KEY: &str = \"\";\n",
    )
    .unwrap();

    let mut cmd: Command = cargo_bin_cmd!("tokmd");
    cmd.current_dir(dir.path())
        .arg("--exclude")
        .arg("**/sensitive_data/**")
        .arg("run")
        .arg("--output-dir")
        .arg(output_dir.to_str().unwrap())
        .arg("--redact")
        .arg("paths")
        .arg(".")
        .assert()
        .success();

    let lang_content = fs::read_to_string(output_dir.join("lang.json")).unwrap();

    // The exclude pattern should NOT appear in raw form
    assert!(
        !lang_content.contains("sensitive_data"),
        "lang.json should not contain raw exclude pattern 'sensitive_data'.\nContent: {}",
        lang_content
    );

    // excluded_redacted should be true
    assert!(
        lang_content.contains(r#""excluded_redacted":true"#),
        "lang.json should have excluded_redacted: true when exclude patterns are redacted.\nContent: {}",
        lang_content
    );
}

#[test]
fn test_run_redact_excluded_patterns_in_module_json() {
    // Given: Sensitive exclude patterns
    // When: We run `tokmd run --exclude <pattern> --redact paths`
    // Then: module.json should have excluded_redacted: true and hashed patterns
    let dir = tempdir().unwrap();
    let output_dir = dir.path().join("run-redact-excluded-module");

    let src_dir = dir.path().join("src");
    fs::create_dir_all(&src_dir).unwrap();
    fs::write(src_dir.join("lib.rs"), "pub fn hello() {}\n").unwrap();

    let private_dir = dir.path().join("private_module");
    fs::create_dir_all(&private_dir).unwrap();
    fs::write(private_dir.join("internal.rs"), "fn private() {}\n").unwrap();

    let mut cmd: Command = cargo_bin_cmd!("tokmd");
    cmd.current_dir(dir.path())
        .arg("--exclude")
        .arg("**/private_module/**")
        .arg("run")
        .arg("--output-dir")
        .arg(output_dir.to_str().unwrap())
        .arg("--redact")
        .arg("paths")
        .arg(".")
        .assert()
        .success();

    let module_content = fs::read_to_string(output_dir.join("module.json")).unwrap();

    // The exclude pattern should NOT appear in raw form
    assert!(
        !module_content.contains("private_module"),
        "module.json should not contain raw exclude pattern 'private_module'.\nContent: {}",
        module_content
    );

    // excluded_redacted should be true
    assert!(
        module_content.contains(r#""excluded_redacted":true"#),
        "module.json should have excluded_redacted: true when exclude patterns are redacted.\nContent: {}",
        module_content
    );
}

#[test]
fn test_run_redact_all_no_raw_paths_anywhere() {
    // Given: A directory structure with sensitive names
    // When: We run `tokmd run --exclude <pattern> --redact all`
    // Then: No raw path substrings should appear in ANY run artifact
    //       (--redact all hashes both paths AND module names)
    let dir = tempdir().unwrap();
    let output_dir = dir.path().join("run-full-redact");

    // Create structure with identifiable names
    let acme_corp = dir.path().join("acme_corp_internal");
    let secret_project = acme_corp.join("secret_project_x");
    fs::create_dir_all(&secret_project).unwrap();
    fs::write(secret_project.join("classified.rs"), "fn classified() {}\n").unwrap();
    fs::write(acme_corp.join("public.rs"), "fn public_api() {}\n").unwrap();

    let mut cmd: Command = cargo_bin_cmd!("tokmd");
    cmd.current_dir(dir.path())
        .arg("--exclude")
        .arg("**/node_modules/**")
        .arg("--exclude")
        .arg("**/vendor/**")
        .arg("run")
        .arg("--output-dir")
        .arg(output_dir.to_str().unwrap())
        .arg("--redact")
        .arg("all") // Use 'all' to redact module names too
        .arg("acme_corp_internal")
        .assert()
        .success();

    // Check ALL output files for leaks
    let lang_content = fs::read_to_string(output_dir.join("lang.json")).unwrap();
    let export_content = fs::read_to_string(output_dir.join("export.jsonl")).unwrap();
    let receipt_content = fs::read_to_string(output_dir.join("receipt.json")).unwrap();

    // In scan metadata, these should NOT appear (they're hashed in scan.paths and scan.excluded)
    let metadata_sensitive = ["acme_corp", "node_modules", "vendor"];

    for term in &metadata_sensitive {
        assert!(
            !lang_content.contains(term),
            "lang.json should not contain '{}' in scan metadata when redacted.\nContent: {}",
            term,
            lang_content
        );
    }

    // In export.jsonl file paths should be hashed
    assert!(
        !export_content.contains("classified.rs"),
        "export.jsonl should not contain raw filename 'classified.rs' when redacted.\nContent: {}",
        export_content
    );
    assert!(
        !export_content.contains("public.rs"),
        "export.jsonl should not contain raw filename 'public.rs' when redacted.\nContent: {}",
        export_content
    );

    // Exclude patterns should be hashed
    assert!(
        !export_content.contains("node_modules"),
        "export.jsonl should not contain raw exclude pattern when redacted"
    );
    assert!(
        !export_content.contains("vendor"),
        "export.jsonl should not contain raw exclude pattern when redacted"
    );

    // Receipt file should not contain sensitive terms
    for term in &metadata_sensitive {
        assert!(
            !receipt_content.contains(term),
            "receipt.json should not contain '{}' when redacted.\nContent: {}",
            term,
            receipt_content
        );
    }
}

#[test]
fn test_run_redact_paths_vs_all_difference() {
    // Given: A directory structure with identifiable names
    // This test documents the difference between --redact paths and --redact all
    let dir = tempdir().unwrap();
    let paths_output = dir.path().join("run-paths-only");
    let all_output = dir.path().join("run-all");

    let test_module = dir.path().join("test_module_name");
    fs::create_dir_all(&test_module).unwrap();
    fs::write(test_module.join("code.rs"), "fn code() {}\n").unwrap();

    // Run with --redact paths
    let mut cmd1: Command = cargo_bin_cmd!("tokmd");
    cmd1.current_dir(dir.path())
        .arg("run")
        .arg("--output-dir")
        .arg(paths_output.to_str().unwrap())
        .arg("--redact")
        .arg("paths")
        .arg("test_module_name")
        .assert()
        .success();

    // Run with --redact all
    let mut cmd2: Command = cargo_bin_cmd!("tokmd");
    cmd2.current_dir(dir.path())
        .arg("run")
        .arg("--output-dir")
        .arg(all_output.to_str().unwrap())
        .arg("--redact")
        .arg("all")
        .arg("test_module_name")
        .assert()
        .success();

    let paths_export = fs::read_to_string(paths_output.join("export.jsonl")).unwrap();
    let all_export = fs::read_to_string(all_output.join("export.jsonl")).unwrap();

    // With --redact paths: file paths are hashed, but module names MAY be visible
    // (Current behavior allows module names to be visible with paths redaction)
    assert!(
        !paths_export.contains("code.rs"),
        "--redact paths should hash file paths"
    );

    // With --redact all: both file paths AND module names are hashed
    assert!(
        !all_export.contains("code.rs"),
        "--redact all should hash file paths"
    );
    assert!(
        !all_export.contains("test_module_name"),
        "--redact all should hash module names in export rows"
    );
}

#[test]
fn test_run_redact_all_hashes_modules_too() {
    // Given: A module structure
    // When: We run `tokmd run --redact all`
    // Then: Module names in export.jsonl should also be hashed
    let dir = tempdir().unwrap();
    let output_dir = dir.path().join("run-redact-all");

    let identifiable_module = dir.path().join("identifiable_module_name");
    fs::create_dir_all(&identifiable_module).unwrap();
    fs::write(identifiable_module.join("code.rs"), "fn module_code() {}\n").unwrap();

    let mut cmd: Command = cargo_bin_cmd!("tokmd");
    cmd.current_dir(dir.path())
        .arg("run")
        .arg("--output-dir")
        .arg(output_dir.to_str().unwrap())
        .arg("--redact")
        .arg("all")
        .arg(".")
        .assert()
        .success();

    let export_content = fs::read_to_string(output_dir.join("export.jsonl")).unwrap();

    // Module name should be hashed (not appear in raw form)
    assert!(
        !export_content.contains("identifiable_module_name"),
        "export.jsonl should not contain raw module name when --redact all is used.\nContent: {}",
        export_content
    );

    // Should contain hash patterns (16 hex chars) in module field
    let has_hashed_module = export_content
        .lines()
        .any(|line| line.contains(r#""module":"#) && line.contains(r#""type":"row""#));
    assert!(
        has_hashed_module,
        "export.jsonl should contain rows with module field"
    );
}

#[test]
fn test_run_redact_consistency_across_artifacts() {
    // Given: Same scan with redaction
    // When: We check lang.json, module.json, and export.jsonl
    // Then: All should have consistent redaction markers in scan metadata
    let dir = tempdir().unwrap();
    let output_dir = dir.path().join("run-redact-consistency");

    let src_dir = dir.path().join("consistent_source");
    fs::create_dir_all(&src_dir).unwrap();
    fs::write(src_dir.join("main.rs"), "fn main() {}\n").unwrap();

    let mut cmd: Command = cargo_bin_cmd!("tokmd");
    cmd.current_dir(dir.path())
        .arg("--exclude")
        .arg("**/excluded_dir/**")
        .arg("run")
        .arg("--output-dir")
        .arg(output_dir.to_str().unwrap())
        .arg("--redact")
        .arg("paths")
        .arg("consistent_source")
        .assert()
        .success();

    let lang_content = fs::read_to_string(output_dir.join("lang.json")).unwrap();
    let module_content = fs::read_to_string(output_dir.join("module.json")).unwrap();
    let export_content = fs::read_to_string(output_dir.join("export.jsonl")).unwrap();

    // All should have excluded_redacted: true
    assert!(
        lang_content.contains(r#""excluded_redacted":true"#),
        "lang.json should have excluded_redacted: true"
    );
    assert!(
        module_content.contains(r#""excluded_redacted":true"#),
        "module.json should have excluded_redacted: true"
    );
    assert!(
        export_content.contains(r#""excluded_redacted":true"#),
        "export.jsonl should have excluded_redacted: true"
    );

    // All should have redact mode indicator in export.jsonl
    assert!(
        export_content.contains(r#""redact":"paths""#),
        "export.jsonl should indicate redact mode is 'paths'"
    );

    // scan.paths should NOT contain raw input path in any file
    assert!(
        !lang_content.contains(r#""paths":["consistent_source"]"#),
        "lang.json scan.paths should not contain raw path"
    );
    assert!(
        !module_content.contains(r#""paths":["consistent_source"]"#),
        "module.json scan.paths should not contain raw path"
    );
    assert!(
        !export_content.contains(r#""paths":["consistent_source"]"#),
        "export.jsonl scan.paths should not contain raw path"
    );

    // scan.excluded should NOT contain raw exclude pattern
    assert!(
        !lang_content.contains("excluded_dir"),
        "lang.json should not contain raw exclude pattern"
    );
    assert!(
        !module_content.contains("excluded_dir"),
        "module.json should not contain raw exclude pattern"
    );
    assert!(
        !export_content.contains("excluded_dir"),
        "export.jsonl should not contain raw exclude pattern"
    );

    // export.jsonl file paths should be hashed
    assert!(
        !export_content.contains("main.rs"),
        "export.jsonl should not contain raw filename when redacted"
    );
}

#[test]
fn test_run_without_redact_shows_raw_paths() {
    // Given: A directory with identifiable paths
    // When: We run `tokmd run` WITHOUT --redact
    // Then: Raw paths should appear in the output (proving redaction is opt-in)
    let dir = tempdir().unwrap();
    let output_dir = dir.path().join("run-no-redact");

    let visible_dir = dir.path().join("visible_path");
    fs::create_dir_all(&visible_dir).unwrap();
    fs::write(visible_dir.join("exposed.rs"), "fn exposed() {}\n").unwrap();

    let mut cmd: Command = cargo_bin_cmd!("tokmd");
    cmd.current_dir(dir.path())
        .arg("--exclude")
        .arg("**/should_be_visible/**")
        .arg("run")
        .arg("--output-dir")
        .arg(output_dir.to_str().unwrap())
        .arg("visible_path")
        .assert()
        .success();

    let lang_content = fs::read_to_string(output_dir.join("lang.json")).unwrap();
    let module_content = fs::read_to_string(output_dir.join("module.json")).unwrap();
    let export_content = fs::read_to_string(output_dir.join("export.jsonl")).unwrap();

    // Without redaction, raw paths SHOULD appear
    assert!(
        lang_content.contains("visible_path"),
        "lang.json SHOULD contain raw path when not redacted"
    );
    assert!(
        module_content.contains("visible_path"),
        "module.json SHOULD contain raw path when not redacted"
    );
    // export.jsonl contains file paths in rows
    assert!(
        export_content.contains("exposed.rs"),
        "export.jsonl SHOULD contain raw filenames when not redacted"
    );
    // Exclude patterns should be visible
    assert!(
        lang_content.contains("should_be_visible"),
        "lang.json SHOULD contain raw exclude patterns when not redacted"
    );
}

#[test]
fn test_run_redact_with_absolute_paths() {
    // Given: Absolute paths as input (which might reveal filesystem structure)
    // When: We run `tokmd run --redact paths`
    // Then: Absolute path components should be redacted
    let dir = tempdir().unwrap();
    let output_dir = dir.path().join("run-redact-absolute");

    let src_dir = dir.path().join("src");
    fs::create_dir_all(&src_dir).unwrap();
    fs::write(src_dir.join("lib.rs"), "pub fn lib() {}\n").unwrap();

    // Use absolute path as input
    let absolute_src = src_dir.canonicalize().unwrap();

    let mut cmd: Command = cargo_bin_cmd!("tokmd");
    cmd.current_dir(dir.path())
        .arg("run")
        .arg("--output-dir")
        .arg(output_dir.to_str().unwrap())
        .arg("--redact")
        .arg("paths")
        .arg(absolute_src.to_str().unwrap())
        .assert()
        .success();

    let lang_content = fs::read_to_string(output_dir.join("lang.json")).unwrap();

    // The absolute path should NOT appear (it would reveal filesystem structure)
    // Extract parent directory name from the temp dir to check
    let temp_dir_name = dir.path().file_name().unwrap().to_str().unwrap();
    assert!(
        !lang_content.contains(temp_dir_name),
        "lang.json should not contain temp directory name '{}' when redacted.\nContent: {}",
        temp_dir_name,
        lang_content
    );
}

#[test]
#[cfg(feature = "git")]
fn test_diff_git_refs() {
    if !git_available() {
        return;
    }

    let dir = tempdir().unwrap();
    let repo = dir.path().join("repo");
    fs::create_dir_all(&repo).unwrap();

    git_cmd(&repo, &["init"]);
    git_cmd(&repo, &["config", "user.email", "test@example.com"]);
    git_cmd(&repo, &["config", "user.name", "Tokmd Test"]);

    let src_dir = repo.join("src");
    fs::create_dir_all(&src_dir).unwrap();
    fs::write(src_dir.join("lib.rs"), "fn a() {}\n").unwrap();
    git_cmd(&repo, &["add", "."]);
    git_cmd(&repo, &["commit", "-m", "initial"]);

    fs::write(src_dir.join("lib.rs"), "fn a() {}\nfn b() {}\n").unwrap();
    git_cmd(&repo, &["add", "."]);
    git_cmd(&repo, &["commit", "-m", "add b"]);

    let mut cmd: Command = cargo_bin_cmd!("tokmd");
    cmd.current_dir(&repo)
        .arg("diff")
        .arg("HEAD~1")
        .arg("HEAD")
        .assert()
        .success()
        .stdout(predicate::str::contains("## Diff:"))
        .stdout(predicate::str::contains("Rust"));
}
