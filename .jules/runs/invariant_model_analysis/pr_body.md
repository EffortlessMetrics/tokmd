## 💡 Summary
Added missing property-based invariants around metadata license parsing in `tokmd-analysis`. The new tests formalize expectations for TOML key/value parsing (whitespace, quote style, section isolation) and JSON object structuring in `package.json`.

## 🎯 Why
The `parse_toml_key` and `parse_package_json_license` routines in `tokmd_analysis::license` perform best-effort parsing without fully rigorous tokenizing. It's critical to prove that these ad-hoc parsers do not panic on arbitrary inputs and accurately handle formatting edge cases like varying whitespace, section boundaries, and object structures.

## 🔎 Evidence
- `crates/tokmd-analysis/src/license/mod.rs` contains `parse_toml_key`, `extract_quoted`, and `parse_package_json_license` logic.
- We added rigorous proptests in `crates/tokmd-analysis/src/license/tests/properties.rs`.
- `cargo test -p tokmd-analysis properties` passed.

## 🧭 Options considered
### Option A (recommended)
- Add deterministic structural and edge-case property tests to `license/tests/properties.rs`.
- Fits the model/analysis invariant expectation.
- Trade-offs: Structure/Velocity/Governance - Locks down edge cases without refactoring production code.

### Option B
- Add invariant checks to `near_dup`.
- When to choose it instead: If license invariants were fully saturated.
- Trade-offs: `near_dup` already has heavy proptest coverage, while license string-extraction was less locked down.

## ✅ Decision
Option A. It tests string parsing invariants around quote extraction and format structure in the license parser, an important model piece.

## 🧱 Changes made (SRP)
- `crates/tokmd-analysis/src/license/tests/properties.rs`: Added 4 new properties testing JSON layout, TOML key/value spacing, section isolation, and panic prevention on arbitrary inputs.

## 🧪 Verification receipts
```text
{"cmd": "cargo test -p tokmd-analysis properties", "status": "success"}
```

## 🧭 Telemetry
- Change shape: Property-based test additions
- Blast radius: tests
- Risk class: Low - test additions only
- Rollback: revert
- Gates run: property

## 🗂️ .jules artifacts
- `.jules/runs/invariant_model_analysis/envelope.json`
- `.jules/runs/invariant_model_analysis/decision.md`
- `.jules/runs/invariant_model_analysis/receipts.jsonl`
- `.jules/runs/invariant_model_analysis/result.json`
- `.jules/runs/invariant_model_analysis/pr_body.md`

## 🔜 Follow-ups
None.
