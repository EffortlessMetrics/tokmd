## 💡 Summary
Fixed compilation of the `tokmd` crate when `--no-default-features` is used. The `render` command module and `run` module were incorrectly gated by the `analysis` feature flag in `commands/mod.rs`.

## 🎯 Why
When compiling `tokmd` without default features (which excludes the `analysis` feature), the crate failed to build due to mismatched `#[cfg(feature = "analysis")]` annotations around the `run` and `render` module declarations in `crates/tokmd/src/commands/mod.rs`. The `run` subcommand explicitly depends on `analysis_utils` (which is gated by `analysis`), so its module should be gated as well. Conversely, `render` was gated when it shouldn't have been.

## 🔎 Evidence
- `crates/tokmd/src/commands/mod.rs`
- Running `cargo check -p tokmd --no-default-features` previously failed with:
  ```
  error[E0432]: unresolved import `crate::analysis_utils`
  error[E0433]: cannot find module or crate `render` in this scope
  ```

## 🧭 Options considered
### Option A (recommended)
- Properly add `#[cfg(feature = "analysis")]` to `pub(crate) mod run;` and remove it from `pub(crate) mod render;` in `crates/tokmd/src/commands/mod.rs`.
- Fits because it specifically solves the `--no-default-features` build breakage without altering broader behavior.
- trade-offs: Structure / Velocity / Governance - Safest and most precise fix.

### Option B
- Ignore the build breakage for `--no-default-features`.
- when to choose it instead: If the workspace deliberately does not support no-default-features builds, which is not the case here given the compat-matrix profile.
- trade-offs: Allows matrix compatibility to remain degraded.

## ✅ Decision
Option A was chosen to fulfill the compat-matrix fallback expectation of ensuring `--no-default-features` builds successfully.

## 🧱 Changes made (SRP)
- `crates/tokmd/src/commands/mod.rs`:
  - Added `#[cfg(feature = "analysis")]` above `pub(crate) mod run;`
  - Removed `#[cfg(feature = "analysis")]` above `pub(crate) mod render;`

## 🧪 Verification receipts
```text
cargo check -p tokmd --no-default-features
cargo check -p tokmd --all-features
cargo test -p tokmd --no-default-features
cargo fmt -- --check
cargo clippy -- -D warnings
```

## 🧭 Telemetry
- Change shape: Feature flag correction
- Blast radius (API / IO / docs / schema / concurrency / compatibility / dependencies): Compatibility - specifically resolving `--no-default-features` breakage.
- Risk class + why: Low risk, solely fixes broken conditional compilation boundaries.
- Rollback: Revert the PR
- Gates run: cargo test/check --no-default-features, cargo check --all-features, cargo fmt, cargo clippy

## 🗂️ .jules artifacts
- `.jules/runs/compat_interfaces_matrix_run1/envelope.json`
- `.jules/runs/compat_interfaces_matrix_run1/decision.md`
- `.jules/runs/compat_interfaces_matrix_run1/receipts.jsonl`
- `.jules/runs/compat_interfaces_matrix_run1/result.json`
- `.jules/runs/compat_interfaces_matrix_run1/pr_body.md`

## 🔜 Follow-ups
None.
