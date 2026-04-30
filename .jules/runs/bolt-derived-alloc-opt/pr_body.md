## 💡 Summary
Converted derived analysis aggregation loops in `crates/tokmd-analysis/src/derived/mod.rs` to operate on borrowed `&str` and reference structures instead of blindly allocating `String`s and `clone()`ing rows at every inner loop step. This optimizes thousands of unnecessary allocations inside report generators like `build_max_file_report`, `build_lang_purity_report`, `build_nesting_report`, `build_boilerplate_report`, and `build_polyglot_report`.

## 🎯 Why
Derived metric loops routinely iterate over every `FileStatRow` or `FileRow` produced during the analysis run, grouping and bucketing them. Previously, inside these inner `for` loops, keys like `.lang.clone()` and `.module.clone()` were eagerly allocated into strings on every loop iteration, even for simple frequency counts or `BTreeMap` insertion logic where `&str` handles map lookups efficiently. These loops sit precisely in the analysis orchestrator's hot path. Refactoring to use borrowed keys locally eliminates these thousands of string allocations and structurally optimizes the analysis phase.

## 🔎 Evidence
- Found in: `crates/tokmd-analysis/src/derived/mod.rs`.
- `build_lang_purity_report`: Previously allocated `row.module.clone()` per hit, mapped into `BTreeMap<String, ...>`, now uses `&str`.
- `build_max_file_report`: Previously continuously cloned `row.clone()` strings inside `by_lang.insert(row.lang.clone(), row.clone())`, now borrows `&FileStatRow`.
- Commands run: `cargo check -p tokmd-analysis`, `cargo test -p tokmd-analysis`, `cargo test -p tokmd --test determinism_regression`. All tests passed with structurally improved aggregation routines.

## 🧭 Options considered
### Option A (recommended)
- Replace internal `BTreeMap<String, ...>` allocations with `BTreeMap<&str, ...>` within report generation loops.
- Maps internal values via refs instead of cloning rows.
- Converts to owned only at final struct aggregation time.
- **Why it fits:** Direct, structural hot-path win perfectly scoped for the Bolt persona.
- **Trade-offs:** `&'a str` lifetime management inside functional boundaries, no visible cost downstream.

### Option B
- Thread scaling with `rayon` parallelism for map grouping.
- **When to choose it:** If string cloning overhead was unfixable or CPU work was computationally dense.
- **Trade-offs:** Wastes binary size and overhead masking bad data structures. Fixing allocation is objectively superior first.

## ✅ Decision
Option A was chosen. Eliminating string allocations at their source avoids creating garbage, structurally tightening performance metrics without sacrificing deterministic outcomes or requiring extra threads.

## 🧱 Changes made (SRP)
- `crates/tokmd-analysis/src/derived/mod.rs`

## 🧪 Verification receipts
```text
cargo test -p tokmd-analysis
...
test result: ok. 59 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.16s
...
cargo test -p tokmd --test determinism_regression
...
test result: ok. 26 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.45s
```

## 🧭 Telemetry
- Change shape: Structural allocation reduction
- Blast radius: API (none) / IO (none) / docs (none) / schema (none) / concurrency (none) / compatibility (none) / dependencies (none)
- Risk class: Low + structurally proven by test harness.
- Rollback: `git checkout crates/tokmd-analysis/src/derived/mod.rs`.
- Gates run: `cargo test -p tokmd-analysis`, `cargo test -p tokmd --test determinism_regression`.

## 🗂️ .jules artifacts
- `.jules/runs/bolt-derived-alloc-opt/envelope.json`
- `.jules/runs/bolt-derived-alloc-opt/decision.md`
- `.jules/runs/bolt-derived-alloc-opt/receipts.jsonl`
- `.jules/runs/bolt-derived-alloc-opt/result.json`
- `.jules/runs/bolt-derived-alloc-opt/pr_body.md`

## 🔜 Follow-ups
None immediately.
