## 💡 Summary
Removed unnecessary `String` allocations for static property names in the CycloneDX exporter. This structural improvement prevents 7+ string allocations per file row during SBOM export.

## 🎯 Why
During CycloneDX export (`tokmd-format` crate), the `CycloneDxProperty` struct was declaring `name: String`, forcing allocations for static strings like `"tokmd:code"`, `"tokmd:lang"`, etc. For large repos, allocating strings per property per file scales poorly and slows down export while increasing peak memory usage.

## 🔎 Evidence
Minimal proof:
- file path: `crates/tokmd-format/src/export/cyclonedx.rs`
- observed behavior: `CycloneDxProperty` defined `name: String`, and the exporter iterated rows, doing `name: "tokmd:code".to_string()`.
- The fix correctly changes `name` to `&'static str` for static keys.

## 🧭 Options considered
### Option A (recommended)
- what it is: Update `CycloneDxProperty` struct to use `&'static str` for the `name` field, removing `.to_string()` for static keys.
- why it fits this repo and shard: Direct structural performance win in formatting pipeline, strictly avoiding allocations.
- trade-offs: Structure / Velocity / Governance - Structure improves, velocity is unchanged, governance stays in line with performance expectations.

### Option B
- what it is: Use `Cow<'static, str>` instead of `&'static str`.
- when to choose it instead: If the `name` field needed to be dynamically generated occasionally.
- trade-offs: More complex code for no added benefit since all properties here use static literal names.

## ✅ Decision
Chose Option A to strictly eliminate the allocation without complexity overhead.

## 🧱 Changes made (SRP)
- `crates/tokmd-format/src/export/cyclonedx.rs`

## 🧪 Verification receipts
```text
test result: ok. 144 passed; 0 failed
```

## 🧭 Telemetry
- Change shape: Optimization (structural/hot-path reduction)
- Blast radius: Local to `tokmd-format` CycloneDX serialization. Does not alter JSON schemas structurally, just reduces allocations under the hood before serialization.
- Risk class + why: Low risk. Serialization outputs identical JSON.
- Rollback: Revert the PR.
- Gates run: `cargo check`, `cargo test`, `cargo fmt`, `cargo clippy`

## 🗂️ .jules artifacts
- `.jules/runs/bolt_core_pipeline_refactorer/envelope.json`
- `.jules/runs/bolt_core_pipeline_refactorer/decision.md`
- `.jules/runs/bolt_core_pipeline_refactorer/receipts.jsonl`
- `.jules/runs/bolt_core_pipeline_refactorer/result.json`
- `.jules/runs/bolt_core_pipeline_refactorer/pr_body.md`

## 🔜 Follow-ups
None.
