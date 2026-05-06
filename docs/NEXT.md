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
- Affected proof plans now include advisory scoped coverage and mutation commands derived from matched scope metadata while leaving existing proof commands required and CI behavior unchanged.
- `cargo xtask proof --plan --summary-md <path>` now writes a Markdown proof-plan artifact, and the informational PR affected-plan job appends that Rust-generated summary while keeping existing CI jobs authoritative.
- `cargo xtask proof --plan --evidence-json <path>` now writes a machine-readable planned-evidence artifact for scoped coverage and mutation commands. The artifact records `planned` / `not_executed` status and zero executed counts so consumers do not confuse planned advisory evidence with passing evidence.
- `cargo xtask proof --plan --executor-summary <path>` now writes an informational coverage-only executor summary prototype. It selects non-required coverage commands, records them as skipped with `tool_execution_not_enabled`, and does not invoke `cargo llvm-cov`.
- `cargo xtask proof --plan --executor-summary <path> --executor-mode dry-run` now exercises the executor selection boundary for at most one non-required coverage command without invoking `cargo llvm-cov`.
- Executor summaries now include an `execution_guard` block. CI evidence execution remains disabled unless a future workflow explicitly passes `--allow-ci-evidence-execution`, and current executor modes still report zero executed commands.
- Rust-generated proof-plan Markdown now surfaces executor guard status whenever an executor summary is requested, so the affected-plan CI summary shows whether planner-selected evidence execution is blocked or explicitly opted in.
- `ci/proof.toml` now declares the first executor promotion rule: coverage is the only supported executor family, CI execution requires explicit opt-in, and dry-run selection is policy-limited before any CI job can execute planner-selected evidence commands.
- `cargo xtask proof-policy --check` and `--json` now report the active executor policy rule alongside scope, allowlist, fixture blob, and dependency-boundary counts.
- `cargo xtask proof --plan --executor-manifest <path>` now writes a planner-selected executor command manifest with the executor policy, guard status, selection rule, stable command ids, and zero executed counts.
- The PR-only affected-plan CI artifact job now writes and uploads `executor-manifest.json` alongside `affected.json`, `proof-plan.json`, `proof-evidence.json`, `executor-summary.json`, and `proof-plan.md`, without opting into executor command execution.
- `cargo xtask proof-artifacts-check` now verifies executor summary/manifest consistency without executing planned commands, including schema, guard, count, and command-entry drift checks.
- The PR-only affected-plan CI artifact job now runs `cargo xtask proof-artifacts-check` after artifact generation, records its status, and uploads the verifier output while remaining informational.
- The affected-plan CI job now fails on affected-scope generation, proof-plan generation, or proof artifact verifier failure, while executor command execution remains disabled and existing proof commands remain separately authoritative.
- The proof scope registry now classifies model/scan path-normalization changes and `.jules` provenance updates so generated knowledge artifacts and core path hot-path work do not appear as unknown files in affected plans.
- No-panic policy now has a governed allowlist (`policy/no-panic-allowlist.toml`, schema 0.3) and a semantic checker (`cargo xtask check-no-panic-family`). Identity is `path + family + selector` so reformatting source files does not invalidate receipts. Default mode is advisory: schema/shape, expired entries, and stale entries block; unallowlisted findings are reported only. `cargo xtask no-panic-propose` writes proposed allowlist entries to `target/no-panic-proposed-allowlist.toml`. The strict (blocking) flip is staged behind member-crate `[lints] workspace = true` adoption and a panic-family debt burn-down.
- Workspace dependency graph changes now have an explicit proof scope for root/workspace manifests and `Cargo.lock`, routing affected plans to deny, dependency-boundary, and publish-surface checks instead of leaving lockfile-only changes unknown.
- Fuzz harness changes now have an explicit proof scope for `fuzz/Cargo.toml`, targets, dictionaries, corpora, and harness docs, routing affected plans to the fuzz harness inventory check before deeper fuzz execution is promoted.
- The proof executor now has a deliberately opt-in local coverage execution experiment. `--executor-mode execute` cannot be combined with `--plan`, requires explicit local or CI opt-in, runs only planner-selected non-required coverage commands, and writes executor summary/manifest artifacts while required proof jobs remain authoritative.
- `cargo xtask proof-artifacts-check` now allows enabled execution guards for non-executed artifacts; this verifier still rejects executed artifacts by `execution_status` and executed counts until an execution verifier lands.
- `cargo xtask proof-execution-artifacts-check` now verifies opted-in executed executor artifacts separately from the no-execution verifier, requiring executed status, an enabled guard, zero failed commands, and matching summary/manifest command records.
- `.github/workflows/proof-executor.yml` now provides a scoped coverage executor experiment for manual dispatch and non-required PR runs. It runs planner-selected non-required coverage commands with explicit CI opt-in, verifies executed artifacts, uploads proof artifacts, and leaves required PR proof jobs unchanged.
- Manual proof-executor run `25464543145` on `main` passed on 2026-05-06. It verified the workflow-dispatch no-diff path: `affected_status=0`, `executor_status=0`, `verifier_status=0`, `execution_guard.reason=ci_explicit_opt_in_enabled`, and zero selected/executed commands.
- Manual proof-executor run `25465495509` on a disposable branch passed on 2026-05-06. It matched `crates/tokmd-core/tests/ffi_parity_w53.rs` to `tokmd_core_ffi`, selected one advisory coverage command, executed it with `exit_code=0`, produced `target/proof/coverage/tokmd_core_ffi.lcov`, and passed `cargo xtask proof-execution-artifacts-check`.
- The proof executor workflow now runs on pull requests as `Scoped Coverage Executor (Non-Required)`. It remains outside the required CI aggregate, executes only planner-selected non-required coverage commands, keeps Codecov upload manual-only, and leaves existing proof jobs authoritative.
- Next proof-policy operational slice: collect successful non-required PR executor runs across multiple affected scopes before considering any required-gate or default Codecov-upload promotion.

## References

- Historical drain ledger: [PR_DRAIN.md](../PR_DRAIN.md)
- Current testing strategy: [testing.md](testing.md)
