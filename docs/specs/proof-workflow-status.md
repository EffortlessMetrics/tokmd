# Spec: Proof Workflow Status

- Status: active
- Schema family, if any: `tokmd.proof_workflow_status.v1`;
  verifier receipt `tokmd.proof_workflow_status_check.v1`
- Related ADRs:
- Related proof scopes: `proof_control_plane`, `project_truth_docs`

## Contract

A proof workflow status packet is a developer/CI-facing receipt that
summarizes status arbitration for existing proof workflows. It consumes
already-generated proof artifacts and explicit command exit codes, then emits a
small JSON receipt, optional Markdown summary, and workflow-friendly final exit
recommendation.

The packet must not execute proof commands. It must not upload coverage, call
GitHub APIs, install tools, mutate source artifacts, change required CI gates,
promote advisory evidence, enable default Codecov upload, or decide that a PR
can merge.

The first consumers are GitHub Actions workflow steps:

- `.github/workflows/ci.yml` `fast-proof-run`;
- `.github/workflows/proof-executor.yml` `scoped-coverage-executor`.

The producer is `cargo xtask`, not the public `tokmd` CLI. The
workflow remains responsible for runner setup, cache, tool installation,
artifact upload, and service integration. The Rust-owned packet owns only:

- parsing supplied source artifact paths;
- recording explicit command exit codes;
- preserving advisory/required classification from checked policy inputs;
- rendering the human summary table;
- computing the same final workflow exit priority that the shell currently
  applies.

## Inputs

The status command accepts explicit inputs. It must not discover hidden
state from the filesystem, workflow environment, GitHub APIs, timestamps, or
downloaded artifact directories.

Shared inputs:

- `--workflow-kind fast-proof-run | scoped-coverage-executor`;
- `--proof-policy <PATH>` with schema `tokmd.proof_policy.v1`, when available;
- one or more `--status <NAME>=<INTEGER>` values for command exit codes;
- source artifact paths, supplied explicitly and recorded as source refs;
- `--json <PATH>` for the status packet;
- optional `--summary-md <PATH>` for a Markdown summary;
- optional `--env-output <PATH>` for workflow-compatible key/value outputs.

Fast proof-run inputs should cover the current workflow statuses:

- `proof_run_status`;
- `proof_run_artifacts_status`;
- `proof_run_observation_status`;
- `proof-plan.json` with schema `tokmd.proof_plan.v1`;
- `proof-run-summary.json` with schema `tokmd.proof_run_summary.v1`;
- `proof-run-artifacts-check.json` with schema
  `tokmd.proof_run_artifacts_check.v1`;
- `proof-run-observation.json` with schema
  `tokmd.proof_run_observation.v1`.

Scoped coverage executor inputs should cover the current workflow statuses:

- `affected_status`;
- `executor_status`;
- `verifier_status`;
- `observation_status`;
- `collection_status`;
- `affected.json` with schema `tokmd.affected.v1`;
- `proof-plan.json` with schema `tokmd.proof_plan.v1`;
- `executor-summary.json` with schema `tokmd.proof_executor_summary.v1`;
- `executor-manifest.json` with schema `tokmd.proof_executor_manifest.v1`;
- `proof-execution-artifacts-check.json` with schema
  `tokmd.proof_execution_artifacts_check.v1`;
- `proof-executor-observation.json` with schema
  `tokmd.proof_executor_observation.v1`;
- `proof-executor-observation-collection.json` with schema
  `tokmd.proof_executor_observation_collection.v1`.

Input paths must be relative when recorded in the packet. The command should
reject absolute paths and path escapes for source artifact references unless a
future plan documents a narrow CI-only exception.

## Outputs

The JSON packet uses schema `tokmd.proof_workflow_status.v1`.

Initial shape:

```json
{
  "schema": "tokmd.proof_workflow_status.v1",
  "ok": true,
  "mode": "workflow_status_only",
  "workflow_kind": "fast_proof_run",
  "required": false,
  "advisory": true,
  "policy_guardrails": {
    "required_gate": false,
    "codecov_default_upload": false
  },
  "source_artifacts": [
    {
      "role": "proof_run_summary",
      "path": "target/proof-run/proof-run-summary.json",
      "schema": "tokmd.proof_run_summary.v1",
      "required": true,
      "available": true
    }
  ],
  "command_statuses": [
    {
      "name": "proof_run_status",
      "exit_code": 0,
      "blocking": true
    }
  ],
  "recommended_exit_code": 0,
  "summary": {
    "title": "Fast Proof Run",
    "advisory_note": "Fast proof-run artifact generation is advisory and is not part of the required CI aggregate yet."
  },
  "errors": []
}
```

