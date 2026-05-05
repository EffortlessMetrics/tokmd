## 💡 Summary
Replaced non-deterministic `localeCompare` with strict Unicode code unit comparison for sorting HTML analysis reports.

## 🎯 Why
`String.prototype.localeCompare()` is platform-dependent and causes determinism drift in JS/Node environments (as noted in PR #1551 for browser runners). Replacing it ensures exact lexicographical sorting matching Rust's `BTreeMap` and `String::cmp` behavior, thereby locking in deterministic contract-bearing output across environments.

## 🔎 Evidence
- File path: `crates/tokmd-format/src/analysis/templates/report.html`
- Observed behavior: Used `aVal.localeCompare(bVal)` for column sorting.
- Receipt: Tests failing initially without updating insta snapshots, demonstrating the output generation uses this logic.

## 🧭 Options considered
### Option A (recommended)
- Replace `localeCompare` with strict Unicode code unit comparison (`a < b ? -1 : a > b ? 1 : 0`).
- Fits because it eliminates platform-dependent determinism drift and aligns with the strict determinism requirements of the `core-pipeline` shard.
- Trade-offs: Structure: simplified; Velocity: high; Governance: aligns with the `contracts-determinism` gate profile.

### Option B
- Keep `localeCompare` but explicitly pass a fixed locale like `'en-US'`.
- Choose this if human-linguistic sorting is preferred over exact deterministic structural sorting.
- Trade-offs: May still suffer from subtle platform engine (V8/SpiderMonkey) implementation differences, violating strict determinism gates.

## ✅ Decision
Option A was chosen to enforce strictly deterministic output and align perfectly with Rust's native sorting behavior across environments.

## 🧱 Changes made (SRP)
- `crates/tokmd-format/src/analysis/templates/report.html`: Updated sorting logic to use strict `<`/`>` comparisons instead of `localeCompare`.

## 🧪 Verification receipts
```text
INSTA_UPDATE=always cargo test -p tokmd-format
cargo test -p tokmd-format
```

## 🧭 Telemetry
- Change shape: Refactor
- Blast radius: Output HTML formatting only.
- Risk class: Low, only changes client-side sorting behavior to be strictly deterministic.
- Rollback: Revert the template changes and insta snapshots.
- Gates run: `cargo test -p tokmd-format`

## 🗂️ .jules artifacts
- `.jules/runs/run-gatekeeper-determinism/envelope.json`
- `.jules/runs/run-gatekeeper-determinism/decision.md`
- `.jules/runs/run-gatekeeper-determinism/receipts.jsonl`
- `.jules/runs/run-gatekeeper-determinism/result.json`
- `.jules/runs/run-gatekeeper-determinism/pr_body.md`

## 🔜 Follow-ups
None.
