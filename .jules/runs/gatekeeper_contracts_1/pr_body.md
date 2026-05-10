## 💡 Summary
Fixed schema drift by aligning `crates/tokmd/schemas/handoff.schema.json` with its canonical documentation source in `docs/`.

## 🎯 Why
The source schema for handoff (`crates/tokmd/schemas/handoff.schema.json`) was out of sync with the true schema definition in `docs/handoff.schema.json`. Specifically, it was missing recently added definitions like `smart_excluded_files`, `token_estimation`, and `code_audit`. This misalignment violates the `contracts-determinism` gate profile, which requires schema surfaces to remain exactly in sync with their code definitions.

## 🔎 Evidence
Minimal proof:
- `crates/tokmd/schemas/handoff.schema.json`
- `docs/handoff.schema.json`
- Diff showed the crates schema missing v5 additive fields:
```text
diff -u crates/tokmd/schemas/handoff.schema.json docs/handoff.schema.json
--- crates/tokmd/schemas/handoff.schema.json
+++ docs/handoff.schema.json
@@ -21,6 +21,7 @@
     "included_files",
     "excluded_paths",
     "excluded_patterns",
+    "smart_excluded_files",
```

## 🧭 Options considered
### Option A (recommended)
- Align `crates/tokmd/schemas/handoff.schema.json` by overwriting it with the updated `docs/handoff.schema.json`.
- Why it fits: The Gatekeeper persona protects contract-bearing surfaces and prevents schema drift.
- Trade-offs: Structure is improved by keeping contracts unified; Velocity is high; Governance is locked in for deterministic documentation.

### Option B
- Leave the JSON schema in crates as is, but try to only update markdown.
- When to choose it: Never, as the JSON schema contract itself needs to be perfectly aligned for external consumers.
- Trade-offs: Fails to solve the schema validation gap.

## ✅ Decision
I chose Option A. Copying the schema directly from the documentation folder guarantees that our internal schemas match our published source of truth exactly.

## 🧱 Changes made (SRP)
- `crates/tokmd/schemas/handoff.schema.json`: Synced with `docs/handoff.schema.json` to include `smart_excluded_files`, `token_estimation`, and `code_audit`.

## 🧪 Verification receipts
```text
cargo test -p xtask
test result: ok. 16 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s (schema valid)
```

## 🧭 Telemetry
- Change shape: Content alignment
- Blast radius: `crates/tokmd/schemas/` schema
- Risk class: Low - pure schema sync update, no rust code changes.
- Rollback: `git checkout crates/tokmd/schemas/handoff.schema.json`
- Gates run: `cargo test -p xtask`, `cargo xtask docs --check`

## 🗂️ .jules artifacts
- `.jules/runs/gatekeeper_contracts_1/envelope.json`
- `.jules/runs/gatekeeper_contracts_1/decision.md`
- `.jules/runs/gatekeeper_contracts_1/receipts.jsonl`
- `.jules/runs/gatekeeper_contracts_1/result.json`
- `.jules/runs/gatekeeper_contracts_1/pr_body.md`

## 🔜 Follow-ups
None.
