## 💡 Summary
Fixes unused code warnings for `export_bundle` when building `tokmd` with `--no-default-features`.

## 🎯 Why
When compiling without default features (which excludes the `analysis` feature), the `export_bundle` module is compiled but entirely unused, resulting in dead code warnings (`ExportMetaLite`, `ExportBundle`, `load_export_from_inputs`, etc.). To ensure a clean matrix build, we must conditionally compile this module only when `feature = "analysis"` is active.

## 🔎 Evidence
- file path: `crates/tokmd/src/lib.rs`
- observed behavior: `cargo check -p tokmd --no-default-features` produced multiple `dead_code` warnings.

## 🧭 Options considered
### Option A (recommended)
- what it is: Conditionally compile `mod export_bundle;` in `lib.rs` using `#[cfg(feature = "analysis")]`.
- why it fits this repo and shard: It is the most idiomatic Rust approach to dead code feature flag warnings and limits compilation only to the required surfaces.
- trade-offs:
  - Structure: Clean feature boundary.
  - Velocity: Minor improvement to `--no-default-features` build times.
  - Governance: No behavior change.

### Option B
- what it is: Use `#[allow(dead_code)]`.
- when to choose it instead: If the code could conceptually be consumed elsewhere outside of an explicit feature flag.
- trade-offs: Degrades code hygiene and suppresses legitimate warnings if it truly becomes dead code.

## ✅ Decision
Chose Option A to maintain strong code hygiene and accurate feature boundary isolation.

## 🧱 Changes made (SRP)
- Added `#[cfg(feature = "analysis")]` to `mod export_bundle;` in `crates/tokmd/src/lib.rs`.

## 🧪 Verification receipts
```text
{"command": "cargo check -p tokmd --no-default-features", "status": "success"}
{"command": "cargo test -p tokmd --no-default-features", "status": "success"}
{"command": "cargo test -p tokmd --all-features", "status": "success"}
```

## 🧭 Telemetry
- Change shape: Minor conditional compilation fix.
- Blast radius: Extremely low risk. Internal feature matrix compatibility. No public API/IO changes.
- Risk class: Low
- Rollback: Trivial git revert.
- Gates run: `cargo check/test` across `--no-default-features` and `--all-features`.

## 🗂️ .jules artifacts
- `.jules/runs/compat_interfaces_matrix/envelope.json`
- `.jules/runs/compat_interfaces_matrix/decision.md`
- `.jules/runs/compat_interfaces_matrix/receipts.jsonl`
- `.jules/runs/compat_interfaces_matrix/result.json`
- `.jules/runs/compat_interfaces_matrix/pr_body.md`

## 🔜 Follow-ups
None.