`ok` means the packet was built from parseable inputs and internally
consistent. It does not mean proof should be promoted, coverage should upload,
or a PR should merge.

`recommended_exit_code` mirrors the current workflow shell behavior. For the
current implementation, any non-zero blocking command status should produce
the first non-zero status in current workflow priority order. That keeps the
workflow behavior compatible while moving arbitration into a testable Rust
surface.

The optional Markdown summary should reproduce the current workflow summary
tables in a stable form. The optional env output should contain only simple
workflow keys, such as:

```text
ok=true
recommended_exit_code=0
workflow_kind=fast_proof_run
```

The verifier receipt uses schema `tokmd.proof_workflow_status_check.v1`. It
should verify:

- packet schema and mode;
- supported workflow kind;
- relative source artifact paths;
- source artifact role and schema vocabulary;
- command status names and integer values;
- summary consistency with command statuses;
- recommended exit code consistency with the status priority rule;
- advisory/required guardrails remain compatible with checked policy;
- no embedded errors in a packet marked `ok = true`.

The verifier must not replace source artifact verifiers such as
`proof-run-artifacts-check`, `proof-execution-artifacts-check`,
`proof-policy --check`, or proof observation checkers.

## Compatibility

The status packet is additive and developer/CI-facing. Existing proof artifacts
remain authoritative for their own domains:

- `proof-plan.json` owns planned proof commands;
- `proof-run-summary.json` owns required proof execution results;
- executor summaries and manifests own scoped coverage execution;
- artifact verifier receipts own source artifact validity;
- observations own compact evidence for later aggregation;
- `ci/proof.toml` owns policy.

The implementation must preserve current workflow artifact names, summary
wording, and exit behavior unless the PR explicitly documents a
behavior-compatible rewrite. Consumers must be able to ignore the status packet
and keep reading the source artifacts.

This spec does not add a public `tokmd review` command, does not change
`tokmd cockpit`, does not change `tokmd handoff`, does not enable default
Codecov upload, and does not make advisory evidence required.

## Proof Requirements

For spec-only changes, validation is documentation-control proof:

```bash
cargo xtask doc-artifacts --check
cargo xtask docs --check
cargo xtask proof-policy --check
cargo xtask affected --base origin/main --head HEAD --json-output target/proof/affected-proof-workflow-status.json
cargo xtask proof --profile affected --base origin/main --head HEAD --plan --plan-json target/proof/proof-plan-proof-workflow-status.json --evidence-json target/proof/proof-evidence-proof-workflow-status.json
cargo fmt-check
git diff --check
```

Implementation proof for the Rust command should include focused `xtask` tests
for:

- valid fast proof-run packet output;
- valid scoped coverage executor packet output;
- missing optional artifact reported as unavailable, not passing;
- missing required artifact reported as an error;
- non-zero status priority preserved for each workflow kind;
- advisory status preserved as advisory;
- rejected absolute paths and path escapes;
- deterministic JSON and Markdown output;
- verifier success and failure receipts.

Workflow integration proof should also cover:

- fast proof-run workflow uses the Rust-owned status summary;
- scoped coverage executor remains non-required and PR-visible;
- Codecov upload remains manual-only;
- required CI aggregate semantics are unchanged;
- artifact uploads include the new status packet and check receipt only after
  the verifier exists.

## Open Questions

- Whether future proof workflow kinds should reuse this packet shape or get a
  separate status family.
- Whether future verifier fields should include deeper source-artifact
  cross-checks beyond the current packet-level consistency checks.
- Whether the env output should include a rendered `status` string in addition
  to `recommended_exit_code`.
- Whether future source artifact schema validation should parse deeper than
  top-level schema fields or delegate to existing verifiers when their receipts
  are supplied.
- Whether the Markdown summary should preserve exact current wording or use a
  new compact summary after the first behavior-compatible migration.
