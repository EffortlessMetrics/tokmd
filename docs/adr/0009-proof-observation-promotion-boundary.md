# ADR-0009: Proof observation promotion boundary

- Status: accepted
- Date: 2026-05-15

## Context

tokmd now has a Rust-owned proof-control plane that can classify changed files,
plan affected proof, execute explicitly opted-in required proof, collect
advisory executor observations, summarize promotion-readiness thresholds, and
verify an aggregate proof-observation decision packet.

The current artifact chain includes:

- `affected.json` from `cargo xtask affected`;
- `proof-plan.json` and `proof-evidence.json` from `cargo xtask proof`;
- `proof-run-summary.json` and `proof-run-observation.json` for required proof
  when a lane explicitly opts in;
- `proof-executor-observation-collection.json` for downloaded advisory
  executor observations;
- `proof-executor-promotion-readiness.json` for checked policy thresholds;
- `proof-observation-decision.json` from `cargo xtask proof-observation-status`;
- `proof-observation-decision-check.json` from
  `cargo xtask proof-observation-status-check`.

That makes observations reviewable, but it does not make them gates. The
important durable decision is whether the existence of this machinery changes
the current proof policy.

## Decision

Proof observations remain in routine observation mode.

The current decision is **continued observation, not promotion**. Advisory fast
proof, scoped coverage, mutation, coverage telemetry, and Codecov upload remain
advisory unless a future maintainer decision changes checked policy and
workflow behavior deliberately.

The proof-observation decision packet is a review artifact. It may summarize
criteria as met or missing, but it does not:

- execute proof;
- make advisory evidence required;
- enable default Codecov upload;
- increase the default PR executor command limit;
- replace source artifact verifiers;
- produce a merge verdict;
- make cockpit or handoff treat advisory proof as passing proof.

A future promotion proposal must cite fresh source receipts and a verified
decision packet from the post-verifier collector flow before changing
`ci/proof.toml` or GitHub workflow defaults.

## Consequences

- Maintainers get one verified advisory surface for proof-observation review
  without changing CI gates.
- GitHub Actions remains the runner, cache, artifact upload, and external
  GitHub API shell; Rust-owned `xtask` commands own receipt semantics.
- Cockpit and handoff may later link verified decision receipts as evidence
  handles, but they must not treat advisory proof as a merge decision.
- Promotion work needs a separate proposal or ADR that names the proof family,
  policy change, workflow change, rollback path, and fresh evidence.
- Missing, stale, skipped, or advisory evidence remains visible as evidence
  state, not as proof that a requirement passed.

## Alternatives

- Promote scoped coverage or fast proof immediately because the collector and
  decision packet exist.
- Remove the collector because advisory evidence is not a gate.
- Move decision-packet semantics into GitHub Actions YAML.

These alternatives were rejected. Immediate promotion would confuse evidence
visibility with gate readiness. Removing the collector would discard useful
review evidence. Encoding semantics in workflow YAML would move receipt logic
out of the Rust-owned proof-control plane.

## Enforcement

- `ci/proof.toml` remains the checked policy source for proof scopes, executor
  defaults, and promotion thresholds.
- `cargo xtask proof-policy --check` must continue rejecting required-gate or
  default-Codecov behavior unless the policy explicitly supports it.
- `cargo xtask proof-observation-status-check` verifies only the aggregate
  decision packet; source artifacts keep their own verifiers.
- Workflow changes may upload observation and decision receipts, but they must
  not silently promote advisory checks.
- Cockpit or handoff integration requires a separate plan and must link the
  verified receipt rather than replacing its verifier.

## Related specs

- `docs/specs/proof-observation-decision-packet.md`
- `docs/ci/proof-observation-artifacts.md`
- `docs/plans/proof-observation-decision-readiness.md`
- `ci/proof.toml`
