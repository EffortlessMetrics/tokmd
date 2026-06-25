# Spec: UB-Review Single-Tight CI Gate

- Status: active
- Schema family, if any: n/a (workflow-shape contract enforced by `cargo xtask ci-gate-contract`)
- Related ADRs: `docs/ci/routed-ci-policy.md` (transitional Rust Small routing)
- Related proof scopes: `proof_control_plane`, `project_truth_docs`, `tokmd_cli`
- Tracking: issue #226; family reference `EffortlessMetrics/unsafe-review-swarm` #1524; `EffortlessMetrics/ub-review` #325

## Contract

`tokmd-swarm` adopts the family **single tight CI gate** shape:

1. **One required status check** whose job conclusion reflects **only** the
   deterministic core floor.
2. **One gate job** on a runner chosen by a minimal **advisory** `route` job
   (self-hosted primary, GitHub-hosted overflow).
3. **Advisory `ub-review`** in the **same** gate job (`review-direct`,
   `posting: review`, `continue-on-error: true`).
4. **Fork PRs** skip only the advisory step; the core gate still runs on
   GitHub-hosted overflow.
5. **Advisory failures** emit concise what/why/fix notes in the job summary;
   they never present as a bare red required check.

The deterministic core floor is the cheap static suite that must stay green:

```text
cargo fmt --check          (via cargo xtask gate --check)
cargo check (warm graph)
cargo clippy -D warnings
cargo test --no-run        (compile-only in gate; full tests in core background lane)
cargo xtask gate --check   (repo policy bundle: fmt, warm check, clippy, test-compile)
```

The live migration may launch the broader proof set concurrently in the
background (full `cargo test`, `proof-policy`, and other xtask policy checks)
sharing one workspace `CARGO_TARGET_DIR`. Cargo's target-dir lock serialises
overlap safely; a disk-headroom guard prevents concurrent builds from filling
the runner.

`ub-review` must never fail the merge. Its grouped PR review, gate manifest,
model availability, and lane opinions are advisory only.

### Relationship to evidence packets (#280)

The advisory layer should consume deterministic precontext written before lanes
start (`target/ci-core/precontext.md`) and may attach `sensors/tokmd/` evidence
produced by `tokmd evidence-packet` / the evidence-packet Action path. Packet
artifacts follow `docs/specs/evidence-packet-workflow.md`; they do not replace
the deterministic floor or become merge blockers.

### Claim boundary

This gate shape proves:

- the deterministic static/policy floor passed on the selected runner;
- advisory review artifacts were attempted when org secrets are available;
- route receipts explain self-hosted vs GitHub-hosted overflow.

It does **not** prove:

- undefined-behavior absence, memory safety, or public reachability;
- merge readiness beyond the deterministic floor;
- that LLM review ran successfully (advisory only);
- release, publish, signing, fuzz, mutation, Miri, or deep proof lanes;
- that fork PRs received LLM review (intentionally skipped).

## Inputs

| Input | Owner | Used for |
| --- | --- | --- |
| GitHub event (`pull_request`, `push`, `merge_group`) | GitHub | Route trust, fork detection, advisory skip |
| `EM_RUNNER_READ_TOKEN` | Org secret | Org runner discovery for self-hosted primary |
| `MINIMAX_API_KEY`, `OPENCODE` | Org secrets | Advisory model lanes (same-repo PRs only) |
| Base/head refs | Workflow | Diff scope, precontext, ub-review packet |
| `target/ci-core/precontext.md` | Gate job | `pr-thread-context` for ub-review lanes |

## Outputs

| Artifact | Consumer | Required |
| --- | --- | --- |
| Required check: `Tokmd Rust Result` | Branch protection | yes |
| `target/ci-core/precontext.md` | ub-review `pr-thread-context` | yes on PRs |
| `target/ci-core/core_exit` / `core.log` | Final assert step | yes |
| ub-review packet under `target/ub-review/` | Reviewers, ledger | advisory |
| Grouped PR review | GitHub PR UI | advisory, same-repo only |
| Route job outputs (`runner`, `runner_kind`) | Gate `runs-on` | yes |

## Target workflow shape

Reference fixture: `fixtures/ci-gate-contract/reference-ci.yml`.

```text
jobs:
  route          # advisory, ubuntu-latest, NOT required
  tokmd-rust-result   # single required check
    setup once (checkout fetch-depth 0, toolchain, rust-cache)
    fast precontext + background core gate
    ub-review (advisory, fork-skipped)
    ub-review advisory status note
    assert core gate verdict (only hard failure)
```

### Advisory ub-review pins

- Action: `EffortlessMetrics/ub-review@<immutable-sha>` (bump deliberately).
- `mode: review-direct`, `posting: review`, `setup-rust: false`,
  `tool-bundle: core`.
- `provider-policy: minimax-primary`, `minimax-model: MiniMax-M3`,
  `opencode-model: deepseek-v4-flash`.
- Repo-local `ub-review` config TOML is preferred once available; until then
  document the preset/config choice in the workflow comment block.

### Permissions

```yaml
permissions:
  contents: read
```

The gate job adds `pull-requests: write` only so ub-review can post its grouped
advisory review.

## Compatibility

Pre-migration baseline (phases 1–2):

- `.github/workflows/ci.yml` ran many parallel jobs with a `CI (Required)`
  aggregator.
- A separate routed Rust Small frontdoor workflow owned `Tokmd Rust Small
  Result`. It was retired in phase 3 (#299); its routing folded into the
  `ci.yml` `route` job and the required `Tokmd Rust Result` gate.

Phased migration:

| Phase | Scope | Required-check impact |
| --- | --- | --- |
| 1 (this spec) | Normative contract, `cargo xtask ci-gate-contract`, reference fixture, advisory live gap report | none |
| 2 | Collapse `ci.yml` frontdoor into single gate + route; retire `CI (Required)` aggregator | replace with `Tokmd Rust Result` |
| 3 (done, #299) | Folded `em-routed-rust-small.yml` routing into `ci.yml` `route` job; retired the duplicate Rust Small workflow and its lane catalogue entries | kept one `Tokmd Rust Result` name |
| 4 | Move remaining advisory workflows (cockpit, ripr, coverage upload) to explicit non-required lanes | unchanged advisory posture |

Until phase 2 lands, satellite workflows (`cockpit.yml`, `droid-review.yml`,
`coverage.yml`, `ripr.yml`, `ci-policy.yml`, etc.) remain separate and
advisory/non-required unless already configured otherwise.

## Proof Requirements

| Proof | Establishes | Does not establish |
| --- | --- | --- |
| `cargo xtask ci-gate-contract --check --workflow fixtures/ci-gate-contract/reference-ci.yml` | Reference target shape matches contract markers | Live `ci.yml` is migrated |
| `cargo test -p xtask ci_gate_contract` | Checker rejects forbidden multi-lane markers and missing required markers | Workflow runtime behaviour on GitHub |
| CI Policy advisory step on live `.github/workflows/ci.yml` | Reports migration gap with actionable summary | Blocks merge during phase 1 |
| Phase 2+ required step on live `ci.yml` | Live workflow keeps the tight gate shape | ub-review model quality |

Checker owner: `cargo xtask ci-gate-contract` in `xtask/src/tasks/ci_gate_contract.rs`.

## Open Questions

- Repo-local `ub-review` profile for Rust/tokmd PRs (preset vs `config:` TOML).
- Whether full `cargo test --all-features` stays in the background core lane or
  moves behind risk-pack labels after consolidation.
- Branch-protection rename plan: `CI (Required)` + `Tokmd Rust Small Result`
  → single `Tokmd Rust Result`.
