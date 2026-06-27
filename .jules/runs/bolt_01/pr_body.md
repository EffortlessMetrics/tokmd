## 💡 Summary
Refactored `compute_diff_rows` in `tokmd-format` to operate on `&str` references instead of cloning `String` objects, avoiding unnecessary allocations during the language aggregation loops.

## 🎯 Why
The `diff` computation pipeline aggregates old and new languages, sorts them, and maps differences. Previously it `clone()`d the language name on every item and implicitly instantiated multiple copies of default `LangRow` fallback structs inside the loop body. Removing allocations reduces heap churn in a core processing path where arrays scale with the number of languages.

## 🔎 Evidence
- **File path(s):** `crates/tokmd-format/src/diff/compute.rs`
- **Observed behavior:** `compute_diff_rows` used to build a `Vec<String>` and instantiate owned `LangRow` structs just to extract integer counts.
- **Receipts:**
  - `cargo bench -p tokmd-format` verified safe scaling
  - `cargo test -p tokmd-format diff` passes deterministically

## 🧭 Options considered
### Option A (recommended)
- Update `compute_diff_rows` to operate on `Vec<&str>` instead of `Vec<String>`.
- Extract scalar properties directly using `Option::unwrap_or` rather than cloning the whole fallback struct instance.
- **Trade-offs:** Minimizes `clone()` operations in the hot path. Improves latency with negligible code footprint changes.

### Option B
- Rewrite JSON formatting pathways to stream values without strings.
- **Trade-offs:** Significantly more complex, breaks modular boundaries and requires widespread rewrite of serialization logic.

## ✅ Decision
Option A was chosen. Changing `compute_diff_rows` to reference existing `&str` fields is a clean, localized win for avoiding string allocations during aggregation.

## 🧱 Changes made (SRP)
- `crates/tokmd-format/src/diff/compute.rs`: Changed intermediate collection in `compute_diff_rows` to `Vec<&str>` and mapped values via `old_row.map(|r| r.code).unwrap_or(0)` instead of `unwrap_or_else(|| LangRow {...})`.

## 🧪 Verification receipts
```text
cargo test -p tokmd-format --verbose
cargo clippy -p tokmd-format -- -D warnings
cargo fmt -- --check
```

## 🧭 Telemetry
- **Change shape:** Localized refactor in string allocations
- **Blast radius:** Low (Diff logic only)
- **Risk class:** Low (Tests cover logic paths comprehensively)
- **Rollback:** Safe to revert.
- **Gates run:** `perf-proof`, `clippy`, `fmt`, `test`

## 🗂️ .jules artifacts
- `.jules/runs/bolt_01/envelope.json`
- `.jules/runs/bolt_01/decision.md`
- `.jules/runs/bolt_01/receipts.jsonl`
- `.jules/runs/bolt_01/result.json`
- `.jules/runs/bolt_01/pr_body.md`

## 🔜 Follow-ups
None
