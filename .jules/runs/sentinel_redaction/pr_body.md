## 💡 Summary
Replaced the leniency around file extension preservation in redaction with a strict allowlist. This hardens the security boundary to ensure custom or sensitive file extensions (like `.passwd` or `.secret`) are fully redacted instead of leaked.

## 🎯 Why
Previously, `redact_path` preserved any alphanumeric file extension 8 characters or shorter. This meant paths like `config.secret` or `users.passwd` would be redacted to `<hash>.secret` or `<hash>.passwd`, leaking potentially sensitive structure and violating the security boundary guarantee.

## 🔎 Evidence
- **File:** `crates/tokmd-format/src/redact/mod.rs`
- **Finding:** `let ext = if ext.len() <= 8 && ext.chars().all(|c| c.is_ascii_alphanumeric())` allowed any alphanumeric short extension to bypass full redaction.
- **Receipt:** The `test_redact_path_leak` test previously used `"super_secret_password_123"` which bypassed the bug (because it was > 8 chars). Testing with `"passwd"` caused the test to fail and leak the extension.

## 🧭 Options considered
### Option A (recommended)
- **What it is:** Use an explicit whitelist of safe extensions (e.g., `"rs", "js", "ts", "json", "toml", "md"`).
- **Why it fits this repo and shard:** Fully closes the leakage vector by only preserving known, safe file extensions. Fits the `core-pipeline` shard and the `security-boundary` gate profile perfectly.
- **Trade-offs:** Might redact some obscure but benign extensions, which is a worthwhile trade-off for a security-boundary profile.

### Option B
- **What it is:** Only redact paths fully (no extensions).
- **When to choose it instead:** When the utility of extensions in redacted outputs is not needed.
- **Trade-offs:** Redacted output becomes significantly less useful for debugging or analysis because you can't tell what types of files were affected.

## ✅ Decision
Choose Option A. It provides the strongest security guarantee while retaining the utility of recognizing common file types in redacted SBOM and receipt outputs.

## 🧱 Changes made (SRP)
- `crates/tokmd-format/src/redact/mod.rs`
- `crates/tokmd-format/src/lib.rs`
- `crates/tokmd-format/tests/test_redaction_leak.rs`

## 🧪 Verification receipts
```text
running 1 test
test test_redact_path_leak ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```
```text
test result: ok. 131 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.17s
```

## 🧭 Telemetry
- **Change shape:** Patch (Hardening)
- **Blast radius:** Redacted paths in CycloneDX SBOMs and exported JSONs.
- **Risk class + why:** Low risk. It only strictly tightens the redaction behavior without changing default (non-redacted) functionality.
- **Rollback:** Safe to revert.
- **Gates run:** Built and tested successfully via `cargo test -p tokmd-format`.

## 🗂️ .jules artifacts
- `.jules/runs/sentinel_redaction/envelope.json`
- `.jules/runs/sentinel_redaction/decision.md`
- `.jules/runs/sentinel_redaction/receipts.jsonl`
- `.jules/runs/sentinel_redaction/result.json`
- `.jules/runs/sentinel_redaction/pr_body.md`

## 🔜 Follow-ups
None.
