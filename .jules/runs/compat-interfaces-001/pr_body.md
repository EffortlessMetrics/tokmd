## 💡 Summary
Fixed a compilation failure when building `tokmd` without default features. The crate could not be built with `cargo check -p tokmd --no-default-features` because several modules (`baseline`, `check_ignore`, `handoff`) blindly called `tokmd_git` without feature gating. Added proper `#[cfg(feature = "git")]` and fallback paths.

## 🎯 Why
Building the crate with `--no-default-features` resulted in `error[E0433]: failed to resolve: use of unresolved module or unlinked crate tokmd_git`. Respecting feature boundaries is necessary for platform and downstream dependency compatibility.

## 🔎 Evidence
- `crates/tokmd/src/commands/check_ignore.rs`
- `crates/tokmd/src/commands/baseline.rs`
- `crates/tokmd/src/commands/handoff.rs`
- `cargo check -p tokmd --no-default-features` failed with:
  ```
  error[E0433]: failed to resolve: use of unresolved module or unlinked crate `tokmd_git`
  ```

## 🧭 Options considered
### Option A (recommended)
- Conditionally compile the `tokmd_git` checks using `#[cfg(feature = "git")]` and provide `#[cfg(not(feature = "git"))]` fallbacks (e.g., returning `false` or `None`).
- Respects the modular architecture of the crate and avoids forcing the `git` dependency.
- Trade-offs: Requires a bit more boilerplate code for the fallback implementations.

### Option B
- Always require the `git` feature in `Cargo.toml`.
- When to choose it instead: If the application cannot function at all without `git`.
- Trade-offs: Defeats the purpose of the feature flags and prevents minimal builds.

## ✅ Decision
Chose Option A to cleanly fix the compilation issue while maintaining feature modularity.

## 🧱 Changes made (SRP)
- `crates/tokmd/src/commands/check_ignore.rs`
- `crates/tokmd/src/commands/baseline.rs`
- `crates/tokmd/src/commands/handoff.rs`

## 🧪 Verification receipts
```text
cargo check -p tokmd --no-default-features
cargo check -p tokmd --all-features
cargo test -p tokmd --no-default-features
cargo test -p tokmd --all-features
```

## 🧭 Telemetry
- Change shape: Adding feature flags and simple fallbacks
- Blast radius: compatibility
- Risk class + why: low, because the changes merely compile out code when features are omitted
- Rollback: git revert
- Gates run: `cargo check` and `cargo test` with varying feature flags

## 🗂️ .jules artifacts
- `.jules/runs/compat-interfaces-001/envelope.json`
- `.jules/runs/compat-interfaces-001/decision.md`
- `.jules/runs/compat-interfaces-001/receipts.jsonl`
- `.jules/runs/compat-interfaces-001/result.json`
- `.jules/runs/compat-interfaces-001/pr_body.md`

## 🔜 Follow-ups
None.
