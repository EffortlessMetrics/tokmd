## Problem
`crates/tokmd/src/export_bundle.rs` triggers `dead_code` warnings when compiled with `--no-default-features`. This is because the struct and functions in this file are only utilized by commands that are conditionally compiled out when `tokmd-core/analysis` feature is missing (like `badge`, `baseline`, `analyze`, and `gate`).

These warnings violate feature-boundary hygiene as features should ideally compile warning-free in all combinations, especially with `--no-default-features`. The fact that `export_bundle` handles run receipts means it belongs to the analysis or processing tier conceptually.

## Options Considered

### Option A (recommended)
- **What it is:** Add `#![allow(dead_code)]` at the top of `crates/tokmd/src/export_bundle.rs`.
- **Why it fits:** It's a localized, low-risk fix that satisfies `--no-default-features` strict checks without complicating feature bounds. Alternatively, we could wrap the entire file or its module inclusion in `#[cfg(feature = "analysis")]`. The latter is even better architectural alignment since this file is inherently tied to `analyze`, `badge`, `baseline`, and `gate` subcommands.
- **Trade-offs:**
  - *Structure:* Better boundary hygiene by explicitly tying the file to the feature that uses it, or silencing the harmless dead code warnings. Wrapping the module inclusion in `lib.rs` with `#[cfg(feature = "analysis")]` is cleaner.
  - *Velocity:* Quick and verifiable.
  - *Governance:* Complies with no-warnings strict builds under all feature combinations.

### Option B
- **What it is:** Do not change anything and accept the dead code warning.
- **When to choose it instead:** Never, as it breaks `--no-default-features` matrix builds when `-D warnings` is enforced (e.g. in CI or gate profile).
- **Trade-offs:** Leaves a broken build state.

## Decision
Option A - Specifically, wrap the `export_bundle` module inclusion in `crates/tokmd/src/lib.rs` with `#[cfg(feature = "analysis")]` instead of just `#![allow(dead_code)]`. Wait, is it used by other things? Let's check where it's used.

Let's check where `export_bundle` is used in `crates/tokmd/src/commands/mod.rs` and `crates/tokmd/src/lib.rs`.
