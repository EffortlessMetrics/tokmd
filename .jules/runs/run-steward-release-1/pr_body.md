## 💡 Summary
Updated `NOW.md`, schema definitions, and design documentation to accurately reflect that `v1.10.0` is the current stable release. Replaced stale `1.9.0` version strings in JSON payload examples and operational status paragraphs.

## 🎯 Why
While the codebase manifests and CLI versions correctly reported `1.10.0`, several high-visibility documentation files (`NOW.md`, `SCHEMA.md`) still referenced the `1.9.0` release train as the current operational truth. This creates metadata/drift confusion for users referencing JSON payload examples or checking current roadmap status.

## 🔎 Evidence
- `docs/NOW.md` incorrectly claimed: "Updated after the 1.9.0 release. 1.9.0 is out..."
- `docs/SCHEMA.md`, `docs/design.md`, and `docs/sensor-report-v1.md` had hardcoded `"version": "1.9.0"` in example schema outputs.
- `CHANGELOG.md` and `Cargo.toml` correctly show `1.10.0` as the current release.

## 🧭 Options considered
### Option A (recommended)
- **What it is:** Update `docs/NOW.md` and JSON schema examples to use `1.10.0`.
- **Why it fits this repo and shard:** The Steward persona prioritizes fixing version-consistency drift and release metadata mismatch. Keeping docs aligned with the shipped reality is an explicit rule in `NOW.md`.
- **Trade-offs:** Governance – slightly increases churn on docs for every release, but maintains crucial accuracy.

### Option B
- **What it is:** Do nothing and create a learning PR, as `cargo xtask version-consistency` passed.
- **When to choose it instead:** If the docs drift was minor or if the 1.9.0 references were purely historical context (like those correctly left untouched in `docs/implementation-plan.md`).
- **Trade-offs:** Leaves highly visible operational docs in a stale state, violating the intent of `NOW.md`.

## ✅ Decision
Option A was chosen to eliminate explicit version-consistency drift in operational documentation, fulfilling the Steward mandate for release hygiene.

## 🧱 Changes made (SRP)
- `docs/NOW.md`
- `docs/SCHEMA.md`
- `docs/design.md`
- `docs/sensor-report-v1.md`

## 🧪 Verification receipts
```text
$ cargo xtask docs --check
Documentation is up to date.
     Running `target/debug/xtask docs --check`

$ cargo xtask version-consistency
Checking version consistency against workspace version 1.10.0

  ✓ Cargo crate versions match 1.10.0.
  ✓ Cargo workspace dependency versions match 1.10.0.
  ✓ Node package manifest versions match 1.10.0.
  ✓ No case-insensitive tracked-path collisions detected.
Version consistency checks passed.
```

## 🧭 Telemetry
- Change shape: Documentation update
- Blast radius: Docs / schema examples (no behavioral changes)
- Risk class: Low - pure textual documentation fixes for an existing release.
- Rollback: Revert the documentation updates.
- Gates run: `cargo xtask docs --check`, `cargo xtask version-consistency`, `cargo xtask publish --plan --verbose`

## 🗂️ .jules artifacts
- `.jules/runs/run-steward-release-1/envelope.json`
- `.jules/runs/run-steward-release-1/decision.md`
- `.jules/runs/run-steward-release-1/receipts.jsonl`
- `.jules/runs/run-steward-release-1/result.json`
- `.jules/runs/run-steward-release-1/pr_body.md`

## 🔜 Follow-ups
None.
