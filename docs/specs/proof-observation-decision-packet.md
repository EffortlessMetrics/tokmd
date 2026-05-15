# Spec: Proof Observation Decision Packet

- Status: draft
- Schema family, if any: `tokmd.proof_observation_decision.v1`;
  verifier receipt `tokmd.proof_observation_decision_check.v1`
- Related ADRs:
- Related proof scopes: `proof_control_plane`, `project_truth_docs`

## Contract

A proof observation decision packet is a future advisory summary artifact for
maintainers deciding whether proof observations are ready for promotion,
continued observation, rollback, or simplification.

The packet must aggregate existing Rust-owned proof receipts. It must not run
proof commands, upload coverage, change required gates, enable default Codecov
upload, or decide that a PR can merge.

The packet exists because the current proof-control system already has strong
source receipts, but those receipts are spread across required proof,
non-required executor proof, coverage telemetry, policy thresholds, collection
windows, and verifier outputs. A promotion review needs one stable surface that
answers:

- which required proof ran;
- which advisory proof ran;
- which planned proof did not run;
- which artifacts were missing, stale, skipped, or unavailable;
- which policy thresholds were satisfied;
- which thresholds or evidence classes are still missing;
- which source artifact or command reproduces each claim.

The packet must preserve the current advisory boundary. It may report
`criteria_met` and `criteria_missing`, but it must not emit a merge verdict or
flip a policy decision by itself.

## Inputs

The packet may read these existing artifacts when supplied:

- `affected.json` with schema `tokmd.affected.v1`;
- `proof-policy.json` with schema `tokmd.proof_policy.v1`;
- `proof-plan.json` with schema `tokmd.proof_plan.v1`;
- `proof-evidence.json` with schema `tokmd.proof_evidence_plan.v1`;
- `proof-run-summary.json` with schema `tokmd.proof_run_summary.v1`;
- `proof-run-observation.json` with schema `tokmd.proof_run_observation.v1`;
- `proof-run-observation-collection.json` with schema
  `tokmd.proof_run_observation_collection.v1`;
- `executor-summary.json` with schema `tokmd.proof_executor_summary.v1`;
- `executor-manifest.json` with schema `tokmd.proof_executor_manifest.v1`;
- `proof-executor-observation.json` with schema
  `tokmd.proof_executor_observation.v1`;
- `proof-executor-observation-collection.json` with schema
  `tokmd.proof_executor_observation_collection.v1`;
- `proof-executor-promotion-readiness.json` with schema
  `tokmd.proof_executor_promotion_readiness.v1`;
- `coverage-receipt.json` with schema `tokmd.coverage_receipt.v1`;
- verifier outputs from `proof-artifacts-check`,
  `proof-execution-artifacts-check`, or `proof-run-artifacts-check`.

Input paths should be explicit, repo-relative when possible, and recorded as
source references. The packet should not require network access, GitHub API
access, hidden workflow state, timestamps, absolute local paths, or downloaded
artifact directories that are not named by the caller.

The first implementation is developer tooling under `cargo xtask`. It accepts
explicit source artifacts instead of discovering hidden workflow state:

```bash
cargo xtask proof-observation-status \
  --affected target/proof/affected.json \
  --proof-policy target/proof/proof-policy.json \
  --proof-plan target/proof/proof-plan.json \
  --proof-evidence target/proof/proof-evidence.json \
  --proof-run-observation target/proof-run/proof-run-observation.json \
  --executor-observation-collection target/proof-observations/proof-executor-observation-collection.json \
  --promotion-readiness target/proof-observations/proof-executor-promotion-readiness.json \
  --json target/proof-observations/proof-observation-decision.json

cargo xtask proof-observation-status-check \
  --decision target/proof-observations/proof-observation-decision.json \
  --json target/proof-observations/proof-observation-decision-check.json
```

## Outputs

The JSON packet uses schema `tokmd.proof_observation_decision.v1` and remains
visibility-only.

The packet should include:

- `schema`;
- `ok`, meaning the packet was built from parseable inputs, not that proof is
  promoted;
- `mode`, with an initial value such as `observation_only`;
- `source_artifacts`, listing supplied inputs, schemas, and stable relative
  paths;
