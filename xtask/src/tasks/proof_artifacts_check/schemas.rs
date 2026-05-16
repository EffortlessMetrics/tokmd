pub(super) const SUMMARY_SCHEMA: &str = "tokmd.proof_executor_summary.v1";
pub(super) const MANIFEST_SCHEMA: &str = "tokmd.proof_executor_manifest.v1";
pub(super) const PROOF_RUN_SUMMARY_SCHEMA: &str = "tokmd.proof_run_summary.v1";
pub(super) const PROOF_RUN_OBSERVATION_SCHEMA: &str = "tokmd.proof_run_observation.v1";
pub(super) const PROOF_RUN_OBSERVATION_COLLECTION_SCHEMA: &str =
    "tokmd.proof_run_observation_collection.v1";
pub(super) const OBSERVATION_SCHEMA: &str = "tokmd.proof_executor_observation.v1";
pub(super) const OBSERVATION_COLLECTION_SCHEMA: &str =
    "tokmd.proof_executor_observation_collection.v1";
pub(super) const PROMOTION_READINESS_SCHEMA: &str = "tokmd.proof_executor_promotion_readiness.v1";
pub(super) const PROOF_ARTIFACTS_CHECK_SCHEMA: &str = "tokmd.proof_artifacts_check.v1";
pub(super) const PROOF_EXECUTION_ARTIFACTS_CHECK_SCHEMA: &str =
    "tokmd.proof_execution_artifacts_check.v1";
pub(super) const PROOF_RUN_ARTIFACTS_CHECK_SCHEMA: &str = "tokmd.proof_run_artifacts_check.v1";

pub(super) const SHARED_FIELDS: &[&str] = &[
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

pub(super) const ENTRY_FIELDS: &[&str] = &[
    "scope",
    "kind",
    "required",
    "command",
    "artifact_path",
    "status",
    "skip_reason",
    "exit_code",
];
