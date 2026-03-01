use assert_cmd::Command;
use assert_cmd::cargo::cargo_bin_cmd;
use predicates::str::contains;
use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;

/// Build a `tokmd` command pointed at the test fixtures directory.
fn tokmd() -> Command {
    let mut cmd: Command = cargo_bin_cmd!("tokmd");
    let fixtures = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("data");
    cmd.current_dir(&fixtures);
    cmd
}

/// Build a `tokmd` command pointed at the given directory.
fn tokmd_at(dir: &std::path::Path) -> Command {
    let mut cmd: Command = cargo_bin_cmd!("tokmd");
    cmd.current_dir(dir);
    cmd
}

/// Create a temp directory containing sample source files for predictable output.
fn sample_dir() -> TempDir {
    let tmp = tempfile::tempdir().unwrap();
    fs::write(
        tmp.path().join("main.rs"),
        "fn main() {\n    println!(\"hello\");\n}\n",
    )
    .unwrap();
    fs::write(
        tmp.path().join("lib.rs"),
        "pub fn add(a: i32, b: i32) -> i32 {\n    a + b\n}\n",
    )
    .unwrap();
    fs::write(
        tmp.path().join("app.js"),
        "function greet(name) {\n    return `Hello, ${name}`;\n}\n",
    )
    .unwrap();
    tmp
}

// ---------------------------------------------------------------------------
// 1. Default format (jsonl)
// ---------------------------------------------------------------------------
#[test]
fn export_default_format_is_jsonl() {
    let output = tokmd()
        .arg("export")
        .output()
        .expect("failed to run tokmd export");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    // JSONL: every non-empty line should parse as JSON
    for line in stdout.lines() {
        if line.trim().is_empty() {
            continue;
        }
        serde_json::from_str::<serde_json::Value>(line)
            .unwrap_or_else(|e| panic!("line is not valid JSON: {e}\nline: {line}"));
    }
}

// ---------------------------------------------------------------------------
// 2. --format csv
// ---------------------------------------------------------------------------
#[test]
fn export_format_csv() {
    tokmd()
        .args(["export", "--format", "csv"])
        .assert()
        .success()
        .stdout(contains(","));
}

// ---------------------------------------------------------------------------
// 3. --format jsonl (explicit)
// ---------------------------------------------------------------------------
#[test]
fn export_format_jsonl_explicit() {
    let output = tokmd()
        .args(["export", "--format", "jsonl"])
        .output()
        .expect("failed to run");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        !stdout.trim().is_empty(),
        "JSONL output should not be empty for test fixtures"
    );
}

// ---------------------------------------------------------------------------
// 4. --format json (JSON envelope)
// ---------------------------------------------------------------------------
#[test]
fn export_format_json_envelope() {
    let output = tokmd()
        .args(["export", "--format", "json"])
        .output()
        .expect("failed to run");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    let parsed: serde_json::Value =
        serde_json::from_str(&stdout).expect("output should be valid JSON");
    assert!(parsed.is_object(), "JSON envelope should be an object");
}

// ---------------------------------------------------------------------------
// 5. Export from temp directory with sample files
// ---------------------------------------------------------------------------
#[test]
fn export_from_temp_dir() {
    let tmp = sample_dir();
    let output = tokmd_at(tmp.path())
        .args(["export", "--format", "jsonl"])
        .output()
        .expect("failed to run");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    let lines: Vec<&str> = stdout.lines().filter(|l| !l.trim().is_empty()).collect();
    // At least a meta line + some file rows
    assert!(
        lines.len() >= 2,
        "expected at least 2 JSONL lines (meta + rows), got {}",
        lines.len()
    );
}

// ---------------------------------------------------------------------------
// 6. Verify JSONL output has one JSON object per line
// ---------------------------------------------------------------------------
#[test]
fn export_jsonl_one_object_per_line() {
    let tmp = sample_dir();
    let output = tokmd_at(tmp.path())
        .args(["export", "--format", "jsonl"])
        .output()
        .expect("failed to run");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    for (i, line) in stdout.lines().enumerate() {
        if line.trim().is_empty() {
            continue;
        }
        let val: serde_json::Value = serde_json::from_str(line)
            .unwrap_or_else(|e| panic!("line {i} is not valid JSON: {e}"));
        assert!(
            val.is_object(),
            "line {i} should be a JSON object, got: {val}"
        );
    }
}

// ---------------------------------------------------------------------------
// 7. Verify CSV output has a header row
// ---------------------------------------------------------------------------
#[test]
fn export_csv_has_header_row() {
    let tmp = sample_dir();
    let output = tokmd_at(tmp.path())
        .args(["export", "--format", "csv"])
        .output()
        .expect("failed to run");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    let first_line = stdout
        .lines()
        .next()
        .expect("CSV output should not be empty");
    // Header should contain common column names
    assert!(
        first_line.contains("language") || first_line.contains("lang"),
        "CSV header should contain a language column, got: {first_line}"
    );
    assert!(
        first_line.contains("code"),
        "CSV header should contain a code column, got: {first_line}"
    );
}

