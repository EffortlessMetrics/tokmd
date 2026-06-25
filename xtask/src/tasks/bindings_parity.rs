//! Verify tokmd-core FFI JSON envelope parity against shared fixtures.
//!
//! This command is the repo-native entrypoint for issue #267: it exercises
//! `run_json` through deterministic fixture cases and runs the existing
//! Rust integration/unit tests that guard Python and Node binding surfaces.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{Context, Result, bail};
use serde::Deserialize;
use serde_json::{Map, Value};
use tokmd_core::ffi::{run_json, schema_version, version};

use crate::cli::BindingsParityArgs;

const MANIFEST_SCHEMA: &str = "tokmd.bindings_parity_manifest.v1";
const DEFAULT_MANIFEST: &str = "fixtures/bindings-parity/manifest.json";

#[derive(Debug, Deserialize)]
struct Manifest {
    schema: String,
    #[serde(default)]
    agent_charter: Option<String>,
    cases: Vec<Case>,
}

#[derive(Debug, Deserialize)]
struct Case {
    id: String,
    mode: String,
    args: String,
    expect_ok: bool,
    #[serde(default)]
    golden: Option<String>,
    #[serde(default)]
    data_has_keys: Vec<String>,
    #[serde(default)]
    data_contains: Map<String, Value>,
    #[serde(default)]
    error_contains: Map<String, Value>,
}

pub fn run(args: BindingsParityArgs) -> Result<()> {
    if !args.check {
        bail!("bindings-parity requires `--check` (update mode is not implemented)");
    }

    let repo_root = find_repo_root()?;
    let manifest_path = args
        .manifest
        .clone()
        .unwrap_or_else(|| repo_root.join(DEFAULT_MANIFEST));

    let report = verify_manifest(&repo_root, &manifest_path)?;
    print_fixture_report(&report);

    if args.skip_cargo_tests {
        println!("Skipping cargo test steps (--skip-cargo-tests).");
    } else {
        run_cargo_tests(&repo_root)?;
    }

    if let Some(path) = &args.receipt {
        write_receipt(path, &report)?;
    }

    println!(
        "Bindings parity checks passed ({} fixture case(s)).",
        report.cases.len()
    );
    Ok(())
}

#[derive(Debug, serde::Serialize)]
struct ParityReport {
    schema: &'static str,
    ok: bool,
    manifest: String,
    agent_charter: Option<String>,
    cases: Vec<CaseReport>,
}

#[derive(Debug, serde::Serialize)]
struct CaseReport {
    id: String,
    mode: String,
    expect_ok: bool,
    actual_ok: bool,
}

fn verify_manifest(repo_root: &Path, manifest_path: &Path) -> Result<ParityReport> {
    let raw = fs::read_to_string(manifest_path)
        .with_context(|| format!("read manifest {}", manifest_path.display()))?;
    let manifest: Manifest = serde_json::from_str(&raw)
        .with_context(|| format!("parse manifest {}", manifest_path.display()))?;

    if manifest.schema != MANIFEST_SCHEMA {
        bail!(
            "unsupported manifest schema `{}` (expected `{MANIFEST_SCHEMA}`)",
            manifest.schema
        );
    }

    let mut case_reports = Vec::with_capacity(manifest.cases.len());
    let mut failures = Vec::new();

    for case in &manifest.cases {
        let actual_json = run_json(&case.mode, &case.args);
        let actual: Value = serde_json::from_str(&actual_json).with_context(|| {
            format!(
                "case `{}`: run_json({:?}, ...) did not return valid JSON",
                case.id, case.mode
            )
        })?;

        let actual_ok = actual.get("ok").and_then(Value::as_bool).unwrap_or(false);

        if actual_ok != case.expect_ok {
            failures.push(format!(
                "case `{}`: expected ok={}, got ok={} (payload: {actual_json})",
                case.id, case.expect_ok, actual_ok
            ));
        }

        if case.expect_ok {
            if let Err(err) = check_success_data(&case.id, &actual, case) {
                failures.push(err);
            }
        } else if let Err(err) = check_error_envelope(&case.id, &actual, case) {
            failures.push(err);
        }

        if let Some(golden_rel) = &case.golden {
            let golden_path = repo_root.join("fixtures/bindings-parity").join(golden_rel);
            if let Err(err) = compare_golden(&case.id, &actual, &golden_path) {
                failures.push(err);
            }
        }

        case_reports.push(CaseReport {
            id: case.id.clone(),
            mode: case.mode.clone(),
            expect_ok: case.expect_ok,
            actual_ok,
        });
    }

    if !failures.is_empty() {
        for failure in &failures {
            eprintln!("::error::bindings-parity: {failure}");
        }
        bail!(
            "bindings parity failed for {} case(s):\n  {}",
            failures.len(),
            failures.join("\n  ")
        );
    }

    Ok(ParityReport {
        schema: "tokmd.bindings_parity_report.v1",
        ok: true,
        manifest: path_display(repo_root, manifest_path),
        agent_charter: manifest.agent_charter.clone(),
        cases: case_reports,
    })
}

fn check_success_data(case_id: &str, actual: &Value, case: &Case) -> Result<(), String> {
    let data = actual
        .get("data")
        .and_then(Value::as_object)
        .ok_or_else(|| format!("case `{case_id}`: success envelope missing data object"))?;

    for key in &case.data_has_keys {
        if !data.contains_key(key) {
            return Err(format!(
                "case `{case_id}`: data missing required key `{key}`"
            ));
        }
    }

    for (key, expected) in &case.data_contains {
        let Some(actual_value) = data.get(key) else {
            return Err(format!("case `{case_id}`: data missing key `{key}`"));
        };
        if actual_value != expected {
            return Err(format!(
                "case `{case_id}`: data[{key}] expected {expected}, got {actual_value}"
            ));
        }
    }

    Ok(())
}

