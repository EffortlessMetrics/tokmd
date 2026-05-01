## 💡 Summary
Wrapped `export_bundle` module inclusion in `crates/tokmd/src/lib.rs` with `#[cfg(feature = "analysis")]`. This resolves dead code warnings when compiling the crate with `--no-default-features`.

## 🎯 Why
When compiling `tokmd` without default features (which excludes the `analysis` feature), the `export_bundle` module is compiled, but none of its contents are used. This triggers `dead_code` warnings from `rustc` and breaks strict build constraints like `-D warnings`. Since `export_bundle` handles run receipts which conceptually belong to analysis workflows, its inclusion should respect feature boundaries.

## 🔎 Evidence
- Running `cargo clippy -p tokmd --no-default-features -- -D warnings` throws 8 dead code warnings.
- The `export_bundle.rs` is used exclusively by `badge`, `baseline`, `analyze`, and `gate` subcommands, which are gated behind `#[cfg(feature = "analysis")]` in `crates/tokmd/src/commands/mod.rs`.

## 🧭 Options considered
### Option A (recommended)
- Explicitly gating `mod export_bundle;` in `lib.rs` behind `#[cfg(feature = "analysis")]`.
- **Why it fits this repo and shard:** Adheres to feature-boundary hygiene and structure conventions for the workspace. Silences dead code warnings under strict checking rules.
- **Trade-offs:** Minimal trade-offs. Properly aligns the file module inclusion with its consumers.

### Option B
- Add `#![allow(dead_code)]` to the top of `export_bundle.rs`.
- **When to choose it instead:** If `export_bundle` was used inconsistently across different features and managing the `cfg` became too complex.
- **Trade-offs:** Masks potential genuinely dead code.

## ✅ Decision
Option A. This cleanly associates the module with the feature that relies on it and ensures warnings do not halt restrictive builds.

## 🧱 Changes made (SRP)
- `crates/tokmd/src/lib.rs`

## 🧪 Verification receipts
```text
cargo test -p tokmd --no-default-features
test result: ok. 19 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.27s

cargo clippy -p tokmd --no-default-features -- -D warnings
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 2.40s

cargo clippy --workspace --all-targets -- -D warnings
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 1m 12s
```

## 🧭 Telemetry
- Change shape: Module cfg bound update.
- Blast radius: Compilation configurations under missing `analysis` feature. Minimal to no behavioral change.
- Risk class: Low
- Rollback: Revert `#[cfg(feature = "analysis")]` from `mod export_bundle;` in `lib.rs`.
- Gates run: `cargo check`, `cargo clippy`, `cargo test`, `cargo xtask version-consistency`, `cargo xtask publish --plan`.

## 🗂️ .jules artifacts
- `.jules/runs/surveyor_workspace/envelope.json`
- `.jules/runs/surveyor_workspace/decision.md`
- `.jules/runs/surveyor_workspace/receipts.jsonl`
- `.jules/runs/surveyor_workspace/result.json`
- `.jules/runs/surveyor_workspace/pr_body.md`

## 🔜 Follow-ups
None.
