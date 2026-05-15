# Proof Observation Artifacts

This page inventories the proof-observation artifacts that exist today. It is a
decision-readiness guide, not a promotion decision.

The current proof-control mode remains advisory: fast proof-run observations,
scoped coverage executor observations, mutation evidence, and Codecov upload do
not become required gates unless maintainers intentionally change policy after
reviewing collected evidence.

## Reading Order

For a normal PR or maintainer review, start here:

1. `target/proof/affected.json`
2. `target/proof/proof-plan.json`
3. `target/proof/proof-evidence.json`
4. `target/proof-run/proof-run-summary.json`, when required proof was executed
5. `target/proof-run/proof-run-observation.json`, when required proof was
   converted into a compact observation
6. `target/proof/proof-executor-observation.json`, when advisory executor proof
   ran
7. `target/proof-observations/proof-executor-promotion-readiness.json`, when a
   maintainer is evaluating promotion readiness

The important distinction is planned versus executed evidence:

- `proof-plan.json` and `proof-evidence.json` describe selected or planned work.
- `proof-run-summary.json`, executor summaries with executed counts, and
  observation artifacts describe work that actually ran.
- Collection and readiness receipts summarize observations; they do not execute
  proof and do not make advisory evidence required.

## Artifact Inventory

| Artifact | Schema | Usual path | Writer | Decision use | Verification |
| --- | --- | --- | --- | --- | --- |
| `affected.json` | `tokmd.affected.v1` | `target/proof/affected.json` | `cargo xtask affected --json-output <path>` | Shows changed files, matched proof scopes, and unknown files before proof is planned. | Inspect unknown files; affected planning must report zero unknown files before scoped proof is trusted. |
| `proof-policy.json` | `tokmd.proof_policy.v1` | `target/proof/proof-policy.json` or `target/proof-observations/proof-policy.json` | `cargo xtask proof-policy --json-output <path>` | Captures checked proof policy, executor defaults, promotion thresholds, and Codecov/default-gate state. | `cargo xtask proof-policy --check`. |
| `proof-plan.json` | `tokmd.proof_plan.v1` | `target/proof/proof-plan.json` | `cargo xtask proof --profile affected --plan --plan-json <path>` | Shows required and advisory commands selected from affected scopes. | Re-run the same plan command against the same base/head. |
| `proof-evidence.json` | `tokmd.proof_evidence_plan.v1` | `target/proof/proof-evidence.json` | `cargo xtask proof --plan --evidence-json <path>` | Shows planned evidence families and status before execution. | Treat `planned` / `not_executed` as not passing proof. |
| `proof-plan.md` | markdown summary | `target/proof/proof-plan.md` | `cargo xtask proof --plan --summary-md <path>` | Gives a PR-comment-friendly summary of the selected proof plan. | Compare with `proof-plan.json` if counts or command text matter. |
| `proof-run-summary.json` | `tokmd.proof_run_summary.v1` | `target/proof-run/proof-run-summary.json` or `target/proof/proof-run-summary.json` | `cargo xtask proof --run-required --proof-run-summary <path>` | Shows required proof commands that actually executed under an explicit local or CI guard. | `cargo xtask proof-run-artifacts-check --proof-run-summary <path>`. |
| `proof-run-observation.json` | `tokmd.proof_run_observation.v1` | `target/proof-run/proof-run-observation.json` | `cargo xtask proof-run-observation --proof-run-summary <path> --output <path>` | Compact required-proof observation for routine collection and cockpit import. | Verify the source `proof-run-summary.json` first; collection rejects failed or count-drifted observations. |
| `proof-run-observation-collection.json` | `tokmd.proof_run_observation_collection.v1` | `target/proof-run-observations/proof-run-observation-collection.json` | `cargo xtask proof-run-observations-summary ...` | Summarizes required proof-run observations by profile, scope, guard, and source-run window. | Use as trend evidence only; it is not a proof executor and not a gate. |
| `executor-summary.json` | `tokmd.proof_executor_summary.v1` | `target/proof/executor-summary.json` | `cargo xtask proof --executor-summary <path>` | Shows planner-selected non-required executor commands and execution/skipped status. | `cargo xtask proof-artifacts-check` for non-executed artifacts or `cargo xtask proof-execution-artifacts-check` for executed artifacts. |
| `executor-manifest.json` | `tokmd.proof_executor_manifest.v1` | `target/proof/executor-manifest.json` | `cargo xtask proof --executor-manifest <path>` | Provides stable executor command ids, policy guard state, and selection metadata. | Check summary/manifest consistency with the relevant proof artifact checker. |
| `proof-executor-observation.json` | `tokmd.proof_executor_observation.v1` | `target/proof/proof-executor-observation.json` | `cargo xtask proof-execution-observation --executor-summary <path> --executor-manifest <path> --output <path>` | Compact observation of executed non-required executor evidence, currently scoped coverage. | Source summary and manifest must pass `proof-execution-artifacts-check`; collection rejects failed or count-drifted observations. |
| `proof-executor-observation-collection.json` | `tokmd.proof_executor_observation_collection.v1` | `target/proof-observations/proof-executor-observation-collection.json` | `cargo xtask proof-execution-observations-summary --observations-dir <dir> --output <path>` | Summarizes downloaded executor observations by family, scope, source-run window, and missing/unmatched artifacts. | Use with saved `runs.json`; it is collection evidence, not a gate. |
| `proof-executor-promotion-readiness.json` | `tokmd.proof_executor_promotion_readiness.v1` | `target/proof-observations/proof-executor-promotion-readiness.json` | `cargo xtask proof-execution-observations-summary --promotion-readiness <path>` | Compares collected executor observations against policy-backed promotion thresholds. | Read as readiness evidence only; promotion still requires a maintainer decision and policy/workflow change. |
| `proof-observation-decision.json` | `tokmd.proof_observation_decision.v1` | `target/proof-observations/proof-observation-decision.json` | `cargo xtask proof-observation-status --json <path>` | Advisory aggregate over supplied proof artifacts: source artifacts, required/advisory proof counts, freshness, thresholds, criteria met/missing, and reproduction commands. | It does not execute proof, promote gates, upload Codecov, or replace the source artifact verifiers. |
| `proof-observation-decision-check.json` | `tokmd.proof_observation_decision_check.v1` | `target/proof-observations/proof-observation-decision-check.json` | `cargo xtask proof-observation-status-check --decision <path> --json <path>` | Verifies the advisory decision packet shape, policy guardrails, source artifact references, count consistency, criteria shape, and reproduction commands. | It verifies only the aggregate packet. Source artifacts still need their own verifiers before a promotion decision. |
| `coverage-receipt.json` | `tokmd.coverage_receipt.v1` | `target/coverage/coverage-receipt.json` | `cargo xtask coverage-receipt` | Inventories coverage artifacts and byte counts for coverage telemetry. | Pair with coverage workflow logs and executor observations; byte counts do not prove coverage quality. |
| `thresholds.env` | shell env | `target/proof-observations/thresholds.env` | `cargo xtask proof-observation-thresholds --proof-policy-json <path> --env-output <path>` | Resolves collector thresholds from checked policy plus optional manual overrides. | It is workflow glue, not evidence by itself. |
| `run-ids.txt` | text | `target/proof-observations/run-ids.txt` | `cargo xtask proof-observation-run-ids --runs-json <path> --output <path>` | Turns a saved GitHub run-list window into deterministic download ids for the collector. | Verify the source `runs.json`; external `gh run list` remains the GitHub Actions boundary. |
| artifact-check text/JSON outputs | verifier-specific | `target/proof/*check*` | `cargo xtask proof-artifacts-check`, `proof-execution-artifacts-check`, or `proof-run-artifacts-check` | Shows whether planned, executed, or required proof artifacts satisfy their local verifier contract. | Verifies only the named artifact family and source files. |

