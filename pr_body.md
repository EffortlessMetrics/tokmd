## 💡 Summary
Added a doctest to `pub fn scan` in `crates/tokmd-scan/src/lib.rs` to demonstrate typical library usage of setting up mock source files, applying `ScanOptions`, and checking the resulting code count metrics.

## 🎯 Why / Threat model
Improves library usability for downstream developers by documenting the usage pattern of the core scanning function inline, ensuring the documentation does not silently drift since it runs during `cargo test`.

## 🔎 Finding (evidence)
* `crates/tokmd-scan/src/lib.rs` lacked a doctest on its core `pub fn scan` method.

## 🧭 Options considered
### Option A (recommended)
- Add a doctest explicitly constructing mock `ScanOptions` and demonstrating the scan.
- Why it fits this repo: This repository values doctests as a truth mechanism to prevent documentation drift.
- Trade-offs: Increases CI runtime slightly but adds strong verification guarantees.

### Option B
- Document the behavior implicitly using `README.md` examples.
- When to choose it instead: If the setup logic involves non-trivial mock environment generation that makes the doctest noisy.
- Trade-offs: Documentation examples get stale, reducing trust in the API reference over time.

## ✅ Decision
Option A was selected to enforce truth and trust via a verified doctest directly on the function signature.

## 🧱 Changes made (SRP)
- `crates/tokmd-scan/src/lib.rs`: Added a doctest demonstrating `scan` functionality.

## 🧪 Verification receipts
```json
[
  "running 1 test\ntest crates/tokmd-scan/src/lib.rs - scan (line 80) ... ok\n\ntest result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s\n\nall doctests ran in 2.66s; merged doctests compilation took 2.55s\n    Compiling tokmd-scan v1.8.1 (/app/crates/tokmd-scan)\n    Finished `test` profile [unoptimized + debuginfo] target(s) in 3.42s\n   Doc-tests tokmd_scan\n"
]
```

## 🧭 Telemetry
- Change shape: Documentation addition
- Blast radius: Isolated to documentation rendering and testing; no changes to production code logic.
- Risk class: Negligible
- Rollback: Revert commit
- Merge-confidence gates: `cargo fmt -- --check`, `cargo test -p tokmd-scan`

## 🗂️ .jules updates
Created run envelope `.jules/docs/envelopes/20260322T093657Z.json` and appended the run context to `.jules/docs/ledger.json`.
