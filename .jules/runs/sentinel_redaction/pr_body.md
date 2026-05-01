## 💡 Summary
Hardened `redact_path` behavior in `tokmd-format` to only preserve file extensions up to 4 characters containing strictly ASCII alphabetical characters. This tightens the trust boundary by preventing longer alphanumeric segments from leaking into redacted artifacts.

## 🎯 Why
The previous `redact_path` implementation allowed extensions up to 8 characters containing any ASCII alphanumeric character (e.g. `file.pass1234`). This presented a leakage boundary risk where sensitive alphanumeric strings disguised as file extensions could be directly exposed in redacted logs and output receipts instead of being fully hashed.

## 🔎 Evidence
- **File:** `crates/tokmd-format/src/redact/mod.rs`
- **Observed behavior:** `redact_path` preserved alphanumeric extensions up to 8 characters long, exposing potential secrets.
- **Verification receipt:** Running a custom test case `let path = format!("file.{}", "pass1234");` leaked the "pass1234" segment in the redacted output.

## 🧭 Options considered
### Option A (recommended)
- Tighten the `redact_path` condition to `ext.len() <= 4 && ext.chars().all(|c| c.is_ascii_alphabetic())`.
- Fits the `core-pipeline` shard securely by restricting potential leakage while still identifying code file boundaries (e.g., `.rs`, `.md`, `.js`, `.py`, `.cpp`, `.json`).
- Trade-offs: Non-alphabetic extensions or longer ones (e.g. `f90`, `mp4`, `swift`) lose their suffix and become purely hashed. For redaction, pure hashing is the fail-safe expected behavior.

### Option B
- Maintain an explicit static list of all allowed extensions (e.g., `["rs", "js", "cpp", ...]`).
- Offers perfect safety but comes with high maintenance overhead as new languages are supported in `tokmd`.

## ✅ Decision
Option A was chosen. Restricting extensions to 4 alphabetical characters or less prevents long alphanumeric token leaks while covering almost all common source file extensions effortlessly.

## 🧱 Changes made (SRP)
- `crates/tokmd-format/src/redact/mod.rs`

## 🧪 Verification receipts
```text
cargo test -p tokmd-format && cargo fmt -- --check && cargo clippy -- -D warnings
test result: ok. 16 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.36s
test result: ok. 23 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s
test result: ok. 35 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.10s
test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.11s
...
```

## 🧭 Telemetry
- Change shape: Boundary hardening (redact condition tightening).
- Blast radius: Output / schema. The change affects only the format of the redacted path output (pure hashes vs hashes with extensions). It is fully compatible with determinism and standard `tokmd-types` serialization tests.
- Risk class: Low risk. Hardens trust boundary. Unrecognized extensions correctly fall back to pure hashes.
- Rollback: Revert the condition in `redact_path`.
- Gates run: `cargo check`, `cargo test -p tokmd-format`, `cargo test -p tokmd`, `cargo clippy`, `cargo fmt`.

## 🗂️ .jules artifacts
- `.jules/runs/sentinel_redaction/envelope.json`
- `.jules/runs/sentinel_redaction/decision.md`
- `.jules/runs/sentinel_redaction/receipts.jsonl`
- `.jules/runs/sentinel_redaction/result.json`
- `.jules/runs/sentinel_redaction/pr_body.md`

## 🔜 Follow-ups
None.
