## 💡 Summary
Added mutation-catching property and serialization roundtrip tests for contract-bearing structs.

## 🎯 Why
The `ManualCandidateRecord` struct had un-asserted parsing behavior for the `test_targets` and `do_not_touch` fields. The `DiffRow` and `DiffTotals` calculation lacked explicit tests for their derived delta fields. The `EvidencePacketStatus` enum was missing exhaustive enum deserialization tests, making these core pipeline structures vulnerable to silent mutations.

## 🔎 Evidence
- `crates/tokmd-types/src/packet_siblings.rs` `ManualCandidateRecord` (missing field tests)
- `crates/tokmd-types/src/diff.rs` `DiffRow` (missing calculated property tests)
- `crates/tokmd-types/src/evidence_packet.rs` `EvidencePacketStatus` (missing explicit string mapping tests)
- Receipt: `cargo test -p tokmd-types` passed, confirming the fields map properly.

## 🧭 Options considered
### Option A (recommended)
- what it is: Adding explicit, exhaustive mapping and field property checks inside `crates/tokmd-types/src/`.
- why it fits this repo and shard: High-value `core-pipeline` types where bugs directly leak to the JSON schema or CLI diff.
- trade-offs: Structure: enforces invariants. Velocity: very fast to run and write. Governance: adheres to strict DTO matching.

### Option B
- what it is: Implementing wide fuzzing inputs.
- when to choose it instead: When aiming to catch runtime panics or hangings instead of missed explicit property checks.
- trade-offs: Higher overhead for simple property checks.

## ✅ Decision
Chose Option A to close immediate serialization mutation gaps and math invariants on high-value types.

## 🧱 Changes made (SRP)
- `crates/tokmd-types/src/packet_siblings.rs` - Added `manual_candidates_roundtrip_with_optional_fields` test.
- `crates/tokmd-types/src/diff.rs` - Added `diff_row_delta_consistency` and `diff_totals_delta_consistency` tests.
- `crates/tokmd-types/src/evidence_packet.rs` - Added `evidence_packet_status_serde_exhaustive` test.

## 🧪 Verification receipts
```text
cargo build --verbose
CI=true cargo test --verbose
cargo fmt -- --check
cargo clippy -- -D warnings
```

## 🧭 Telemetry
- Change shape: Tests added
- Blast radius: None (tests only)
- Risk class: Low
- Rollback: Revert tests
- Gates run: cargo test, clippy, fmt

## 🗂️ .jules artifacts
- `.jules/runs/mutant_high_value/envelope.json`
- `.jules/runs/mutant_high_value/decision.md`
- `.jules/runs/mutant_high_value/receipts.jsonl`
- `.jules/runs/mutant_high_value/result.json`
- `.jules/runs/mutant_high_value/pr_body.md`

## 🔜 Follow-ups
None.
