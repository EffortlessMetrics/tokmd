## 💡 Summary
Hardened the `redact_path` function to prevent long arbitrary strings from being leaked as file extensions. The maximum length of an unredacted extension has been reduced from 8 to 5 characters.

## 🎯 Why
During path redaction, the system preserved file extensions to allow identifying the type of redacted files. However, an overly generous length limit (up to 8 characters) allowed sensitive or identifying strings located after a dot (e.g., `file.secret12`) to bypass redaction and leak into the final output. Tightening the bound ensures that only typical short extensions are preserved while arbitrary data is securely hashed.

## 🔎 Evidence
- File path: `crates/tokmd-format/src/redact/mod.rs`, `crates/tokmd-format/tests/test_redaction_leak.rs`
- Finding: `redact_path("file.secret12")` previously leaked the `secret12` portion. Now it successfully hashes the full string into a safe representation like `hash_of_path`.
- Receipt: `cargo test -p tokmd-format` proves the vulnerability is closed.

## 🧭 Options considered
### Option A (recommended)
- What it is: Reduce `ext.len() <= 8` to `ext.len() <= 5` in `redact_path`.
- Why it fits this repo and shard: It directly addresses the leakage in the `core-pipeline` formatting component.
- Trade-offs: Structure is minimally affected. Velocity is high. Governance limits rare 6-8 char extensions (e.g., `.action`) to a fully redacted state, which is an acceptable safety tradeoff.

### Option B
- What it is: Use a strict allowlist of known file extensions.
- When to choose it instead: When zero tolerance for unknown extensions is required.
- Trade-offs: High maintenance overhead for adding new extensions. Legitimate unknown extensions would be entirely obfuscated, reducing diagnostic utility.

## ✅ Decision
Proceeded with Option A to effectively seal the boundary with a low-risk heuristic that aligns with our Sentinel objective without overcomplicating maintenance.

## 🧱 Changes made (SRP)
- `crates/tokmd-format/src/redact/mod.rs`
- `crates/tokmd-format/tests/test_redaction_leak.rs`

## 🧪 Verification receipts
```text
{"cmd": "cargo test -p tokmd-format", "success": true}
{"cmd": "cargo fmt -p tokmd-format -- --check", "success": true}
{"cmd": "cargo clippy -p tokmd-format -- -D warnings", "success": true}
```

## 🧭 Telemetry
- Change shape: Hardening
- Blast radius: API (redaction mode outputs)
- Risk class: Low - affects redacted views, tightening output security.
- Rollback: Revert the PR and unblock the test case.
- Gates run: `cargo test`, `cargo fmt`, `cargo clippy`

## 🗂️ .jules artifacts
- `.jules/runs/sentinel_redaction/envelope.json`
- `.jules/runs/sentinel_redaction/decision.md`
- `.jules/runs/sentinel_redaction/receipts.jsonl`
- `.jules/runs/sentinel_redaction/result.json`
- `.jules/runs/sentinel_redaction/pr_body.md`

## 🔜 Follow-ups
None
