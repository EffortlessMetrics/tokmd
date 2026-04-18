## 💡 Summary
Added a golden snapshot test for the `tokmd analyze --format json` command output. This locks in the deterministic shape and schema of analysis receipts, completing contract coverage for the primary data pipeline commands alongside `export`, `lang`, and `module`.

## 🎯 Why
The core pipeline (`tokmd analyze`) emits contract-bearing JSON receipts. While regression tests verified that certain fields were stable and sorted, there was no full-output golden snapshot test (unlike `export` and `module`), leaving the surface open to accidental schema or formatting drift.

## 🔎 Evidence
- `crates/tokmd/tests/cli_snapshot_golden.rs` lacked a `snapshot_analyze_json` test.
- `cargo test -p tokmd --test cli_snapshot_golden` verified that the newly added snapshot passes reliably with normalized non-deterministic fields like `base_signature`, `target_path`, and `generated_at_ms`.

## 🧭 Options considered
### Option A (recommended)
- Add `snapshot_analyze_json` to `cli_snapshot_golden.rs` and update normalization functions.
- Fits this repo and shard because it protects contract-bearing outputs and determinism without modifying core pipeline behaviors.
- Trade-offs: Increases test determinism but introduces a minor maintenance burden to run `cargo insta accept` upon valid schema changes.

### Option B
- Add explicit missing tests for edge case flags on `tokmd analyze`.
- Choose this when specific flags cause nondeterminism.
- Trade-offs: Does not comprehensively verify the output structure as a full-snapshot test does.

## ✅ Decision
I chose Option A. By adding `snapshot_analyze_json` and matching regex normalizers for `base_signature` and `target_path`, the entire analysis receipt schema is now structurally locked and deterministic, fulfilling the Gatekeeper persona's objective.

## 🧱 Changes made (SRP)
- Added `snapshot_analyze_json` in `crates/tokmd/tests/cli_snapshot_golden.rs` with `#[cfg(feature = "analysis")]`.
- Updated `normalize` in `crates/tokmd/tests/cli_snapshot_golden.rs` to handle `base_signature` and `target_path`.
- Generated and accepted the new snapshot `crates/tokmd/tests/snapshots/cli_snapshot_golden__analyze_json.snap`.

## 🧪 Verification receipts
```text
$ cargo test -p tokmd --test cli_snapshot_golden
running 13 tests
test snapshot_export_jsonl ... ok
test snapshot_export_json ... ok
test snapshot_export_csv ... ok
test snapshot_analyze_json ... ok
test snapshot_lang_markdown ... ok
test snapshot_help ... ok
test snapshot_lang_json_structure ... ok
test snapshot_lang_json ... ok
test snapshot_module_markdown ... ok
test snapshot_lang_tsv ... ok
test snapshot_module_json ... ok
test snapshot_module_tsv ... ok
test snapshot_version ... ok

test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.26s
```

## 🧭 Telemetry
- Change shape: Test addition
- Blast radius: Tests
- Risk class: Low, only adds tests.
- Rollback: Revert commit
- Gates run: `cargo test -p tokmd --test cli_snapshot_golden`

## 🗂️ .jules artifacts
- `.jules/runs/gatekeeper_determinism/envelope.json`
- `.jules/runs/gatekeeper_determinism/decision.md`
- `.jules/runs/gatekeeper_determinism/receipts.jsonl`
- `.jules/runs/gatekeeper_determinism/result.json`
- `.jules/runs/gatekeeper_determinism/pr_body.md`

## 🔜 Follow-ups
None at this time.
