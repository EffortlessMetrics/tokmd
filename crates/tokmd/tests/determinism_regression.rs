//! Regression tests for determinism across lang, module, and export commands.
//!
//! This module verifies that all three commands produce byte-identical output
//! across multiple consecutive runs, ensuring deterministic output for CI/CD
//! pipelines and reproducible builds.

mod common;

use assert_cmd::Command;

fn tokmd_cmd() -> Command {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_tokmd"));
    cmd.current_dir(common::fixture_root());
    cmd
}

/// Normalize non-deterministic fields (timestamps, tool version) so we can
/// compare outputs byte-for-byte.
fn normalize_envelope(output: &str) -> String {
    let re_ts = regex::Regex::new(r#""generated_at_ms":\d+"#).expect("valid regex");
    let s = re_ts
        .replace_all(output, r#""generated_at_ms":0"#)
        .to_string();
    let re_ver =
        regex::Regex::new(r#"("tool":\{"name":"tokmd","version":")[^"]+"#).expect("valid regex");
    re_ver.replace_all(&s, r#"${1}0.0.0"#).to_string()
}

// ---------------------------------------------------------------------------
// Regression: Verify determinism across all commands
// ---------------------------------------------------------------------------

/// Verify lang command produces identical output across 5 consecutive runs.
#[test]
fn lang_command_produces_deterministic_output() {
    let run = || {
        let o = tokmd_cmd()
            .args(["lang", "--format", "json"])
            .output()
            .expect("run");
        normalize_envelope(&String::from_utf8_lossy(&o.stdout))
    };

    let results: Vec<String> = (0..5).map(|_| run()).collect();

    for i in 1..results.len() {
        assert_eq!(
            results[0],
            results[i],
            "lang command output differs between run 1 and run {}",
            i + 1
        );
    }
}

/// Verify module command produces identical output across 5 consecutive runs.
#[test]
fn module_command_produces_deterministic_output() {
    let run = || {
        let o = tokmd_cmd()
            .args(["module", "--format", "json"])
            .output()
            .expect("run");
        normalize_envelope(&String::from_utf8_lossy(&o.stdout))
    };

    let results: Vec<String> = (0..5).map(|_| run()).collect();

    for i in 1..results.len() {
        assert_eq!(
            results[0],
            results[i],
            "module command output differs between run 1 and run {}",
            i + 1
        );
    }
}

/// Verify export command produces identical output across 5 consecutive runs.
#[test]
fn export_command_produces_deterministic_output() {
    let run = || {
        let o = tokmd_cmd()
            .args(["export", "--format", "json"])
            .output()
            .expect("run");
        normalize_envelope(&String::from_utf8_lossy(&o.stdout))
    };

    let results: Vec<String> = (0..5).map(|_| run()).collect();

    for i in 1..results.len() {
        assert_eq!(
            results[0],
            results[i],
            "export command output differs between run 1 and run {}",
            i + 1
        );
    }
}

/// Verify all three commands are deterministic in CSV format.
#[test]
fn all_commands_deterministic_csv_format() {
    let run_lang_csv = || {
        let o = tokmd_cmd()
            .args(["lang", "--format", "csv"])
            .output()
            .expect("run");
        String::from_utf8_lossy(&o.stdout).to_string()
    };

    let run_module_csv = || {
        let o = tokmd_cmd()
            .args(["module", "--format", "csv"])
            .output()
            .expect("run");
        String::from_utf8_lossy(&o.stdout).to_string()
    };

    let run_export_csv = || {
        let o = tokmd_cmd()
            .args(["export", "--format", "csv"])
            .output()
            .expect("run");
        String::from_utf8_lossy(&o.stdout).to_string()
    };

    // Each command should be deterministic
    assert_eq!(run_lang_csv(), run_lang_csv(), "lang CSV not deterministic");
    assert_eq!(
        run_module_csv(),
        run_module_csv(),
        "module CSV not deterministic"
    );
    assert_eq!(
        run_export_csv(),
        run_export_csv(),
        "export CSV not deterministic"
    );
}

/// Verify TSV output is deterministic for all commands.
#[test]
fn all_commands_deterministic_tsv_format() {
    let run_lang_tsv = || {
        let o = tokmd_cmd()
            .args(["lang", "--format", "tsv"])
            .output()
            .expect("run");
        String::from_utf8_lossy(&o.stdout).to_string()
    };

    let run_module_tsv = || {
        let o = tokmd_cmd()
            .args(["module", "--format", "tsv"])
            .output()
            .expect("run");
        String::from_utf8_lossy(&o.stdout).to_string()
    };

    let run_export_tsv = || {
        let o = tokmd_cmd()
            .args(["export", "--format", "tsv"])
            .output()
            .expect("run");
        String::from_utf8_lossy(&o.stdout).to_string()
    };

    // Each command should be deterministic
    assert_eq!(run_lang_tsv(), run_lang_tsv(), "lang TSV not deterministic");
    assert_eq!(
        run_module_tsv(),
        run_module_tsv(),
        "module TSV not deterministic"
    );
    assert_eq!(
        run_export_tsv(),
        run_export_tsv(),
        "export TSV not deterministic"
    );
}

/// Verify Markdown format is deterministic for all commands.
#[test]
fn all_commands_deterministic_markdown_format() {
    let run_lang_md = || {
        let o = tokmd_cmd()
            .args(["lang", "--format", "md"])
            .output()
            .expect("run");
        String::from_utf8_lossy(&o.stdout).to_string()
    };

    let run_module_md = || {
        let o = tokmd_cmd()
            .args(["module", "--format", "md"])
            .output()
            .expect("run");
        String::from_utf8_lossy(&o.stdout).to_string()
    };

    let run_export_jsonl = || {
        let o = tokmd_cmd()
            .args(["export", "--format", "jsonl"])
            .output()
            .expect("run");
        normalize_envelope(&String::from_utf8_lossy(&o.stdout))
    };

    // Each command should be deterministic
    assert_eq!(
        run_lang_md(),
        run_lang_md(),
        "lang Markdown not deterministic"
    );
    assert_eq!(
        run_module_md(),
        run_module_md(),
        "module Markdown not deterministic"
    );
    assert_eq!(
        run_export_jsonl(),
        run_export_jsonl(),
        "export JSONL not deterministic"
    );
}

/// Verify that row ordering is stable across runs (no randomization).
#[test]
fn row_ordering_is_stable() {
    let get_codes = |cmd_args: &[&str]| -> Vec<u64> {
        let o = tokmd_cmd().args(cmd_args).output().expect("run");
        let json: serde_json::Value = serde_json::from_slice(&o.stdout).expect("valid JSON");
        json.get("rows")
            .and_then(|v| v.as_array())
            .unwrap_or(&vec![])
            .iter()
            .map(|r| r["code"].as_u64().unwrap_or(0))
            .collect()
    };

    let lang_codes_1 = get_codes(&["lang", "--format", "json"]);
    let lang_codes_2 = get_codes(&["lang", "--format", "json"]);
    assert_eq!(
        lang_codes_1, lang_codes_2,
        "lang row order changed between runs"
    );

    let module_codes_1 = get_codes(&["module", "--format", "json"]);
    let module_codes_2 = get_codes(&["module", "--format", "json"]);
    assert_eq!(
        module_codes_1, module_codes_2,
        "module row order changed between runs"
    );

    let export_codes_1 = get_codes(&["export", "--format", "json"]);
    let export_codes_2 = get_codes(&["export", "--format", "json"]);
    assert_eq!(
        export_codes_1, export_codes_2,
        "export row order changed between runs"
    );
}
