use crate::cli::ProofArtifactsCheckArgs;
use anyhow::{Context, Result, bail};
use serde_json::Value;
use std::fs;
use std::path::Path;

const SUMMARY_SCHEMA: &str = "tokmd.proof_executor_summary.v1";
const MANIFEST_SCHEMA: &str = "tokmd.proof_executor_manifest.v1";

const SHARED_FIELDS: &[&str] = &[
    "mode",
    "status",
    "execution_status",
    "execution_guard",
    "family",
    "required",
    "profile",
    "base",
    "head",
    "ok",
    "changed_files",
    "unknown_files",
];

const ENTRY_FIELDS: &[&str] = &[
    "scope",
    "kind",
    "required",
    "command",
    "artifact_path",
    "status",
    "skip_reason",
    "exit_code",
];

pub fn run(args: ProofArtifactsCheckArgs) -> Result<()> {
    let summary = read_json(&args.executor_summary, "executor summary")?;
    let manifest = read_json(&args.executor_manifest, "executor manifest")?;

    let report = validate_executor_artifacts(&summary, &manifest, VerificationMode::NoExecution)?;
    println!(
        "Proof artifacts OK: {} command(s), execution_status {}, guard {}",
        report.selected, report.execution_status, report.guard_reason
    );
    Ok(())
}

