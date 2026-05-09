//! Shared proof-evidence JSON fixtures for owner-module tests.

use super::artifacts::{
    COVERAGE_RECEIPT_SCHEMA, PROOF_EXECUTOR_OBSERVATION_SCHEMA, PROOF_RUN_OBSERVATION_SCHEMA,
    PROOF_RUN_SUMMARY_SCHEMA, ProofEvidenceArtifact, parse_proof_evidence_json,
};

fn parse_value(value: serde_json::Value) -> ProofEvidenceArtifact {
    parse_proof_evidence_json(&value.to_string()).expect("parse proof evidence")
}

pub(super) fn proof_run_summary_artifact(head: &str) -> ProofEvidenceArtifact {
    parse_value(serde_json::json!({
        "schema": PROOF_RUN_SUMMARY_SCHEMA,
        "status": "passed",
        "execution_status": "executed",
        "execution_guard": {
            "required": true,
            "enabled": true,
            "ci": true,
            "allow_ci_required_execution": true,
            "allow_local_required_execution": false,
            "reason": "ci_required_execution_opted_in"
        },
        "profile": "fast",
        "base": "origin/main",
        "head": head,
        "ok": true,
        "changed_files": ["crates/tokmd-cockpit/src/lib.rs"],
        "counts": {
            "commands_total": 1,
            "required_planned": 1,
            "advisory_skipped": 0,
            "executed": 1,
            "passed": 1,
            "failed": 0
        },
        "entries": [
            {
                "scope": "tokmd_cockpit",
                "kind": "test",
                "command": "cargo test -p tokmd-cockpit",
                "required": true,
                "advisory": false,
                "artifact_path": null,
                "status": "passed",
                "skip_reason": "",
                "exit_code": 0
            }
        ],
        "unknown_files": []
    }))
}

pub(super) fn proof_run_observation_artifact(head: &str) -> ProofEvidenceArtifact {
    parse_value(serde_json::json!({
        "schema": PROOF_RUN_OBSERVATION_SCHEMA,
        "status": "passed",
        "execution_status": "executed",
        "profile": "fast",
        "base": "origin/main",
        "head": head,
        "ok": true,
        "execution_guard": {
            "enabled": true,
            "ci": true,
            "reason": "required proof-run summary verified"
        },
        "counts": {
            "commands_total": 1,
            "required_planned": 1,
            "advisory_skipped": 0,
            "executed": 1,
            "passed": 1,
            "failed": 0
        },
        "scopes": [
            {
                "name": "tokmd_cockpit",
                "kind": "test",
                "command": "cargo test -p tokmd-cockpit",
                "status": "passed",
                "exit_code": 0
            }
        ],
        "changed_files": ["crates/tokmd-cockpit/src/lib.rs"],
        "unknown_files": []
    }))
}

pub(super) fn proof_executor_observation_artifact(head: &str) -> ProofEvidenceArtifact {
    parse_value(serde_json::json!({
        "schema": PROOF_EXECUTOR_OBSERVATION_SCHEMA,
        "status": "dry_run",
        "execution_status": "dry_run",
        "profile": "affected",
        "base": "origin/main",
        "head": head,
        "family": "coverage",
        "required": false,
        "ok": true,
        "execution_guard": {
            "enabled": true,
            "ci": true,
            "reason": "advisory_executor_enabled"
        },
        "counts": {
            "selected": 1,
            "executed": 0,
            "passed": 0,
            "failed": 0,
            "artifacts": 1
        },
        "scopes": [
            {
                "name": "tokmd_cockpit",
                "kind": "coverage",
                "command": "cargo llvm-cov -p tokmd-cockpit",
                "artifact_path": "target/proof/coverage/tokmd-cockpit.lcov",
                "status": "dry_run",
                "exit_code": null
            }
        ],
        "changed_files": ["crates/tokmd-cockpit/src/render/review_packet.rs"],
        "unknown_files": []
    }))
}

pub(super) fn coverage_receipt_artifact(
    sha: &str,
    ok: bool,
    non_empty: bool,
) -> ProofEvidenceArtifact {
    parse_value(serde_json::json!({
        "schema": COVERAGE_RECEIPT_SCHEMA,
        "schema_version": 1,
        "repo": "EffortlessMetrics/tokmd",
        "lane": "scoped",
        "flag": "tokmd_cockpit",
        "workflow": "Coverage",
        "sha": sha,
        "github": {
            "run_id": "12345",
            "run_attempt": "1",
            "event_name": "pull_request",
            "ref_name": "feature"
        },
        "artifacts": [
            {
                "path": "target/proof/coverage/tokmd-cockpit.lcov",
                "kind": "lcov",
                "bytes": if non_empty { 42 } else { 0 },
                "non_empty": non_empty
            }
        ],
        "status": {
            "ok": ok,
            "missing": [],
            "empty": if non_empty {
                Vec::<String>::new()
            } else {
                vec!["target/proof/coverage/tokmd-cockpit.lcov".to_string()]
            }
        }
    }))
}