// ---------------------------------------------------------------------------
// 8. Verify JSON envelope has schema_version field
// ---------------------------------------------------------------------------
#[test]
fn export_json_has_schema_version() {
    let tmp = sample_dir();
    let output = tokmd_at(tmp.path())
        .args(["export", "--format", "json"])
        .output()
        .expect("failed to run");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    let parsed: serde_json::Value =
        serde_json::from_str(&stdout).expect("output should be valid JSON");
    assert!(
        parsed.get("schema_version").is_some(),
        "JSON envelope should have schema_version field, keys: {:?}",
        parsed.as_object().map(|o| o.keys().collect::<Vec<_>>())
    );
    assert!(
        parsed["schema_version"].is_number(),
        "schema_version should be a number"
    );
}

// ---------------------------------------------------------------------------
// 9. Export with --children separate vs --children parents-only
// ---------------------------------------------------------------------------
#[test]
fn export_children_separate() {
    let tmp = sample_dir();
    tokmd_at(tmp.path())
        .args(["export", "--format", "jsonl", "--children", "separate"])
        .assert()
        .success();
}

#[test]
fn export_children_parents_only() {
    let tmp = sample_dir();
    tokmd_at(tmp.path())
        .args(["export", "--format", "jsonl", "--children", "parents-only"])
        .assert()
        .success();
}

// ---------------------------------------------------------------------------
// 10. Export empty directory produces valid but empty output
// ---------------------------------------------------------------------------
#[test]
fn export_empty_dir_jsonl() {
    let tmp = tempfile::tempdir().unwrap();
    let output = tokmd_at(tmp.path())
        .args(["export", "--format", "jsonl"])
        .output()
        .expect("failed to run");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    // Should still be valid JSONL (possibly just a meta line or empty)
    for line in stdout.lines() {
        if line.trim().is_empty() {
            continue;
        }
        serde_json::from_str::<serde_json::Value>(line)
            .unwrap_or_else(|e| panic!("line is not valid JSON: {e}"));
    }
}

#[test]
fn export_empty_dir_csv() {
    let tmp = tempfile::tempdir().unwrap();
    let output = tokmd_at(tmp.path())
        .args(["export", "--format", "csv"])
        .output()
        .expect("failed to run");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    // CSV should still have a header even with no data rows
    let lines: Vec<&str> = stdout.lines().collect();
    if !lines.is_empty() {
        assert!(
            lines[0].contains("code") || lines[0].contains("language") || lines[0].contains("lang"),
            "first line should be a header"
        );
    }
}

#[test]
fn export_empty_dir_json() {
    let tmp = tempfile::tempdir().unwrap();
    let output = tokmd_at(tmp.path())
        .args(["export", "--format", "json"])
        .output()
        .expect("failed to run");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    let parsed: serde_json::Value =
        serde_json::from_str(&stdout).expect("output should be valid JSON even for empty dir");
    assert!(parsed.is_object());
}

// ---------------------------------------------------------------------------
// Bonus: --format cyclonedx produces valid JSON
// ---------------------------------------------------------------------------
#[test]
fn export_cyclonedx_valid_json() {
    let tmp = sample_dir();
    let output = tokmd_at(tmp.path())
        .args(["export", "--format", "cyclonedx"])
        .output()
        .expect("failed to run");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    let parsed: serde_json::Value =
        serde_json::from_str(&stdout).expect("CycloneDX output should be valid JSON");
    assert!(parsed.is_object());
}

// ---------------------------------------------------------------------------
// Bonus: --output writes to file
// ---------------------------------------------------------------------------
#[test]
fn export_output_to_file() {
    let tmp = sample_dir();
    let out_file = tmp.path().join("export.jsonl");
    tokmd_at(tmp.path())
        .args(["export", "--format", "jsonl", "--output"])
        .arg(&out_file)
        .assert()
        .success();
    assert!(out_file.exists(), "output file should exist");
    let contents = fs::read_to_string(&out_file).unwrap();
    assert!(
        !contents.trim().is_empty(),
        "output file should not be empty"
    );
}

// ---------------------------------------------------------------------------
// Bonus: --min-code filters small files
// ---------------------------------------------------------------------------
#[test]
fn export_min_code_filter() {
    let tmp = sample_dir();
    let output = tokmd_at(tmp.path())
        .args(["export", "--format", "jsonl", "--min-code", "9999"])
        .output()
        .expect("failed to run");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    let row_lines: Vec<&str> = stdout
        .lines()
        .filter(|l| {
            if l.trim().is_empty() {
                return false;
            }
            let v: serde_json::Value = serde_json::from_str(l).unwrap();
            v.get("type").and_then(|t| t.as_str()) == Some("row")
        })
        .collect();
    assert!(
        row_lines.is_empty(),
        "min-code 9999 should filter out all rows from small files, got {} rows",
        row_lines.len()
    );
}
