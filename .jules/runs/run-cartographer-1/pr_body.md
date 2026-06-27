## 💡 Summary
Aligned the `ROADMAP.md` crate hierarchy table with the actual shipped architecture and updated the file policy to cover unowned test fixtures and scripts. The obsolete internal module boundaries in the roadmap (`tokmd-sensor::substrate`, `tokmd-format::redact`, etc.) were removed, and the newly shipped crates (`tokmd-io-port`, `tokmd-cockpit`, and `tokmd-wasm`) were added.

## 🎯 Why
The roadmap's architecture summary drifted from shipped reality. Updating the `Crate Hierarchy` ensures alignment with `docs/architecture.md` and the actual workspace package closure, reducing confusion for contributors navigating the crate graph. Additionally, resolving strict file-policy drift allows gating commands to pass.

## 🔎 Evidence
- File path: `ROADMAP.md`
- Observed behavior: Listed modules instead of crates and lacked newer Tier 0, 3, and 5 crates.
- Receipt: `cargo xtask version-consistency` outputs all 16 active crates aligned.
- File path: `policy/non-rust-allowlist.toml`
- Receipt: `cargo xtask check-file-policy --strict` failed with missing fixtures and scripts, now passes cleanly.

## 🧭 Options considered
### Option A (recommended)
- what it is: Update the `ROADMAP.md` table to match the current crates.io publish list and `docs/architecture.md`. Add `policy/non-rust-allowlist.toml` entries for missing fixtures.
- why it fits this repo and shard: Fixes a concrete drift issue cleanly within the `tooling-governance` scope and shard.
- trade-offs: Structure: High alignment with source of truth. Velocity: Quick, low-risk fix. Governance: Reduces confusion for new contributors.

### Option B
- what it is: Fully restructure the roadmap based on current active goals and architecture.
- when to choose it instead: If the whole roadmap required a ground-up strategic rewrite instead of factual drift correction.
- trade-offs: High effort, likely out of scope for a single PR change.

## ✅ Decision
Option A. Updating the `ROADMAP.md` table to match the current crates.io publish reality and fixing the file-policy drift fixes a concrete issue cleanly within the scope and shard.

## 🧱 Changes made (SRP)
- `ROADMAP.md`
- `policy/non-rust-allowlist.toml`

## 🧪 Verification receipts
```text
$ cargo xtask docs --check
Documentation is up to date.

$ cargo xtask version-consistency
Checking version consistency against workspace version 1.13.1
  ✓ Cargo crate versions match 1.13.1.
  ✓ Cargo workspace dependency versions match 1.13.1.
  ✓ Node package manifest versions match 1.13.1.
  ✓ No case-insensitive tracked-path collisions detected.
Version consistency checks passed.

$ cargo xtask check-file-policy --strict
file-policy OK: 85 entries, 1162 non-Rust files covered, 1309 Rust files skipped
```

## 🧭 Telemetry
- Change shape: Docs/Policy patch
- Blast radius: Docs / Governance only
- Risk class: Low
- Rollback: git revert
- Gates run: `cargo xtask publish --plan --verbose`, `cargo xtask version-consistency`, `cargo xtask docs --check`, `cargo xtask check-file-policy --strict`

## 🗂️ .jules artifacts
- `.jules/runs/run-cartographer-1/envelope.json`
- `.jules/runs/run-cartographer-1/decision.md`
- `.jules/runs/run-cartographer-1/receipts.jsonl`
- `.jules/runs/run-cartographer-1/result.json`
- `.jules/runs/run-cartographer-1/pr_body.md`

## 🔜 Follow-ups
None.
