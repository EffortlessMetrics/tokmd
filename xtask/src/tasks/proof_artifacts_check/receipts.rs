use crate::cli::{ProofArtifactsCheckArgs, ProofRunArtifactsCheckArgs};
use anyhow::Result;
use serde::Serialize;
use std::collections::BTreeMap;
use std::path::Path;

use super::io::write_text;
use super::schemas::PROOF_RUN_ARTIFACTS_CHECK_SCHEMA;
use super::types::{ProofArtifactsReport, ProofRunArtifactsReport};

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub(super) struct ProofArtifactsCheckReceipt {
    pub(super) schema: String,
    pub(super) ok: bool,
    pub(super) verifier: String,
    pub(super) inputs: BTreeMap<String, String>,
    pub(super) counts: ProofArtifactsCheckCounts,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) execution_status: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) guard_reason: Option<String>,
    pub(super) errors: Vec<String>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub(super) struct ProofArtifactsCheckCounts {
    pub(super) selected: usize,
    pub(super) executed: usize,
}

pub(super) fn executor_check_receipt(
    schema: &str,
    verifier: &str,
    args: &ProofArtifactsCheckArgs,
    outcome: &std::result::Result<ProofArtifactsReport, anyhow::Error>,
) -> ProofArtifactsCheckReceipt {
    let mut inputs = BTreeMap::new();
    inputs.insert(
        "executor_summary".to_string(),
        args.executor_summary.to_string_lossy().to_string(),
    );
    inputs.insert(
        "executor_manifest".to_string(),
        args.executor_manifest.to_string_lossy().to_string(),
    );

    match outcome {
        Ok(report) => ProofArtifactsCheckReceipt {
            schema: schema.to_string(),
            ok: true,
            verifier: verifier.to_string(),
            inputs,
            counts: ProofArtifactsCheckCounts {
                selected: report.selected,
                executed: report.executed,
            },
            execution_status: Some(report.execution_status.clone()),
            guard_reason: Some(report.guard_reason.clone()),
            errors: Vec::new(),
        },
        Err(error) => ProofArtifactsCheckReceipt {
            schema: schema.to_string(),
            ok: false,
            verifier: verifier.to_string(),
            inputs,
            counts: ProofArtifactsCheckCounts {
                selected: 0,
                executed: 0,
            },
            execution_status: None,
            guard_reason: None,
            errors: vec![error.to_string()],
        },
    }
}

pub(super) fn proof_run_check_receipt(
    args: &ProofRunArtifactsCheckArgs,
    outcome: &std::result::Result<ProofRunArtifactsReport, anyhow::Error>,
) -> ProofArtifactsCheckReceipt {
    let mut inputs = BTreeMap::new();
    inputs.insert(
        "proof_run_summary".to_string(),
        args.proof_run_summary.to_string_lossy().to_string(),
    );

    match outcome {
        Ok(report) => ProofArtifactsCheckReceipt {
            schema: PROOF_RUN_ARTIFACTS_CHECK_SCHEMA.to_string(),
            ok: true,
            verifier: "proof-run-artifacts-check".to_string(),
            inputs,
            counts: ProofArtifactsCheckCounts {
                selected: report.executed,
                executed: report.executed,
            },
            execution_status: Some("executed".to_string()),
            guard_reason: Some(report.guard_reason.clone()),
            errors: Vec::new(),
        },
        Err(error) => ProofArtifactsCheckReceipt {
            schema: PROOF_RUN_ARTIFACTS_CHECK_SCHEMA.to_string(),
            ok: false,
            verifier: "proof-run-artifacts-check".to_string(),
            inputs,
            counts: ProofArtifactsCheckCounts {
                selected: 0,
                executed: 0,
            },
            execution_status: None,
            guard_reason: None,
            errors: vec![error.to_string()],
        },
    }
}

pub(super) fn write_check_receipt(
    path: Option<&Path>,
    receipt: &ProofArtifactsCheckReceipt,
) -> Result<()> {
    if let Some(path) = path {
        write_text(path, &serde_json::to_string_pretty(receipt)?)?;
    }
    Ok(())
}
