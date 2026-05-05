## 💡 Summary
This is a learning PR. The intended patch addressing `--no-default-features` compilation warnings in `crates/tokmd/src/export_bundle.rs` was gracefully aborted because it was superseded by a merged PR (#1552) that used a structurally superior boundary definition (`#[cfg(feature="analysis")]`).

## 🎯 Why
During the resolution of `dead_code` warnings tied to missing default features, it was discovered via PR feedback that PR #1552 had already merged a fix for the exact issue. That upstream fix utilized proper feature gating (`cfg(feature="analysis")`) which is strictly better than the `allow(dead_code)` workaround attempted here. Proceeding with a redundant or weaker fix violates project guidelines.

## 🔎 Evidence
- PR Comment from reviewer: "Superseded by #1552, which merged the aligned analysis-feature cfg fix. This PR used weaker #[allow(dead_code)] suppression rather than preserving the feature boundary."
- Codebase state confirms that the issue requires `cfg` boundary updates rather than suppression.

## 🧭 Options considered
### Option A
- what it is: Pivot to find another compatibility issue in the `interfaces` shard.
- when to choose it instead: If the original task was fully unblocked and time/scope allowed for a new target discovery.
- trade-offs: Violates the Single Responsibility Principle and prompt constraints which expect one coherent story per run.

### Option B (recommended)
- what it is: Gracefully abort the fix, revert the local changes, and finalize as a learning PR containing a friction item.
- why it fits this repo and shard: Directly adheres to the rule: "If an intended patch is found to be superseded by another merged PR during execution, gracefully abort the redundant fix and create a 'learning PR'".
- trade-offs: Structure/Governance are perfectly preserved. No velocity is lost attempting a doomed patch.

## ✅ Decision
Option B. Aborted the redundant fix and documented the workflow edge case as a friction item.

## 🧱 Changes made (SRP)
- Reverted local changes to `crates/tokmd/src/export_bundle.rs`.

## 🧪 Verification receipts
```text
cargo xtask version-consistency
cargo fmt -- --check
cargo check
```

## 🧭 Telemetry
- Change shape: Reversion and learning generation.
- Blast radius: None.
- Risk class + why: Zero risk.
- Rollback: N/A.
- Gates run: `cargo check`, formatting checks.

## 🗂️ .jules artifacts
- `.jules/runs/compat_interfaces_matrix/envelope.json`
- `.jules/runs/compat_interfaces_matrix/decision.md`
- `.jules/runs/compat_interfaces_matrix/receipts.jsonl`
- `.jules/runs/compat_interfaces_matrix/result.json`
- `.jules/runs/compat_interfaces_matrix/pr_body.md`
- `.jules/friction/open/compat_interfaces_matrix_superseded.md`

## 🔜 Follow-ups
None.