- `policy_state`, including required-gate and Codecov-default booleans from
  checked proof policy;
- `required_proof`, summarizing executed, passed, failed, skipped, planned, and
  missing required proof;
- `advisory_proof`, summarizing executed, passed, failed, skipped, planned, and
  missing advisory proof;
- `freshness`, reporting whether source refs and observation windows are exact,
  partial, stale, or unknown when metadata is available;
- `thresholds`, copied from checked policy or supplied readiness receipts;
- `criteria_met`;
- `criteria_missing`;
- `reproduce`, listing commands that regenerate the source artifacts;
- `errors`, for input-shape failures or missing required inputs.

The packet should avoid:

- timestamps;
- absolute paths;
- raw GitHub tokens or URLs with credentials;
- workflow log blobs;
- raw coverage file contents;
- pass/fail language that sounds like a merge verdict.

The packet may optionally have a Markdown companion for humans, but JSON should
remain the machine authority.

The verifier receipt uses schema
`tokmd.proof_observation_decision_check.v1`. It verifies only the aggregate
packet: schema and mode, source artifact references, policy guardrails, count
consistency, freshness state, criteria shape, reproduction commands, and empty
embedded errors. It does not replace source artifact verifiers such as
`proof-run-artifacts-check`, `proof-execution-artifacts-check`, or
`proof-policy --check`.

## Compatibility

The decision packet must not change existing artifact schemas or product
receipts. Existing outputs remain authoritative for their own domains:

- `affected.json` owns changed-file to proof-scope routing;
- `proof-plan.json` owns planned commands;
- `proof-run-summary.json` owns executed required proof;
- executor summaries and observations own non-required executor evidence;
- coverage receipts own coverage artifact inventory;
- readiness receipts own policy-threshold comparison.

The decision packet is an aggregate. Consumers must be able to ignore it and
keep using the source artifacts directly.

The first implementation should stay in `xtask`. It must not add a public
`tokmd review` command, change `tokmd cockpit`, change `tokmd handoff`, enable
default Codecov upload, or make advisory evidence required. Cockpit or handoff
integration may come later only when the decision packet and its verifier
receipt are supplied as explicit evidence handles.

## Proof Requirements

For this draft spec, validation is documentation-control proof:

```bash
cargo xtask doc-artifacts --check
cargo xtask docs --check
cargo xtask proof-policy --check
cargo xtask affected --base origin/main --head HEAD --json-output target/proof/affected-proof-observation-decision-packet.json
cargo xtask proof --profile affected --base origin/main --head HEAD --plan --plan-json target/proof/proof-plan-proof-observation-decision-packet.json --evidence-json target/proof/proof-evidence-proof-observation-decision-packet.json
cargo fmt-check
git diff --check
```

If a Rust-owned command is added later, the implementation PR should also add
focused `xtask` tests covering:

- valid aggregate output from minimal source artifacts;
- missing optional evidence reported as missing or unavailable, not passing;
- advisory evidence preserved as advisory;
- stale or unknown freshness not treated as available proof;
- rejected absolute paths and path escapes;
- deterministic output across repeated runs;
- no mutation of source artifacts.

The verifier should also cover both success and failure fixtures:

- valid packet writes deterministic `tokmd.proof_observation_decision_check.v1`;
- true required-gate or default-Codecov fields are rejected as incompatible
  with observation-only mode;
- absolute or escaping source artifact paths are rejected;
- count drift, duplicate criteria, non-`cargo xtask` reproduction commands, or
  non-empty embedded errors are rejected.

## Open Questions

- Whether the first implementation should accept many explicit input flags or a
  single observations directory plus named source artifacts.
- Whether required proof-run observations and executor observations should share
  one freshness classifier in the first implementation.
- Whether readiness should be expressed only as `criteria_met` /
  `criteria_missing`, or whether a small `state` enum such as
  `observe`, `needs_more_data`, and `ready_for_maintainer_review` is useful.
- Whether coverage receipts should be packet-level evidence only or linked to
  specific proof scopes when executor observations include matching artifact
  paths.
- Whether cockpit or handoff should consume the verifier receipt directly or
  only link it as a packet-local evidence handle.
