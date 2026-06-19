## 💡 Summary
Added explicit invariant bounds checking for `api_surface` report generator collections and `derived` module density metrics. This tightens proptest coverage around output limits that prevent large or runaway payloads.

## 🎯 Why
Unit tests verified that API surface reporting correctly caps `by_module` at 50 and `top_exporters` at 20, but these limits lacked property-based verification over randomly generated structural boundaries. Similarly, `derived` module density ratios lacked verification that values never exceed `1.0`. Locking these down via invariants reduces regression risk as report processing evolves.

## 🔎 Evidence
- `crates/tokmd-analysis/src/api_surface/tests/properties.rs`
- `crates/tokmd-analysis/src/derived/tests/properties.rs`
- Proptests for structural invariants missed hard boundary limits for `api_surface.by_module` (`<= 50`) and `api_surface.top_exporters` (`<= 20`).
- Missing property bound assertions `[0.0, 1.0]` for `whitespace.total.ratio` and `doc_density.total.ratio`.

## 🧭 Options considered
### Option A (recommended)
- Explicitly test the `top_exporters` (`<= 20`) and `by_module` (`<= 50`) upper bounds inside `api_surface/tests/properties.rs`. Limits iterations to `5` since tests perform repetitive disk I/O per bounds validation.
- Verify ratio variables (`doc_density`, `whitespace`) always reside within `[0.0, 1.0]` bounds in `derived/tests/properties.rs`.
- Fits the `property` gate profile by asserting bounds strictly holding for dynamically scaled inputs.

### Option B
- Only apply `api_surface` limits, ignoring `derived` module ratios.
- Reduces test expansion, but leaves numerical boundaries unproven in property sweeps.

## ✅ Decision
Option A. Enforcing collection bounds (`<= 50`/`<= 20`) directly maps to known serialization stability assumptions, while numerical bounds prevent unexpected `f64` drift. Disk I/O is constrained appropriately to maintain CI velocity.

## 🧱 Changes made (SRP)
- `crates/tokmd-analysis/src/api_surface/tests/properties.rs`: Added `by_module_length_bounded` and `top_exporters_length_bounded` proptests.
- `crates/tokmd-analysis/src/derived/tests/properties.rs`: Added `whitespace_ratio_in_unit_range`, `doc_density_ratio_in_unit_range`, and `verbosity_rate_positive`.

## 🧪 Verification receipts
```text
python3 update_properties.py
python3 update_properties2.py
rm update_properties.py update_properties2.py
cargo test -p tokmd-analysis -- --nocapture | grep -i fail
cargo fmt -- --check
cargo clippy -- -D warnings
```

## 🧭 Telemetry
- Change shape: Tests
- Blast radius: `tests/properties.rs`
- Risk class: Low - Test-only tightening
- Rollback: Revert
- Gates run: `cargo test`, `cargo fmt`, `cargo clippy`

## 🗂️ .jules artifacts
- `.jules/runs/invariant_model_analysis/envelope.json`
- `.jules/runs/invariant_model_analysis/decision.md`
- `.jules/runs/invariant_model_analysis/receipts.jsonl`
- `.jules/runs/invariant_model_analysis/result.json`
- `.jules/runs/invariant_model_analysis/pr_body.md`

## 🔜 Follow-ups
None.
