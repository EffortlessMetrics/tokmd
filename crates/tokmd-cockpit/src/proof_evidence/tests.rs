use super::artifacts::COVERAGE_RECEIPT_SCHEMA;
use super::model::ProofEvidenceKind;
use super::proof_evidence_kind;

#[test]
fn reports_proof_evidence_kind() {
    let raw = serde_json::json!({
        "schema": COVERAGE_RECEIPT_SCHEMA,
        "schema_version": 1,
        "repo": "EffortlessMetrics/tokmd",
        "lane": "scoped",
        "flag": "tokmd_cockpit",
        "workflow": "Coverage",
        "sha": "abc123",
        "github": {
            "run_id": "12345",
            "run_attempt": "1",
            "event_name": "pull_request",
            "ref_name": "feature"
        },
        "artifacts": [],
        "status": { "ok": true, "missing": [], "empty": [] }
    });

    assert_eq!(
        proof_evidence_kind(&raw.to_string()).expect("proof evidence kind"),
        ProofEvidenceKind::CoverageReceipt
    );
}