## Decision-Ready Signals

These artifacts are immediately useful for a maintainer decision:

- `proof-policy.json`, because it records whether evidence is advisory or
  required and whether Codecov upload is default-enabled.
- `affected.json`, because it shows whether proof routing covered all changed
  files.
- `proof-run-summary.json` plus `proof-run-observation.json`, because they show
  required proof that actually ran.
- `executor-summary.json`, `executor-manifest.json`, and
  `proof-executor-observation.json`, because they show advisory executor proof
  that actually ran and which artifacts it produced.
- `proof-executor-observation-collection.json`, because it shows repeated
  evidence across a source-run window.
- `proof-executor-promotion-readiness.json`, because it applies checked policy
  thresholds to the collected executor observations.

These artifacts still need summarization before they are comfortable for a
promotion review:

- Multiple source-run windows need a compact trend view. The current collection
  receipt can show one window; a decision packet should make repeated windows
  easy to compare without opening raw GitHub artifacts.
- Required proof-run and executor observations are separate collections. A
  decision packet should place required, advisory, skipped, missing, and stale
  evidence in one review surface.
- Coverage receipts prove artifact presence and byte counts, not assertion
  quality. Any promotion decision still needs mutation, targeted tests, or
  maintainer judgment where relevant.
- Current collection receipts identify missing and unmatched artifacts, but
  they do not decide whether that absence is acceptable.

