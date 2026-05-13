## 💡 Summary
Hardened path redaction to preserve explicitly known compound suffixes without widening arbitrary suffix leakage. `.tar.gz` is now preserved as a unit, unknown safe-looking chains still collapse to the final extension, and unsafe final suffixes strip all suffix output.

## 🎯 Why
The original `redact_path` relied solely on `Path::extension()` which only retrieved the final extension. That made safe compound archive suffixes like `.tar.gz` less useful in redacted receipts. The replacement keeps compound suffixes explicit and narrow instead of preserving every safe-looking dotted segment.

## 🔎 Evidence
- `crates/tokmd-format/src/redact/mod.rs`
- `crates/tokmd-format/tests/test_redaction_leak.rs`
- `cargo test -p tokmd-format redaction --verbose`

## 🧭 Options considered
### Option A (recommended)
- Preserve explicitly known safe compound suffixes such as `.tar.gz`, otherwise keep only the final allowlisted extension.
- Fits this repo and shard by protecting the data boundary.
- **Trade-offs:** Structure: High signal boundary hardening. Velocity: Easy to audit. Governance: Improves security guarantees.

### Option B
- Use `Path::new().extension()`, retrieving only the final extension.
- Choose it when simple extension matching is enough.
- **Trade-offs:** Drops useful compound archive suffix context such as `.tar.gz`.

## ✅ Decision
Option A. Correctly implements secure path redaction while preserving semantic archive suffixes like `.tar.gz`, avoiding arbitrary safe-chain preservation like `.json.rs`, and hiding unsafe suffixes like `.rs.bak`.

## 🧱 Changes made (SRP)
- `crates/tokmd-format/src/redact/mod.rs`: Updated `redact_path` to preserve explicit safe compound suffixes before falling back to the final extension.
- `crates/tokmd-format/src/redact/extensions.rs`: Added a private compound suffix policy for `.tar.gz`.
- `crates/tokmd-format/tests/test_redaction_leak.rs`: Verified `.tar.gz`, unknown safe chains, and unsafe final suffixes.

## 🧪 Verification receipts
```text
cargo test -p tokmd-format redaction --verbose
test result: ok
cargo test -p tokmd-format scan_args --verbose
test result: ok
cargo clippy -p tokmd-format --all-targets -- -D warnings
finished successfully
cargo xtask proof-policy --check
Proof policy OK
```

## 🧭 Telemetry
- Change shape: Core formatting update
- Blast radius: Output receipts file paths
- Risk class + why: Low, contained to pure formatter.
- Rollback: Revert PR.
- Gates run: `cargo test`

## 🗂️ .jules artifacts
- `.jules/runs/sentinel_redaction/envelope.json`
- `.jules/runs/sentinel_redaction/decision.md`
- `.jules/runs/sentinel_redaction/receipts.jsonl`
- `.jules/runs/sentinel_redaction/result.json`
- `.jules/runs/sentinel_redaction/pr_body.md`

## 🔜 Follow-ups
None.
