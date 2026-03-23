---
# PR Glass Cockpit

Make review boring. Make truth cheap.

## 💡 Summary
Converted `rust,ignore` codeblocks to `rust,no_run` in the core and format crates' README files. This forces `cargo test` to compile the examples and ensure the syntax remains valid over time.

## 🎯 Why / Threat model
Documentation codeblocks often drift from the actual code base. By compiling them, we prevent users from encountering broken examples when they copy-paste them from the README or docs.rs.

## 🔎 Finding (evidence)
- `crates/tokmd-core/README.md` and `crates/tokmd-format/README.md`
- Both had `rust,ignore` blocks containing valid Rust code that can be statically checked.
- We confirmed the blocks compile successfully using `rust,no_run`.

## 🧭 Options considered
### Option A (recommended)
- Replace `rust,ignore` with `rust,no_run` in `tokmd-core` and `tokmd-format` README files.
- Why it fits this repo: This ensures examples are compiled and correctly verified against current types/traits without requiring runtime setups. Prevents example drift.
- Trade-offs: Increases compilation overhead during `cargo test --doc` but the impact is minimal.

### Option B
- Add missing doctest coverage for an untested usage pattern.
- When to choose it instead: If the existing README blocks were entirely pseudo-code and could not be compiled.

## ✅ Decision
Option A was chosen. It immediately prevents documentation drift by utilizing Cargo's existing compiler checks, with zero new code required.

## 🧱 Changes made (SRP)
- Changed `rust,ignore` to `rust,no_run` in `crates/tokmd-core/README.md`.
- Changed `rust,ignore` to `rust,no_run` in `crates/tokmd-format/README.md`.

## 🧪 Verification receipts
```json
{
  "test_docs_core": "cargo test -p tokmd-core --doc",
  "test_docs_format": "cargo test -p tokmd-format --doc"
}
```

## 🧭 Telemetry
- Change shape: Simple text replacement across two markdown files.
- Blast radius (API / IO / config / schema / concurrency): Negligible. Only affects `cargo test --doc` coverage.
- Risk class + why: Lowest risk. Purely docs/test configurations.
- Rollback: Revert the PR.
- Merge-confidence gates: `cargo test --doc` runs for `tokmd-core` and `tokmd-format`, plus full repo `cargo test` and `cargo build`.

## 🗂️ .jules updates
- Updated `.jules/docs/ledger.json` with a new `fix_doctest_drift` run log.
- Generated an envelope `.jules/docs/envelopes/55d008a4-139a-4d03-91c8-9982ad3a5e0e.json` containing run details.
- Generated run notes at `.jules/docs/runs/2026-03-23.md`.

## 📝 Notes (freeform)
Using `rust,no_run` enforces compilation but prevents actual execution. This avoids panics or runtime failures (like missing files or permissions issues) when `cargo test` runs, while still ensuring the code's semantic correctness against the latest project API.
