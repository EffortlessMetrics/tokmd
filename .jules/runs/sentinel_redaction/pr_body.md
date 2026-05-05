## 💡 Summary
Hardened `tokmd-format` path redaction to prevent arbitrary sensitive data leakage through long or non-standard file extensions.

## 🎯 Why
The `redact_path` function generates a hash for a path to hide its directory structure but explicitly preserves the file extension. However, `Path::extension()` returns an arbitrary string following the last dot. A file named `config.my_secret_token_is_12345` would result in the extension being `my_secret_token_is_12345`. `redact_path` blindly appended this extension to the output hash, resulting in a direct leakage of the token in plaintext.

## 🔎 Evidence
- File path: `crates/tokmd-format/src/redact/mod.rs`
- Observed behavior: `redact_path` preserves `ext` without validation, leaking arbitrary strings at the end of filenames.
- Verification test: A unit test (`test_redact_path_leak`) was written to verify the vulnerability and its remediation.

## 🧭 Options considered
### Option A (recommended)
- Prevent arbitrary data leakage by restricting the preserved file extension to alphanumeric ASCII characters and a maximum length of 8. If an extension violates this condition, it is discarded, and only the hash is returned.
- This fits the `core-pipeline` shard and `security-boundary` gate profile perfectly by closing an active leakage vector on a trust-bearing surface.
- Trade-offs: Minor reduction in utility if someone uses extremely long valid extensions, but vastly improves safety.

### Option B
- Hash the entire path and append a generic `.redacted` extension.
- This loses the ability to recognize common file types (like `.rs` or `.json`) in redacted outputs, removing utility.

## ✅ Decision
I have chosen Option A: Hardening `redact_path` to sanitize the file extension by restricting it to short, alphanumeric characters. It successfully prevents data leaks through arbitrary extensions while maintaining the usefulness of the feature.

## 🧱 Changes made (SRP)
- `crates/tokmd-format/src/redact/mod.rs`: Added validation to `ext` in `redact_path` to ensure it is alphanumeric and <= 8 characters long, defaulting to an empty extension otherwise.
- `crates/tokmd-format/tests/test_redaction_leak.rs`: Added a test verifying that redaction does not leak arbitrary extension data.

## 🧪 Verification receipts
```text
{"command": "cargo test -p tokmd-format test_redact_path_leak", "result": "ok"}
{"command": "cargo test -p tokmd-format", "result": "ok"}
```

## 🧭 Telemetry
- Change shape: Hardening fix
- Blast radius: Output generation (`tokmd-format`)
- Risk class: Low - Only modifies the logic used in the explicit `RedactMode::Paths` and `RedactMode::All` features.
- Rollback: Revert the PR.
- Gates run: targeted `cargo test -p tokmd-format`

## 🗂️ .jules artifacts
- `.jules/runs/sentinel_redaction/envelope.json`
- `.jules/runs/sentinel_redaction/decision.md`
- `.jules/runs/sentinel_redaction/receipts.jsonl`
- `.jules/runs/sentinel_redaction/result.json`
- `.jules/runs/sentinel_redaction/pr_body.md`

## 🔜 Follow-ups
None