pub fn run_execution(args: ProofArtifactsCheckArgs) -> Result<()> {
    let summary = read_json(&args.executor_summary, "executor summary")?;
    let manifest = read_json(&args.executor_manifest, "executor manifest")?;

    let report = validate_executor_artifacts(&summary, &manifest, VerificationMode::Execution)?;
    println!(
        "Proof execution artifacts OK: {} executed command(s), guard {}",
        report.executed, report.guard_reason
    );
    Ok(())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum VerificationMode {
    NoExecution,
    Execution,
}

#[derive(Debug, PartialEq, Eq)]
struct ProofArtifactsReport {
    selected: usize,
    executed: usize,
    execution_status: String,
    guard_reason: String,
}

fn read_json(path: &Path, label: &str) -> Result<Value> {
    let raw = fs::read_to_string(path)
        .with_context(|| format!("failed to read {label} artifact `{}`", path.display()))?;
    serde_json::from_str(&raw)
        .with_context(|| format!("failed to parse {label} artifact `{}`", path.display()))
}

fn validate_executor_artifacts(
    summary: &Value,
    manifest: &Value,
    mode: VerificationMode,
) -> Result<ProofArtifactsReport> {
    expect_schema(summary, SUMMARY_SCHEMA, "executor summary")?;
    expect_schema(manifest, MANIFEST_SCHEMA, "executor manifest")?;

    for field in SHARED_FIELDS {
        expect_equal(summary, manifest, field)?;
    }

    let execution_status = expect_string(
        field(summary, "execution_status", "executor summary")?,
        "execution_status",
        "executor summary",
    )?;
    let guard_enabled = expect_bool(
        field(summary, "execution_guard.enabled", "executor summary")?,
        "execution_guard.enabled",
        "executor summary",
    )?;

    let summary_selected = expect_usize(
        field(summary, "counts.selected", "executor summary")?,
        "counts.selected",
        "executor summary",
    )?;
    let summary_executed = expect_usize(
        field(summary, "counts.executed", "executor summary")?,
        "counts.executed",
        "executor summary",
    )?;
    let manifest_selected = expect_usize(
        field(manifest, "selection.selected", "executor manifest")?,
        "selection.selected",
        "executor manifest",
    )?;
    let manifest_executed = expect_usize(
        field(manifest, "selection.executed", "executor manifest")?,
        "selection.executed",
        "executor manifest",
    )?;

    if summary_selected != manifest_selected {
        bail!(
            "executor artifact mismatch at selected count: summary {summary_selected} != manifest {manifest_selected}"
        );
    }
    if summary_executed != manifest_executed {
        bail!(
            "executor artifact mismatch at executed count: summary {summary_executed} != manifest {manifest_executed}"
        );
    }

    expect_string_value(
        field(manifest, "selection.source", "executor manifest")?,
        "proof_plan",
        "selection.source",
        "executor manifest",
    )?;
    expect_bool_value(
        field(manifest, "selection.required_included", "executor manifest")?,
        false,
        "selection.required_included",
        "executor manifest",
    )?;

    let entries = expect_array(
        field(summary, "entries", "executor summary")?,
        "entries",
        "executor summary",
    )?;
    let commands = expect_array(
        field(manifest, "commands", "executor manifest")?,
        "commands",
        "executor manifest",
    )?;
    if entries.len() != summary_selected {
        bail!(
            "executor summary entries length {} does not match selected count {summary_selected}",
            entries.len()
        );
    }
    if commands.len() != manifest_selected {
        bail!(
            "executor manifest commands length {} does not match selected count {manifest_selected}",
            commands.len()
        );
    }

    for (index, (entry, command)) in entries.iter().zip(commands.iter()).enumerate() {
        validate_command_entry(index, entry, command)?;
    }

    validate_execution_state(
        summary,
        entries,
        mode,
        &execution_status,
        guard_enabled,
        summary_selected,
        summary_executed,
    )?;

    let guard_reason = expect_string(
        field(summary, "execution_guard.reason", "executor summary")?,
        "execution_guard.reason",
        "executor summary",
    )?;

    Ok(ProofArtifactsReport {
        selected: summary_selected,
        executed: summary_executed,
        execution_status,
        guard_reason,
    })
}

fn validate_execution_state(
    summary: &Value,
    entries: &[Value],
    mode: VerificationMode,
    execution_status: &str,
    guard_enabled: bool,
    selected: usize,
    executed: usize,
) -> Result<()> {
    match mode {
        VerificationMode::NoExecution => {
            if execution_status == "executed" {
                bail!(
                    "executor artifacts report executed commands; use proof-execution-artifacts-check for executed artifacts"
                );
            }
            if executed != 0 {
                bail!(
                    "executor artifacts report {executed} executed command(s); no-execution verifier requires zero"
                );
            }
        }
        VerificationMode::Execution => {
            expect_string_value(
                field(summary, "mode", "executor summary")?,
                "execute",
                "mode",
                "executor summary",
            )?;
            if execution_status != "executed" {
                bail!(
                    "executor artifacts have execution_status `{execution_status}`; execution verifier requires `executed`"
                );
            }
            if !guard_enabled {
                bail!(
                    "executor artifacts have execution_guard.enabled=false; execution verifier requires explicit opt-in"
                );
            }

            let failed = expect_usize(
                field(summary, "counts.failed", "executor summary")?,
                "counts.failed",
                "executor summary",
            )?;
            if failed != 0 {
                bail!(
                    "executor artifacts report {failed} failed command(s); execution verifier requires zero"
                );
            }

            let skipped = expect_usize(
                field(summary, "counts.skipped", "executor summary")?,
                "counts.skipped",
                "executor summary",
            )?;
            let dry_run = expect_usize(
                field(summary, "counts.dry_run", "executor summary")?,
                "counts.dry_run",
                "executor summary",
            )?;
            let passed = expect_usize(
                field(summary, "counts.passed", "executor summary")?,
                "counts.passed",
                "executor summary",
            )?;
            if skipped != 0 || dry_run != 0 {
                bail!(
                    "executor artifacts report skipped={skipped} dry_run={dry_run}; execution verifier requires executed commands only"
                );
            }
            if executed != selected {
                bail!(
                    "executor artifacts report {executed} executed command(s) for {selected} selected command(s)"
                );
            }
            if passed != selected {
                bail!(
                    "executor artifacts report {passed} passed command(s) for {selected} selected command(s)"
                );
            }
            expect_string_value(
                field(summary, "status", "executor summary")?,
                "passed",
                "status",
                "executor summary",
            )?;

            for (index, entry) in entries.iter().enumerate() {
                validate_executed_entry(index, entry)?;
                validate_executed_artifact_path(index, entry)?;
            }
        }
    }
    Ok(())
}

fn validate_executed_entry(index: usize, entry: &Value) -> Result<()> {
    let expected_index = index + 1;
    expect_string_value(
        field(entry, "status", "executor summary entry")?,
        "passed",
        "status",
        "executor summary entry",
    )
    .with_context(|| format!("executor summary entry {expected_index} failed status check"))?;
    expect_string_value(
        field(entry, "skip_reason", "executor summary entry")?,
        "",
        "skip_reason",
        "executor summary entry",
    )
    .with_context(|| format!("executor summary entry {expected_index} failed skip check"))?;

    let exit_code = field(entry, "exit_code", "executor summary entry")?;
    if exit_code.as_i64() != Some(0) {
        bail!(
            "executor summary entry {expected_index} exit_code must be 0 for passed execution, got {}",
            render_json(exit_code)
        );
    }
    Ok(())
}

fn validate_executed_artifact_path(index: usize, entry: &Value) -> Result<()> {
    let expected_index = index + 1;
    let kind = expect_string(
        field(entry, "kind", "executor summary entry")?,
        "kind",
        "executor summary entry",
    )?;
    let artifact_path = field(entry, "artifact_path", "executor summary entry")?;
    if artifact_path.is_null() {
        return Ok(());
    }

    let artifact_path = expect_string(artifact_path, "artifact_path", "executor summary entry")?;
    if artifact_path.trim().is_empty() {
        bail!("executor summary entry {expected_index} artifact_path must not be empty");
    }

    let metadata = fs::metadata(&artifact_path).with_context(|| {
        format!("executor summary entry {expected_index} artifact `{artifact_path}` was not found")
    })?;
    if !metadata.is_file() {
        bail!("executor summary entry {expected_index} artifact `{artifact_path}` is not a file");
    }
    if metadata.len() == 0 {
        bail!("executor summary entry {expected_index} artifact `{artifact_path}` is empty");
    }

    if kind == "coverage" {
        validate_lcov_artifact(expected_index, &artifact_path)?;
    }

    Ok(())
}

fn validate_lcov_artifact(index: usize, artifact_path: &str) -> Result<()> {
    let raw = fs::read_to_string(artifact_path).with_context(|| {
        format!(
            "executor summary entry {index} LCOV artifact `{artifact_path}` is not readable text"
        )
    })?;

    if !raw.lines().any(|line| line.starts_with("SF:")) {
        bail!("executor summary entry {index} LCOV artifact `{artifact_path}` has no `SF:` record");
    }
    if !raw.lines().any(|line| line == "end_of_record") {
        bail!(
            "executor summary entry {index} LCOV artifact `{artifact_path}` has no `end_of_record`"
        );
    }

    Ok(())
}

fn validate_command_entry(index: usize, entry: &Value, command: &Value) -> Result<()> {
    let expected_index = index + 1;
    let manifest_index = expect_usize(
        field(command, "index", "executor manifest command")?,
        "index",
        "executor manifest command",
    )?;
    if manifest_index != expected_index {
        bail!(
            "executor manifest command index mismatch at position {expected_index}: got {manifest_index}"
        );
    }

    let id = expect_string(
        field(command, "id", "executor manifest command")?,
        "id",
        "executor manifest command",
    )?;
    let expected_prefix = format!("{expected_index:04}-");
    if !id.starts_with(&expected_prefix) {
        bail!("executor manifest command id `{id}` does not start with `{expected_prefix}`");
    }

    for field_name in ENTRY_FIELDS {
        let entry_value = field(entry, field_name, "executor summary entry")?;
        let command_value = field(command, field_name, "executor manifest command")?;
        if entry_value != command_value {
            bail!(
                "executor command mismatch at `{field_name}` for command {expected_index}: summary {} != manifest {}",
                render_json(entry_value),
                render_json(command_value)
            );
        }
    }
    Ok(())
}

fn expect_schema(value: &Value, expected: &str, label: &str) -> Result<()> {
    expect_string_value(field(value, "schema", label)?, expected, "schema", label)
}

fn expect_equal(summary: &Value, manifest: &Value, path: &str) -> Result<()> {
    let summary_value = field(summary, path, "executor summary")?;
    let manifest_value = field(manifest, path, "executor manifest")?;
    if summary_value != manifest_value {
        bail!(
            "executor artifact mismatch at `{path}`: summary {} != manifest {}",
            render_json(summary_value),
            render_json(manifest_value)
        );
    }
    Ok(())
}

fn field<'a>(value: &'a Value, path: &str, label: &str) -> Result<&'a Value> {
    let mut current = value;
    for segment in path.split('.') {
        current = current
            .get(segment)
            .with_context(|| format!("{label} artifact is missing `{path}`"))?;
    }
    Ok(current)
}

