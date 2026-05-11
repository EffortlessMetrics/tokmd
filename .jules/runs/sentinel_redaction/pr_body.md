## 💡 Summary
Fixed path redaction logic in `tokmd-format` to reliably collapse consecutive slashes (e.g. `//`) into a single slash (`/`). This ensures that identical logical paths passed to redaction boundaries deterministically yield the same hash output without leakage or discrepancies.

## 🎯 Why
The core-pipeline processes file paths and applies redaction (via `short_hash` and `redact_path`) to prevent leaking filesystem structures. Previously, `clean_path` normalized backwards slashes, `.` segments, and `./` prefixes, but it missed normalizing consecutive slashes. As a result, logically identical paths like `src/lib.rs` and `src//lib.rs` hashed differently. Given the `tokmd-format` redact module is a core security boundary, this caused test determinism failures when generating receipts that pass through differently formatted but functionally identical source paths.

## 🔎 Evidence
- File: `crates/tokmd-format/src/redact/mod.rs`
- Finding: `short_hash("src/lib.rs")` and `short_hash("src//lib.rs")` previously resulted in completely divergent BLAKE3 16-character string hashes.

## 🧭 Options considered
### Option A (recommended)
- **What it is**: Update the `clean_path` utility inside `crates/tokmd-format/src/redact/mod.rs` to replace all `//` occurrences with `/` prior to applying BLAKE3 hashing.
- **Why it fits this repo and shard**: Matches the exact goal of the core-pipeline and Sentinel persona—securing redaction mechanisms by standardizing deterministic outputs from identical data.
- **Trade-offs**:
  - Structure: Native fit, alongside other `clean_path` normalization steps.
  - Velocity: Extremely fast to implement with high payoff.
  - Governance: Solidifies the deterministic contract of receipts.

### Option B
- **What it is**: Do nothing and assume upstream parsers handle removing consecutive slashes.
- **When to choose it instead**: If paths provided were firmly guaranteed string-perfect by all call sites.
- **Trade-offs**: Retains silent determinism risks on the format boundary layer, forcing callers to implement redundant deduplication logic.

## ✅ Decision
Option A was selected. Modifying `clean_path` guarantees deterministic output regardless of slightly misformatted inputs upstream, satisfying the security-boundary gate constraints safely. Unit tests were added to prove the fix correctly hashes `//` and `///` exactly identical to `/`.

## 🧱 Changes made (SRP)
- `crates/tokmd-format/src/redact/mod.rs`: Added a `while` loop within `clean_path` to convert `//` to `/`. Added two test methods `test_short_hash_normalizes_double_slashes` and `test_redact_path_normalizes_double_slashes` to verify behavior.

## 🧪 Verification receipts
```text
{"timestamp": "2026-05-11T15:00:58Z", "command": "cargo test -p tokmd-format redact", "output": "test result: ok. 28 passed; 0 failed"}
{"timestamp": "2026-05-11T15:00:58Z", "command": "cargo test -p tokmd-types determinism", "output": "test result: ok. 1 passed; 0 failed"}
{"timestamp": "2026-05-11T15:00:58Z", "command": "cargo fmt -- --check", "output": "Completed with no errors"}
{"timestamp": "2026-05-11T15:00:58Z", "command": "cargo clippy -- -D warnings", "output": "Completed with no errors"}
{"timestamp": "2026-05-11T15:00:58Z", "command": "cargo build -p tokmd-format -p tokmd-types", "output": "Finished `dev` profile [unoptimized + debuginfo] target(s)"}
```

## 🧭 Telemetry
- Change shape: Hardening/bugfix
- Blast radius: API / tests (impacts redaction hash outputs for poorly-formatted inputs)
- Risk class: Low - strengthens normalization, reduces determinism errors.
- Rollback: Revert the `clean_path` adjustment in `redact/mod.rs`.
- Gates run: targeted cargo test, `cargo fmt -- --check`, `cargo clippy -- -D warnings`, and targeted cargo build.

## 🗂️ .jules artifacts
- `.jules/runs/sentinel_redaction/envelope.json`
- `.jules/runs/sentinel_redaction/decision.md`
- `.jules/runs/sentinel_redaction/receipts.jsonl`
- `.jules/runs/sentinel_redaction/result.json`
- `.jules/runs/sentinel_redaction/pr_body.md`

## 🔜 Follow-ups
None required.