# PR Glass Cockpit

Make review boring. Make truth cheap.

## 💡 Summary
Replaced generic `.expect("should exist")` and `.expect("should have name")` unwrap/expect assertions with descriptive, context-aware messages in `crates/tokmd/src/context_pack.rs` and `crates/tokmd/src/export_bundle.rs`.

## 🎯 Why (user/dev pain)
The generic "should exist" or "should have name" messages on test assertions or critical execution paths provide poor Developer Experience (DX). When tests fail or the code panics on these, the maintainer has no context about *what* invariant failed without meticulously reading the stack trace. By providing highly specific messages like "failed to parse explicit token budget from CLI string", troubleshooting is significantly accelerated.

## 🔎 Evidence (before/after)
- **Before**: `assert_eq!(parse_budget("128k").expect("should exist"), 128_000);`
- **After**: `assert_eq!(parse_budget("128k").expect("failed to parse explicit token budget from CLI string"), 128_000);`
- **File paths**:
  - `crates/tokmd/src/context_pack.rs`
  - `crates/tokmd/src/export_bundle.rs`

## 🧭 Options considered
### Option A (recommended)
- What it is: Clean up generic `.expect("should exist")` messages in test and execution logic by replacing them with descriptive context-aware panic messages.
- Why it fits this repo: Directly improves DX and readability; tightly constrained, safe, and fits the strict Palette persona mandate.
- Trade-offs: Structure: none. Velocity: slightly slower than bulk `unwrap()` replacements. Governance: strict and easy to verify.

### Option B
- What it is: Refactor testing logic entirely to use `anyhow::Result` and the `?` operator.
- When to choose it instead: For larger restructuring tasks or full panic burndowns across entire test modules.
- Trade-offs: Increases blast radius unnecessarily for a quick DX improvement pass, making review harder.

## ✅ Decision
Choosing Option A because the generic `.expect("should exist")` pattern degrades the DX for future maintainers trying to track test panics. Improving the `expect()` strings is low-risk, tightly constrained, and squarely fits the Palette persona's mandate for error message quality.

## 🧱 Changes made (SRP)
- `crates/tokmd/src/context_pack.rs`: Replaced 31 instances of `.expect("should exist")` with context specific, high fidelity failure reasons (e.g. `failed to parse explicit token budget from CLI string`, `expected README.md to be present in selection`, etc).
- `crates/tokmd/src/export_bundle.rs`: Replaced `.expect("should exist")` and `.expect("should have name")` with contextually accurate equivalents referencing export bundle structure.

## 🧪 Verification receipts
- `cargo build`: PASS
- `cargo clippy -- -D warnings`: PASS
- `cargo test -p tokmd`: PASS
- `cargo fmt -- --check`: PASS

## 🧭 Telemetry
- Change shape: Search-and-replace strings in tests and one logic file.
- Blast radius (API / IO / docs / schema / concurrency): Negligible; strictly internal panic messages.
- Risk class + why: Extremely Low. No logic flow changes, only text modifications.
- Rollback: `git checkout -- crates/tokmd/src/context_pack.rs crates/tokmd/src/export_bundle.rs`
- Merge-confidence gates (what ran): `build`, `fmt`, `clippy`, `test`

## 🗂️ .jules updates
- Updated `.jules/palette/ledger.json` with run status and targeted files.
- Appended details of this run to `.jules/palette/runs/YYYY-MM-DD.md`.
- Stored results in envelope `.jules/palette/envelopes/<run_id>.json`.

## 📝 Notes (freeform)
This aligns with the Palette "burn down panics" priority without introducing over-complex refactoring.

## 🔜 Follow-ups
None currently identified.