## Reproduction Commands

Generate the core plan and planned-evidence receipts:

```bash
cargo xtask affected --base origin/main --head HEAD --json-output target/proof/affected.json
cargo xtask proof --profile affected --base origin/main --head HEAD --plan --plan-json target/proof/proof-plan.json --evidence-json target/proof/proof-evidence.json --summary-md target/proof/proof-plan.md
```

Execute and verify required proof when a lane intentionally opts in:

```bash
cargo xtask proof --profile affected --base origin/main --head HEAD --run-required --allow-local-required-execution --proof-run-summary target/proof-run/proof-run-summary.json
cargo xtask proof-run-artifacts-check --proof-run-summary target/proof-run/proof-run-summary.json
cargo xtask proof-run-observation --proof-run-summary target/proof-run/proof-run-summary.json --output target/proof-run/proof-run-observation.json
```

Verify or collect advisory executor artifacts only when they already exist:

```bash
cargo xtask proof-execution-artifacts-check --executor-summary target/proof/executor-summary.json --executor-manifest target/proof/executor-manifest.json
cargo xtask proof-execution-observation --executor-summary target/proof/executor-summary.json --executor-manifest target/proof/executor-manifest.json --output target/proof/proof-executor-observation.json
cargo xtask proof-execution-observations-summary --observations-dir target/proof-observations/runs --output target/proof-observations/proof-executor-observation-collection.json --summary-md target/proof-observations/proof-executor-observation-collection.md --promotion-readiness target/proof-observations/proof-executor-promotion-readiness.json
```

Resolve collector thresholds from checked policy:

```bash
cargo xtask proof-policy --json-output target/proof-observations/proof-policy.json
cargo xtask proof-observation-thresholds --proof-policy-json target/proof-observations/proof-policy.json --env-output target/proof-observations/thresholds.env
```

Aggregate the supplied artifacts into an advisory decision-status packet:

```bash
cargo xtask proof-observation-status \
  --affected target/proof/affected.json \
  --proof-policy target/proof-observations/proof-policy.json \
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

## What This Does Not Decide

This inventory does not decide that any proof family is ready to promote. It
only identifies which artifacts a future proposal, spec, or decision record
should cite.

The draft aggregate contract lives in
[`docs/specs/proof-observation-decision-packet.md`](../specs/proof-observation-decision-packet.md).
It defines a future advisory packet shape for summarizing the artifacts listed
here without executing proof or changing policy.

Promotion still requires:

- a maintainer decision;
- a policy change in `ci/proof.toml`;
- matching workflow behavior in the same review;
- a rollback path;
- evidence that the promoted proof catches meaningful failures within an
  acceptable runtime and flake budget.

Until then, routine proof observations remain advisory evidence.
