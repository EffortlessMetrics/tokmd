## 💡 Summary
Fixed a security vulnerability in the CycloneDX SBOM export format where path and module redaction was incorrectly bypassed. Added a dedicated test to prevent regression.

## 🎯 Why
The CycloneDX SBOM builder (`write_export_cyclonedx_impl`) was directly iterating over `export.rows` instead of passing them through the `redact_rows(..., redact)` filter function like the other exporters (JSON, CSV, TSV). This caused sensitive file paths and module boundaries to be exposed in SBOMs even when `--redact paths` or `--redact all` were specified.

## 🔎 Evidence
File: `crates/tokmd-format/src/lib.rs:596`
Observation:
```rust
    // Apply redaction to rows before generating components
    let components: Vec<CycloneDxComponent> = export
        .rows
        .iter()
        .map(|row| {
```
Command showing the failure when testing redaction logic:
```text
test test_write_export_cyclonedx_honors_redact_mode ... FAILED
```

## 🧭 Options considered
### Option A (recommended)
- Route CycloneDX row processing through `redact_rows(&export.rows, redact)` as in `write_export_csv` and `write_export_jsonl`.
- Fits the repo and shard by resolving the data leakage and unifying the data pipeline around a single redaction entry point.
- Trade-offs: Structure/Velocity/Governance - Low risk, aligns identically with the JSON and CSV export logic.

### Option B
- Introduce a separate redaction pass specifically for CycloneDX component generation.
- When to choose: If SBOMs require specialized redaction models distinct from `FileRow` models.
- Trade-offs: Suboptimal, duplicates logic, and breaks consistency.

## ✅ Decision
Chose Option A. Applying the existing `redact_rows` iterator unifies behavior and immediately fixes the security boundary.

## 🧱 Changes made (SRP)
- `crates/tokmd-format/src/lib.rs`: Replaced `export.rows.iter()` with `redact_rows(&export.rows, redact)` in `write_export_cyclonedx_impl`.
- `crates/tokmd-format/tests/test_cyclonedx_redaction.rs`: Created contract test asserting `RedactMode::Paths` and `RedactMode::All` effectively short-hash output values.

## 🧪 Verification receipts
```text
$ cargo test -p tokmd-format --test test_cyclonedx_redaction
test test_write_export_cyclonedx_honors_redact_mode ... ok

$ cargo test -p tokmd-format
test result: ok. 193 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

## 🧭 Telemetry
- Change shape: Hardening
- Blast radius: CycloneDX SBOM formatting (trust boundary)
- Risk class: Low - Modifies only what is exported. Re-uses well-tested redaction primitives.
- Rollback: Revert `crates/tokmd-format/src/lib.rs` diff.
- Gates run: targeted `cargo test -p tokmd-format`, `cargo clippy -p tokmd-format -- -D warnings`, `cargo fmt -- --check`.

## 🗂️ .jules artifacts
- `.jules/runs/run_sentinel_redaction_1/envelope.json`
- `.jules/runs/run_sentinel_redaction_1/decision.md`
- `.jules/runs/run_sentinel_redaction_1/receipts.jsonl`
- `.jules/runs/run_sentinel_redaction_1/result.json`
- `.jules/runs/run_sentinel_redaction_1/pr_body.md`

## 🔜 Follow-ups
None.
