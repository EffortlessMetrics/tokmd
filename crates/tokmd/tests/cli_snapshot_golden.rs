//! Golden snapshot tests for CLI output formats.
//!
//! These tests capture the full output of each major CLI command and format
//! combination as insta snapshots.  Any unintentional change to the output
//! format will cause a test failure, making regressions easy to spot.

mod common;

use assert_cmd::Command;

fn tokmd_cmd() -> Command {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_tokmd"));
    cmd.current_dir(common::fixture_root());
    cmd
}

fn run_tokmd(args: &[&str]) -> String {
    let output = tokmd_cmd()
        .args(args)
        .output()
        .unwrap_or_else(|err| panic!("failed to run tokmd {}: {err}", args.join(" ")));

    assert!(
        output.status.success(),
        "tokmd {} failed with status {}\nstderr:\n{}",
        args.join(" "),
        output.status,
        String::from_utf8_lossy(&output.stderr)
    );

    String::from_utf8(output.stdout).expect("tokmd output should be valid UTF-8")
}

fn assert_snapshot_normalized(name: &str, stdout: &str) {
    insta::assert_snapshot!(name, normalize(stdout));
}

/// Replace dynamic values (timestamps, versions, absolute paths) with stable
/// placeholders so snapshots are deterministic across machines and runs.
fn normalize(output: &str) -> String {
    let re_ts = regex::Regex::new(r#""generated_at_ms":\s*\d+"#).unwrap();
    let s = re_ts
        .replace_all(output, r#""generated_at_ms":0"#)
        .to_string();

    let re_ver =
        regex::Regex::new(r#"("tool":\s*\{\s*"name":\s*"tokmd",\s*"version":\s*")[^"]+"#).unwrap();
    let s = re_ver.replace_all(&s, r#"${1}0.0.0"#).to_string();

    // Normalize --version output line (e.g. "tokmd 0.42.1" -> "tokmd <VERSION>")
    let re_version_line = regex::Regex::new(r"tokmd \d+\.\d+\.\d+(?:[-+][0-9A-Za-z.-]+)?").unwrap();
    let s = re_version_line
        .replace_all(&s, "tokmd <VERSION>")
        .to_string();

    // Normalize absolute paths that may leak into scan.paths
    let re_abs = regex::Regex::new(r#""paths":\["[^"]*"\]"#).unwrap();
    let s = re_abs.replace_all(&s, r#""paths":["<ROOT>"]"#).to_string();

    // Normalize analysis-specific dynamic fields.
    let re_target = regex::Regex::new(r#""target_path":\s*"[^"]*""#).unwrap();
    let s = re_target
        .replace_all(&s, r#""target_path":"<ROOT>""#)
        .to_string();
    let re_base_signature = regex::Regex::new(r#""base_signature":\s*"[^"]+""#).unwrap();
    let s = re_base_signature
        .replace_all(&s, r#""base_signature":"<BASE_SIGNATURE>""#)
        .to_string();
    let re_integrity_hash = regex::Regex::new(r#""hash":\s*"[0-9a-f]{64}""#).unwrap();
    let s = re_integrity_hash
        .replace_all(&s, r#""hash":"<INTEGRITY_HASH>""#)
        .to_string();
    let re_markdown_hash = regex::Regex::new(r#"Hash: `[0-9a-f]{64}`"#).unwrap();
    let s = re_markdown_hash
        .replace_all(&s, "Hash: `<INTEGRITY_HASH>`")
        .to_string();

    // Normalize binary name across platforms (tokmd.exe on Windows -> tokmd)
    s.replace("tokmd.exe", "tokmd")
}

// ---------------------------------------------------------------------------
// 1. JSON output snapshots
// ---------------------------------------------------------------------------

#[test]
fn snapshot_lang_json() {
    let stdout = run_tokmd(&["lang", "--format", "json"]);
    assert_snapshot_normalized("lang_json", &stdout);
}

#[test]
fn snapshot_lang_json_structure() {
    let stdout = run_tokmd(&["lang", "--format", "json"]);
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();

    // Verify structural keys are present — the snapshot captures exact values
    assert_eq!(json["mode"], "lang");
    assert!(json["schema_version"].is_number());
    assert!(json["rows"].is_array());
    assert!(json["total"].is_object());
    assert!(json["tool"].is_object());
}

// ---------------------------------------------------------------------------
// 2. Markdown output snapshots (default format)
// ---------------------------------------------------------------------------

#[test]
fn snapshot_lang_markdown() {
    let stdout = run_tokmd(&["lang"]);
    insta::assert_snapshot!("lang_markdown", stdout);
}

// ---------------------------------------------------------------------------
// 3. TSV output snapshots
// ---------------------------------------------------------------------------

#[test]
fn snapshot_lang_tsv() {
    let stdout = run_tokmd(&["lang", "--format", "tsv"]);
    insta::assert_snapshot!("lang_tsv", stdout);
}

// ---------------------------------------------------------------------------
// 4. Module output snapshots
// ---------------------------------------------------------------------------

#[test]
fn snapshot_module_json() {
    let stdout = run_tokmd(&["module", "--format", "json"]);
    assert_snapshot_normalized("module_json", &stdout);
}

#[test]
fn snapshot_module_markdown() {
    let stdout = run_tokmd(&["module"]);
    insta::assert_snapshot!("module_markdown", stdout);
}

#[test]
fn snapshot_module_tsv() {
    let stdout = run_tokmd(&["module", "--format", "tsv"]);
    insta::assert_snapshot!("module_tsv", stdout);
}

// ---------------------------------------------------------------------------
// 5. Export output snapshots
// ---------------------------------------------------------------------------

#[test]
fn snapshot_export_json() {
    let stdout = run_tokmd(&["export", "--format", "json"]);
    assert_snapshot_normalized("export_json", &stdout);
}

#[test]
fn snapshot_export_jsonl() {
    let stdout = run_tokmd(&["export", "--format", "jsonl"]);
    assert_snapshot_normalized("export_jsonl", &stdout);
}

#[test]
fn snapshot_export_csv() {
    let stdout = run_tokmd(&["export", "--format", "csv"]);
    insta::assert_snapshot!("export_csv", stdout);
}

// ---------------------------------------------------------------------------
// 6. Analyze output snapshots
// ---------------------------------------------------------------------------

#[test]
#[cfg(feature = "analysis")]
fn snapshot_analyze_json() {
    let stdout = run_tokmd(&[
        "analyze", ".", "--preset", "receipt", "--format", "json", "--no-git",
    ]);
    assert_snapshot_normalized("analyze_json", &stdout);
}

#[test]
#[cfg(feature = "analysis")]
fn snapshot_analyze_markdown() {
    let stdout = run_tokmd(&[
        "analyze", ".", "--preset", "receipt", "--format", "md", "--no-git",
    ]);
    assert_snapshot_normalized("analyze_markdown", &stdout);
}

// ---------------------------------------------------------------------------
// 7. Version output snapshot
// ---------------------------------------------------------------------------

#[test]
fn snapshot_version() {
    let stdout = run_tokmd(&["--version"]);
    assert_snapshot_normalized("version", &stdout);
}

// ---------------------------------------------------------------------------
// 8. Help output snapshot
// ---------------------------------------------------------------------------

#[test]
fn snapshot_help() {
    let stdout = run_tokmd(&["--help"]);
    // Normalize the version in the help header.
    assert_snapshot_normalized("help", &stdout);
}