fn check_error_envelope(case_id: &str, actual: &Value, case: &Case) -> Result<(), String> {
    let error = actual
        .get("error")
        .and_then(Value::as_object)
        .ok_or_else(|| format!("case `{case_id}`: error envelope missing error object"))?;

    for (key, expected) in &case.error_contains {
        let Some(actual_value) = error.get(key) else {
            return Err(format!("case `{case_id}`: error missing key `{key}`"));
        };
        if actual_value != expected {
            return Err(format!(
                "case `{case_id}`: error[{key}] expected {expected}, got {actual_value}"
            ));
        }
    }

    Ok(())
}

fn compare_golden(case_id: &str, actual: &Value, golden_path: &Path) -> Result<(), String> {
    let golden_raw = fs::read_to_string(golden_path).map_err(|err| {
        format!(
            "case `{case_id}`: read golden {}: {err}",
            golden_path.display()
        )
    })?;
    let golden: Value = serde_json::from_str(&golden_raw).map_err(|err| {
        format!(
            "case `{case_id}`: parse golden {}: {err}",
            golden_path.display()
        )
    })?;

    if !value_contains(actual, &golden) {
        return Err(format!(
            "case `{case_id}`: envelope does not match golden {}",
            golden_path.display()
        ));
    }

    Ok(())
}

fn value_contains(actual: &Value, expected: &Value) -> bool {
    match expected {
        Value::Object(expected_map) => {
            let Some(actual_map) = actual.as_object() else {
                return false;
            };
            expected_map.iter().all(|(key, expected_value)| {
                actual_map
                    .get(key)
                    .is_some_and(|actual_value| value_contains(actual_value, expected_value))
            })
        }
        Value::Array(expected_items) => {
            let Some(actual_items) = actual.as_array() else {
                return false;
            };
            expected_items.len() == actual_items.len()
                && expected_items
                    .iter()
                    .zip(actual_items.iter())
                    .all(|(expected_item, actual_item)| value_contains(actual_item, expected_item))
        }
        _ => actual == expected,
    }
}

fn run_cargo_tests(repo_root: &Path) -> Result<()> {
    let steps = [
        (
            "tokmd-core bindings_parity integration test",
            vec![
                "test".to_string(),
                "-p".to_string(),
                "tokmd-core".to_string(),
                "--test".to_string(),
                "bindings_parity".to_string(),
                "--all-features".to_string(),
            ],
        ),
        (
            "tokmd-node binding unit tests",
            vec![
                "test".to_string(),
                "-p".to_string(),
                "tokmd-node".to_string(),
                "--lib".to_string(),
                "--all-features".to_string(),
            ],
        ),
        (
            "tokmd-python property tests",
            vec![
                "test".to_string(),
                "-p".to_string(),
                "tokmd-python".to_string(),
                "--test".to_string(),
                "property_tests".to_string(),
                "--all-features".to_string(),
            ],
        ),
    ];

    for (label, cargo_args) in steps {
        println!("Running {label}...");
        let status = Command::new("cargo")
            .current_dir(repo_root)
            .args(&cargo_args)
            .status()
            .with_context(|| format!("spawn cargo for {label}"))?;

        if !status.success() {
            bail!("{label} failed (exit {status})");
        }
    }

    Ok(())
}

fn print_fixture_report(report: &ParityReport) {
    println!("Bindings parity fixture manifest: {}", report.manifest);
    if let Some(charter) = &report.agent_charter {
        println!("Agent charter: {charter}");
    }
    println!(
        "Core version: {} (schema_version={})",
        version(),
        schema_version()
    );
    for case in &report.cases {
        println!("  ✓ {} mode={} ok={}", case.id, case.mode, case.actual_ok);
    }
}

fn write_receipt(path: &Path, report: &ParityReport) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("create receipt parent {}", parent.display()))?;
    }
    let json = serde_json::to_string_pretty(report).context("serialize bindings parity receipt")?;
    fs::write(path, format!("{json}\n"))
        .with_context(|| format!("write receipt {}", path.display()))?;
    println!("Wrote bindings parity receipt to {}", path.display());
    Ok(())
}

fn find_repo_root() -> Result<PathBuf> {
    let mut dir = std::env::current_dir()?;
    loop {
        let cargo_toml = dir.join("Cargo.toml");
        if cargo_toml.exists() {
            let content = fs::read_to_string(&cargo_toml)
                .with_context(|| format!("read {}", cargo_toml.display()))?;
            if content.contains("[workspace]") {
                return Ok(dir);
            }
        }
        if !dir.pop() {
            bail!("could not find workspace root");
        }
    }
}

fn path_display(repo_root: &Path, path: &Path) -> String {
    path.strip_prefix(repo_root)
        .unwrap_or(path)
        .display()
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn value_contains_supports_partial_object_matching() {
        let actual = serde_json::json!({
            "ok": false,
            "error": {
                "code": "invalid_json",
                "message": "Invalid JSON: trailing characters at line 1 column 5"
            }
        });
        let expected = serde_json::json!({
            "ok": false,
            "error": { "code": "invalid_json" }
        });
        assert!(value_contains(&actual, &expected));
    }

    #[test]
    fn manifest_parses_default_fixture() {
        let repo_root = find_repo_root().expect("repo root");
        let manifest_path = repo_root.join(DEFAULT_MANIFEST);
        let raw = fs::read_to_string(&manifest_path).expect("read manifest");
        let manifest: Manifest = serde_json::from_str(&raw).expect("parse manifest");
        assert_eq!(manifest.schema, MANIFEST_SCHEMA);
        assert!(!manifest.cases.is_empty());
    }
}
