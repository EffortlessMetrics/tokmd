# Decision

## Option A (recommended)
Update the `proof-artifacts-check` command in `xtask/src/tasks/proof_artifacts_check.rs` to not fail when `execution_guard.enabled` is `true`.

- **What it is:** Modify the verification logic to accept executor summary artifacts where execution is guarded/disabled (e.g., when the flag `--allow-ci-evidence-execution` is not passed on an external PR).
- **Why it fits this repo and this shard:** The `tooling-governance` shard owns the deterministic checks of CI artifacts, and the `contracts-determinism` gate profile governs CI policy. The CI runner relies on `proof-artifacts-check` to verify the generated artifacts without needing to explicitly mock or disable the guard, especially when processing external PRs.
- **Trade-offs:**
  - *Structure:* Correctly aligns the verification tool with its intent: verifying the artifact structure, not enforcing the CI's choice to guard execution.
  - *Velocity:* High velocity, allows CI to proceed correctly when the guard is active.
  - *Governance:* Aligned with deterministic artifact verification.

## Option B
Update the GitHub CI workflow (`.github/workflows/ci.yml`) to pass `--allow-ci-evidence-execution` to `cargo xtask proof`.

- **What it is:** Change the CI invocation so that the artifact generated always has the guard disabled.
- **When to choose it instead:** If the artifact check was specifically designed to ensure *execution* occurred (which it isn't; its status says `not_executed`).
- **Trade-offs:** Exposes CI to running evidence commands implicitly, violating the `explicit_opt_in` design for CI evidence execution, presenting a security risk on external PRs.

## Decision
**Option A** was chosen because it correctly updates the contract of `proof-artifacts-check` to verify the generated `executor-summary.json` without failing due to the correctly-applied execution guard. This ensures CI correctly handles blocked execution plans (e.g., external PRs) without terminating the pipeline incorrectly.