fn expect_array<'a>(value: &'a Value, path: &str, label: &str) -> Result<&'a Vec<Value>> {
    value
        .as_array()
        .with_context(|| format!("{label} `{path}` must be an array"))
}

fn expect_bool(value: &Value, path: &str, label: &str) -> Result<bool> {
    value
        .as_bool()
        .with_context(|| format!("{label} `{path}` must be a boolean"))
}

fn expect_bool_value(value: &Value, expected: bool, path: &str, label: &str) -> Result<()> {
    let actual = expect_bool(value, path, label)?;
    if actual != expected {
        bail!("{label} `{path}` must be {expected}, got {actual}");
    }
    Ok(())
}

fn expect_string(value: &Value, path: &str, label: &str) -> Result<String> {
    value
        .as_str()
        .map(ToOwned::to_owned)
        .with_context(|| format!("{label} `{path}` must be a string"))
}

fn expect_string_value(value: &Value, expected: &str, path: &str, label: &str) -> Result<()> {
    let actual = expect_string(value, path, label)?;
    if actual != expected {
        bail!("{label} `{path}` must be `{expected}`, got `{actual}`");
    }
    Ok(())
}

fn expect_usize(value: &Value, path: &str, label: &str) -> Result<usize> {
    let number = value
        .as_u64()
        .with_context(|| format!("{label} `{path}` must be a non-negative integer"))?;
    usize::try_from(number).with_context(|| format!("{label} `{path}` is too large"))
}

