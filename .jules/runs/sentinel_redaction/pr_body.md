## 💡 Summary
This is a learning PR. The intended boundary hardening for `redact_path` extension preservation was gracefully aborted because it was superseded by another merged PR (#1553) which implemented a strict allowlist.

## 🎯 Why
The `redact_path` function previously preserved alphanumeric extensions up to 8 characters, introducing a leakage vector where sensitive 8-character strings (e.g., `pass1234`) could be exposed in redacted outputs. This run aimed to harden this boundary. However, the work was superseded by #1553, necessitating a learning PR rather than a redundant code patch.

## 🔎 Evidence
- **File:** `crates/tokmd-format/src/redact/mod.rs`
- **Observed behavior:** PR review comment explicitly noted: "Superseded by #1553, which chose explicit allowlisted extension preservation over the weaker strict-short-alpha fallback and landed with targeted redaction tests."
- **Receipts:** Reverted my patch in `crates/tokmd-format/src/redact/mod.rs` to keep the code unchanged.

## 🧭 Options considered
### Option A (recommended)
- Create a learning PR to document the workflow edge case where a patch is superseded.
- Fits the repo style because it avoids forcing a redundant fix or artificial code changes when the primary target is already resolved.
- Trade-offs: No code is patched in this run, but repository knowledge and friction items are accurately tracked.

### Option B
- Force a different, lower-value refactor inside the shard.
- This is explicitly against the rules ("Do not force a fake fix").

## ✅ Decision
Option A was chosen. I gracefully aborted the redundant fix and created a learning PR containing a friction item and persona note to document the occurrence.

## 🧱 Changes made (SRP)
- `.jules/friction/open/superseded_pr.md`
- `.jules/personas/sentinel/notes/redaction.md`

## 🧪 Verification receipts
```text
git checkout crates/tokmd-format/src/redact/mod.rs
```

## 🧭 Telemetry
- Change shape: Learning PR
- Blast radius: None (documentation only).
- Risk class: Zero risk.
- Rollback: N/A
- Gates run: N/A

## 🗂️ .jules artifacts
- `.jules/runs/sentinel_redaction/envelope.json`
- `.jules/runs/sentinel_redaction/decision.md`
- `.jules/runs/sentinel_redaction/receipts.jsonl`
- `.jules/runs/sentinel_redaction/result.json`
- `.jules/runs/sentinel_redaction/pr_body.md`
- `.jules/friction/open/superseded_pr.md`
- `.jules/personas/sentinel/notes/redaction.md`

## 🔜 Follow-ups
See friction item `.jules/friction/open/superseded_pr.md`.
