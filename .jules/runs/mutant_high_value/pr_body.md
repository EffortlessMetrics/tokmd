## 💡 Summary
Added missing mutant-killing tests for the `tokmd-format` diff render formatters.

## 🎯 Why
The file `crates/tokmd-format/src/diff/render.rs` contains logic for formatting output using ANSI colors (`format_delta_colored`, `format_pct_delta_colored`) and calculating percentages (`percent_change`). While there was a mutant-killing test for the basic `format_delta` function, these other helper functions had zero direct test coverage. The Mutant persona requires strengthening behavioral checks around meaningful code paths.

## 🔎 Evidence
- File path: `crates/tokmd-format/src/diff/render.rs`
- Finding: The test module `tests` only contained a single test `test_format_delta`.
- Command receipt: Running `cargo test -p tokmd-format` passed, showing the gap in test coverage.

## 🧭 Options considered
### Option A (recommended)
- Add missing unit tests for `format_delta_colored`, `format_pct_delta_colored`, and `percent_change` in `crates/tokmd-format/src/diff/render.rs`.
- Why it fits this repo and shard: It directly addresses untested display formatting logic in the `tokmd-format` crate, fulfilling the Mutant persona's core objective to improve tests around meaningful code changes.
- Trade-offs: Structure / Velocity / Governance - Low risk, high reward test improvement.

### Option B
- Add more fine-grained assertions in `crates/tokmd-format/src/analysis/tests.rs` to cover `render_obj_coordinate_math`.
- When to choose it instead: If the `diff/render.rs` logic was already heavily tested.
- Trade-offs: Weaker return on investment because the `fun_outputs` math already has explicit mutant-killing tests.

## ✅ Decision
Option A was chosen. Adding the missing unit tests for the diff render formatters provides immediate value by closing a concrete assertion gap for production code that currently had zero direct test coverage.

## 🧱 Changes made (SRP)
- `crates/tokmd-format/src/diff/render.rs`: Added `test_format_delta_colored`, `test_format_pct_delta_colored`, and `test_percent_change`.

## 🧪 Verification receipts
```text
test diff::render::tests::test_format_delta_colored ... ok
test diff::render::tests::test_format_pct_delta_colored ... ok
test diff::render::tests::test_percent_change ... ok
test diff::tests::test_compute_diff_rows_language_added ... ok
test diff::tests::test_compute_diff_rows_language_removed ... ok
test diff::tests::test_compute_diff_rows_unchanged_excluded ... ok
...
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.20s
     Running `/app/target/debug/deps/tokmd_format-a61ddec0c687c132`
```

## 🧭 Telemetry
- Change shape: Tests only
- Blast radius: None (tests only)
- Risk class + why: Low risk; changes are strictly additive test functions.
- Rollback: `git revert`
- Gates run: `mutation` (fallback rules: `cargo mutants`, targeted tests, `cargo build`, `cargo test`, `cargo fmt`, `cargo clippy`)

## 🗂️ .jules artifacts
- `.jules/runs/mutant_high_value/envelope.json`
- `.jules/runs/mutant_high_value/decision.md`
- `.jules/runs/mutant_high_value/receipts.jsonl`
- `.jules/runs/mutant_high_value/result.json`
- `.jules/runs/mutant_high_value/pr_body.md`

## 🔜 Follow-ups
None at this time.
