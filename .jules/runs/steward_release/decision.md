# Steward Release Decision

## Option A (Recommended)
Remove the stale `advisories` ignore in `deny.toml` that throws a `warning[advisory-not-detected]` for `RUSTSEC-2023-0071`. This is a low-risk, high-confidence cleanup of a governance/release metadata file that avoids false-positive warnings during CI gates (`cargo deny --all-features check`).

- **Why it fits:** The assigned scope is `tooling-governance` with an emphasis on release metadata and hardening. Fixing the `deny.toml` warning perfectly aligns with improving the `cargo deny` gate (a key fallback expectation).
- **Trade-offs:**
  - *Structure:* Cleans up outdated configuration.
  - *Velocity:* Slightly speeds up CI by not throwing warnings.
  - *Governance:* Reduces noise during release/security audits.

## Option B
Update `xtask/src/tasks/version_consistency.rs` or `Cargo.toml` to address any potential missing `version` bumps. However, `cargo xtask version-consistency` currently passes without issue, so there's no actual drift to fix.

- **Why to choose it instead:** If there was an actual version drift.
- **Trade-offs:** We'd be hallucinating a fix because the checks already pass.

## ✅ Decision
Option A. It's an honest patch that addresses a real, observable issue (`warning[advisory-not-detected]`) in the `deny.toml` file, which is part of the `tooling-governance` shard.
