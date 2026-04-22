## 💡 Summary
Added explicit integration tests to prove that the CLI commands `tokmd run` and `tokmd diff` produce completely deterministic (byte-stable) JSON receipts across multiple execution environments. This completes the deterministic coverage for core pipeline outputs alongside lang, module, and export commands.

## 🎯 Why
The determinism regression suite (`determinism.rs`) thoroughly checked `LangReceipt`, `ModuleReceipt`, and `ExportReceipt` payloads for byte-for-byte exactness across runs, ensuring contract outputs remained perfectly stable. However, `RunReceipt` and `DiffReceipt` determinism were never explicitly proved, leaving a gap where instability could creep into upstream systems consuming these files.

## 🔎 Evidence
- `crates/tokmd/tests/determinism.rs`
- Ran new and existing determinism assertions: `cargo test -p tokmd --test determinism` confirming byte-stable JSON strings.

## 🧭 Options considered
### Option A (recommended)
- Add deterministic snapshot tests directly to the established `crates/tokmd/tests/determinism.rs` suite.
- Why it fits: Aligns smoothly with existing determinism verification structure and satisfies Gatekeeper determinism contracts directly by keeping tests together.
- Trade-offs: Minor code reuse for the envelope normalization block, but velocity is high and structure remains predictable.

### Option B
- Add golden `.snap` files specifically for diff and run in `crates/tokmd/tests/cli_snapshot_golden.rs`.
- When to choose: Helpful for tracking specific visual regressions in outputs, but lacks the same dynamic "re-run N times and compare" stringency.
- Trade-offs: Difficult to manage the temporary folders across mock run environments inside insta frameworks.

## ✅ Decision
Option A was chosen. Adding explicit runtime determinism regressions correctly proves stability for JSON payloads by natively dropping `generated_at_ms` envelopes before doing byte-stable equality tests.

## 🧱 Changes made (SRP)
- `crates/tokmd/tests/determinism.rs` (Appended test invariants for `run_receipt_is_deterministic_across_runs` and `diff_receipt_is_deterministic_across_runs`)

## 🧪 Verification receipts
```text
running 31 tests
test diff_receipt_is_deterministic_across_runs ... ok
test export_csv_consistent_column_count ... ok
test export_csv_is_deterministic ... ok
...
test run_receipt_is_deterministic_across_runs ... ok

test result: ok. 31 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.37s
```

## 🧭 Telemetry
- Change shape: Test Addition
- Blast radius: None to API / IO. Strict addition of property invariants to `determinism.rs`.
- Risk class: Low
- Rollback: Revert additions to `crates/tokmd/tests/determinism.rs`
- Gates run: `cargo test -p tokmd --test determinism`, `cargo fmt`, `cargo clippy -- -D warnings`

## 🗂️ .jules artifacts
- `.jules/runs/gatekeeper_determinism_1/envelope.json`
- `.jules/runs/gatekeeper_determinism_1/decision.md`
- `.jules/runs/gatekeeper_determinism_1/receipts.jsonl`
- `.jules/runs/gatekeeper_determinism_1/result.json`
- `.jules/runs/gatekeeper_determinism_1/pr_body.md`

## 🔜 Follow-ups
None.
