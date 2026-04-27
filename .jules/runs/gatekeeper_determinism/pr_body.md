## 💡 Summary
Added JSON serialization stability property tests under `proptest` for the core pipeline outputs (`FileRow`, `ModuleRow`, `LangRow`, `DiffRow`, `DiffTotals`). This hardens the core types against arbitrary randomized data without changing production code. (Note: previous sort-property tests were removed to avoid duplicating and re-implementing logic belonging to `tokmd-model`).

## 🎯 Why
Our core structural stability invariants are documented and checked against fixed snapshots or limited unit tests, but they were not exhaustively fuzzed or stressed under `proptest` for complex fields like `DiffRow` containing negative values, zeroes, or large scale deltas. The Gatekeeper `contracts-determinism` profile prioritizes proving that contract-bearing surfaces (like our JSON output rows) correctly round-trip safely.

## 🔎 Evidence
- Run `cargo test -p tokmd-types --test determinism_proptest_w80` and observe new exhaustively generated property tests for JSON stability logic over `tokmd_types`.

## 🧭 Options considered
### Option A (recommended)
- Add comprehensive `proptest` files verifying determinism of core output structs.
- Fits the Gatekeeper profile and Shard assignments since it expands proof surface without risking production code changes.
- trade-offs: Structure is improved, Velocity may decrease if people accidentally break tie-break invariants, Governance is enforced.

### Option B
- Tweak minor description drifts in `docs/schema.json` without code change.
- When to choose: if significant drift exists in schema docs.
- Trade-offs: lower leverage impact, no new proof properties added.

## ✅ Decision
Chosen **Option A**. Adding property tests directly hits the highest priority Gatekeeper objective: tightening invariants around deterministic outputs.

## 🧱 Changes made (SRP)
- Added `crates/tokmd-types/tests/determinism_proptest_w80.rs`

## 🧪 Verification receipts
```text
cargo test -p tokmd-types --test determinism_proptest_w80

running 5 tests
test diff_totals_serialize_roundtrip_stable ... ok
test lang_row_serialize_roundtrip_stable ... ok
test module_row_serialize_roundtrip_stable ... ok
test diff_row_serialize_roundtrip_stable ... ok
test file_row_serialize_roundtrip_stable ... ok

test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.10s
```

## 🧭 Telemetry
- Change shape: New test file added.
- Blast radius: Zero API/IO/runtime impact. `tokmd-types` test-only change.
- Risk class: Low, only runs during tests.
- Rollback: `git restore crates/tokmd-types/tests/determinism_proptest_w80.rs`
- Gates run: `cargo test -p tokmd-types`, `cargo fmt`, `cargo clippy`.

## 🗂️ .jules artifacts
- `.jules/runs/gatekeeper_determinism/envelope.json`
- `.jules/runs/gatekeeper_determinism/decision.md`
- `.jules/runs/gatekeeper_determinism/receipts.jsonl`
- `.jules/runs/gatekeeper_determinism/result.json`
- `.jules/runs/gatekeeper_determinism/pr_body.md`

## 🔜 Follow-ups
None.
