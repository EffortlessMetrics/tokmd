# Next Program

The generated PR drain is complete. `PR_DRAIN.md` is now a historical ledger for the duplicate/stale queue and should only change for PR-drain-specific corrections. Active product and control-plane work moves here.

Factory Droid PR #1541 was declined for now. External review services require an approved service, API-key, secret-rotation, fork-PR, and failure-behavior policy before workflow introduction.

## Active Program: Rust-Native Proof Control Plane

Goal: move proof orchestration out of ad hoc GitHub YAML and into checked Rust-owned `xtask` policy and planning logic. GitHub Actions should eventually install tools, restore cache, run `cargo xtask proof ...`, upload artifacts, and show summaries while Rust owns scope mapping, allowlists, dependency boundaries, fixture policy, mutation targeting, coverage targeting, and proof reports.

## Initial Work Packets

1. Add `ci/proof.toml` and `cargo xtask proof-policy --check`.
2. Move dependency-boundary and fixture/blob allowlists into the proof policy while preserving current behavior.
3. Add `cargo xtask affected --base origin/main --head HEAD --json` for changed-file to proof-scope discovery.
4. Add `cargo xtask proof --profile affected --base origin/main --head HEAD --plan` to print a stable proof plan without running it.
5. Wire policy validation and affected-plan artifacts into CI before replacing larger workflow logic.

## Directional Rules

- `tokmd-config` is retired. It must remain forbidden by policy, with ownership in `tokmd-settings`, `tokmd-core`, and `tokmd`.
- `.jules` is an allowed knowledge workspace for durable specs, investigations, friction notes, persona learnings, runbooks, ledgers, envelopes, and generated indexes.
- Coverage remains advisory telemetry until maintainers intentionally promote it to a gate.
- Cockpit remains the current PR-review evidence surface until a separate review command has a distinct artifact contract.

## Checkpoints

- Dependency-boundary checks now read `ci/proof.toml` while preserving the existing sorted `tokmd-analysis*` manifest scan and `dependencies` / `dev-dependencies` / `build-dependencies` coverage.
- Fixture-blob checks now read `ci/proof.toml` while preserving the existing crypto extension and marker detection plus the `.claude`, `.jules`, `vendor`, proof-policy source, and checker-source allowlist behavior.
- `cargo xtask affected --base origin/main --head HEAD --json` now maps changed files to proof scopes from `ci/proof.toml`, reports unknown files, and keeps non-Rust unknown handling policy-driven.
- `cargo xtask proof --profile affected --base origin/main --head HEAD --plan` now prints a stable proof plan without running commands; `fast`, `release`, and `deep` profiles are plan-only placeholders for the next CI integration slices.
- CI now validates `cargo xtask proof-policy --check` as part of the required aggregate and uploads PR-only affected proof artifacts while keeping existing jobs authoritative.
- The proof scope registry now covers first-class product/control-plane surfaces for CLI, gate, cockpit, WASM, browser runner, the composite GitHub Action, schema contracts, and the proof control plane.
- Clippy policy now has a governed ledger and `cargo xtask check-lint-policy` check while keeping the repository MSRV at 1.92 and leaving crate-wide lint inheritance as a later cleanup stack.
- Analysis and formatting module scopes now route `tokmd-analysis`, `tokmd-analysis-types`, and `tokmd-format` changes to targeted package, snapshot, renderer, and module proof commands.
- Next proof-policy operational slice: use affected scope metadata to draft scoped coverage and mutation command plans before moving more GitHub YAML logic.

## References

- Historical drain ledger: [PR_DRAIN.md](../PR_DRAIN.md)
- Current testing strategy: [testing.md](testing.md)