fn render_json(value: &Value) -> String {
    serde_json::to_string(value).unwrap_or_else(|_| "<unrenderable>".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn accepts_matching_no_execution_artifacts() {
        let (summary, manifest) = matching_artifacts();

        let report =
            validate_executor_artifacts(&summary, &manifest, VerificationMode::NoExecution)
                .unwrap();

        assert_eq!(
            report,
            ProofArtifactsReport {
                selected: 1,
                executed: 0,
                execution_status: "dry_run".to_string(),
                guard_reason: "local_requires_--allow-local-evidence-execution".to_string(),
            }
        );
    }

    #[test]
    fn rejects_selected_count_drift() {
        let (summary, mut manifest) = matching_artifacts();
        manifest["selection"]["selected"] = json!(2);

        let error = validate_executor_artifacts(&summary, &manifest, VerificationMode::NoExecution)
            .unwrap_err()
            .to_string();

        assert!(error.contains("selected count"));
    }

    #[test]
    fn rejects_command_payload_drift() {
        let (summary, mut manifest) = matching_artifacts();
        manifest["commands"][0]["command"] = json!("cargo llvm-cov -p tokmd-gate");

        let error = validate_executor_artifacts(&summary, &manifest, VerificationMode::NoExecution)
            .unwrap_err()
            .to_string();

        assert!(error.contains("executor command mismatch"));
    }

    #[test]
    fn accepts_enabled_execution_guard_when_no_commands_executed() {
        let (mut summary, mut manifest) = matching_artifacts();
        summary["execution_guard"]["enabled"] = json!(true);
        manifest["execution_guard"]["enabled"] = json!(true);
        summary["execution_guard"]["ci"] = json!(true);
        manifest["execution_guard"]["ci"] = json!(true);
        summary["execution_guard"]["allow_ci_evidence_execution"] = json!(true);
        manifest["execution_guard"]["allow_ci_evidence_execution"] = json!(true);
        summary["execution_guard"]["reason"] = json!("ci_explicit_opt_in_enabled");
        manifest["execution_guard"]["reason"] = json!("ci_explicit_opt_in_enabled");

        let report =
            validate_executor_artifacts(&summary, &manifest, VerificationMode::NoExecution)
                .unwrap();

        assert_eq!(
            report,
            ProofArtifactsReport {
                selected: 1,
                executed: 0,
                execution_status: "dry_run".to_string(),
                guard_reason: "ci_explicit_opt_in_enabled".to_string(),
            }
        );
    }

    #[test]
    fn rejects_executed_artifacts_even_when_guard_enabled() {
        let (mut summary, mut manifest) = matching_artifacts();
        summary["execution_status"] = json!("executed");
        manifest["execution_status"] = json!("executed");
        summary["execution_guard"]["enabled"] = json!(true);
        manifest["execution_guard"]["enabled"] = json!(true);
        summary["counts"]["executed"] = json!(1);
        manifest["selection"]["executed"] = json!(1);

        let error = validate_executor_artifacts(&summary, &manifest, VerificationMode::NoExecution)
            .unwrap_err()
            .to_string();

        assert!(error.contains("executed commands"));
    }

    #[test]
    fn accepts_matching_executed_artifacts() {
        let (summary, manifest) = executed_artifacts();

        let report =
            validate_executor_artifacts(&summary, &manifest, VerificationMode::Execution).unwrap();

        assert_eq!(
            report,
            ProofArtifactsReport {
                selected: 1,
                executed: 1,
                execution_status: "executed".to_string(),
                guard_reason: "local_explicit_opt_in_enabled".to_string(),
            }
        );
    }

    #[test]
    fn rejects_execution_artifacts_without_enabled_guard() {
        let (mut summary, mut manifest) = executed_artifacts();
        summary["execution_guard"]["enabled"] = json!(false);
        manifest["execution_guard"]["enabled"] = json!(false);

        let error = validate_executor_artifacts(&summary, &manifest, VerificationMode::Execution)
            .unwrap_err()
            .to_string();

        assert!(error.contains("execution_guard.enabled=false"));
    }

    #[test]
    fn rejects_execution_artifacts_with_failed_commands() {
        let (mut summary, mut manifest) = executed_artifacts();
        summary["status"] = json!("failed");
        manifest["status"] = json!("failed");
        summary["counts"]["passed"] = json!(0);
        summary["counts"]["failed"] = json!(1);
        summary["entries"][0]["status"] = json!("failed");
        manifest["commands"][0]["status"] = json!("failed");
        summary["entries"][0]["skip_reason"] = json!("command_failed");
        manifest["commands"][0]["skip_reason"] = json!("command_failed");
        summary["entries"][0]["exit_code"] = json!(1);
        manifest["commands"][0]["exit_code"] = json!(1);

        let error = validate_executor_artifacts(&summary, &manifest, VerificationMode::Execution)
            .unwrap_err()
            .to_string();

        assert!(error.contains("failed command"));
    }

    #[test]
    fn rejects_execution_artifacts_with_missing_output_file() {
        let (mut summary, mut manifest) = executed_artifacts();
        let missing = std::env::temp_dir()
            .join(format!(
                "tokmd-missing-proof-artifact-{}.lcov",
                std::process::id()
            ))
            .to_string_lossy()
            .to_string();
        let _ = fs::remove_file(&missing);
        summary["entries"][0]["artifact_path"] = json!(missing);
        manifest["commands"][0]["artifact_path"] = json!(missing);

        let error = validate_executor_artifacts(&summary, &manifest, VerificationMode::Execution)
            .unwrap_err()
            .to_string();

        assert!(error.contains("was not found"));
    }

    #[test]
    fn rejects_execution_artifacts_with_malformed_lcov_output() {
        let (mut summary, mut manifest) = executed_artifacts();
        let malformed = write_test_artifact(
            "tokmd-malformed-proof-artifact",
            "this is not an LCOV payload\n",
        );
        summary["entries"][0]["artifact_path"] = json!(malformed);
        manifest["commands"][0]["artifact_path"] = json!(malformed);

        let error = validate_executor_artifacts(&summary, &manifest, VerificationMode::Execution)
            .unwrap_err()
            .to_string();

        assert!(error.contains("SF:"));
    }

    #[test]
    fn rejects_dry_run_artifacts_with_execution_verifier() {
        let (summary, manifest) = matching_artifacts();

        let error = validate_executor_artifacts(&summary, &manifest, VerificationMode::Execution)
            .unwrap_err()
            .to_string();

        assert!(error.contains("`mode` must be `execute`"));
    }

    fn executed_artifacts() -> (Value, Value) {
        let (mut summary, mut manifest) = matching_artifacts();
        let artifact_path = write_test_lcov_artifact();
        summary["mode"] = json!("execute");
        manifest["mode"] = json!("execute");
        summary["status"] = json!("passed");
        manifest["status"] = json!("passed");
        summary["execution_status"] = json!("executed");
        manifest["execution_status"] = json!("executed");
        summary["execution_guard"]["enabled"] = json!(true);
        manifest["execution_guard"]["enabled"] = json!(true);
        summary["execution_guard"]["allow_local_evidence_execution"] = json!(true);
        manifest["execution_guard"]["allow_local_evidence_execution"] = json!(true);
        summary["execution_guard"]["reason"] = json!("local_explicit_opt_in_enabled");
        manifest["execution_guard"]["reason"] = json!("local_explicit_opt_in_enabled");
        summary["counts"]["dry_run"] = json!(0);
        summary["counts"]["executed"] = json!(1);
        summary["counts"]["passed"] = json!(1);
        summary["counts"]["failed"] = json!(0);
        manifest["selection"]["executed"] = json!(1);
        summary["entries"][0]["status"] = json!("passed");
        manifest["commands"][0]["status"] = json!("passed");
        summary["entries"][0]["skip_reason"] = json!("");
        manifest["commands"][0]["skip_reason"] = json!("");
        summary["entries"][0]["exit_code"] = json!(0);
        manifest["commands"][0]["exit_code"] = json!(0);
        summary["entries"][0]["artifact_path"] = json!(artifact_path);
        manifest["commands"][0]["artifact_path"] = json!(artifact_path);
        (summary, manifest)
    }

    fn write_test_lcov_artifact() -> String {
        write_test_artifact(
            "tokmd-proof-artifact",
            "TN:\nSF:crates/tokmd-core/src/ffi.rs\nend_of_record\n",
        )
    }

    fn write_test_artifact(name: &str, content: &str) -> String {
        let path = std::env::temp_dir().join(format!("{name}-{}.lcov", std::process::id()));
        fs::write(&path, content).expect("test artifact should be writable");
        path.to_string_lossy().to_string()
    }

    fn matching_artifacts() -> (Value, Value) {
        let guard = json!({
            "required": true,
            "enabled": false,
            "ci": false,
            "ci_execution": "explicit_opt_in",
            "allow_ci_evidence_execution": false,
            "reason": "local_requires_--allow-local-evidence-execution",
            "allow_local_evidence_execution": false
        });
        let entry = json!({
            "scope": "tokmd_core_ffi",
            "kind": "coverage",
            "required": false,
            "command": "cargo llvm-cov -p tokmd-core --lcov --output-path target/proof/coverage/tokmd_core_ffi.lcov",
            "artifact_path": "target/proof/coverage/tokmd_core_ffi.lcov",
            "status": "dry_run",
            "skip_reason": "dry_run_only",
            "exit_code": null
        });
        let summary = json!({
            "schema": SUMMARY_SCHEMA,
            "mode": "dry_run",
            "status": "dry_run",
            "execution_status": "dry_run",
            "execution_guard": guard.clone(),
            "family": "coverage",
            "required": false,
            "profile": "affected",
            "base": "origin/main",
            "head": "HEAD",
            "ok": true,
            "changed_files": ["crates/tokmd-core/src/ffi.rs"],
            "counts": {
                "commands_total": 2,
                "family_planned": 1,
                "selected": 1,
                "skipped": 0,
                "dry_run": 1,
                "executed": 0,
                "required_excluded": 0,
                "selection_excluded": 0,
                "non_family_excluded": 1
            },
            "entries": [entry.clone()],
            "unknown_files": []
        });
        let manifest = json!({
            "schema": MANIFEST_SCHEMA,
            "mode": "dry_run",
            "status": "dry_run",
            "execution_status": "dry_run",
            "execution_guard": guard,
            "family": "coverage",
            "required": false,
            "profile": "affected",
            "base": "origin/main",
            "head": "HEAD",
            "ok": true,
            "changed_files": ["crates/tokmd-core/src/ffi.rs"],
            "selection": {
                "source": "proof_plan",
                "max_dry_run_commands": 1,
                "required_included": false,
                "selected": 1,
                "executed": 0
            },
            "commands": [{
                "id": "0001-tokmd_core_ffi-coverage",
                "index": 1,
                "scope": "tokmd_core_ffi",
                "kind": "coverage",
                "required": false,
                "command": "cargo llvm-cov -p tokmd-core --lcov --output-path target/proof/coverage/tokmd_core_ffi.lcov",
                "artifact_path": "target/proof/coverage/tokmd_core_ffi.lcov",
                "status": "dry_run",
                "skip_reason": "dry_run_only",
                "exit_code": null
            }],
            "unknown_files": []
        });
        (summary, manifest)
    }
}
