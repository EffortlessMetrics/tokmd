## 💡 Summary

If this is a learning PR, say so plainly. I will report on Option A / Option B, and friction.

## 🎯 Why

I am the Auditor 🧾 persona targeting `deps-hygiene` in `core-pipeline`.
I investigated the crates in `crates/tokmd-types`, `crates/tokmd-scan`, `crates/tokmd-model`, and `crates/tokmd-format`.
I did not find any dependencies to remove or features to tighten that wouldn't involve unnecessary churn or break the build.
Since `cargo-machete` and `cargo tree` reported no unused dependencies or redundant features, and the dependencies are properly minimized already, I have no justified patch.

## 🧭 Options considered

### Option A (recommended)
- what it is: Produce a learning PR.
- why it fits this repo and shard: Avoids forcing a fake fix or churn on a stable core pipeline when no actionable dependency hygiene drift exists.
- trade-offs: Structure / Velocity / Governance: Lowers churn, maintains high signal-to-noise ratio in commits.

### Option B
- what it is: Force a patch-level bump or attempt to remove a dependency used by a test, risking breakage.
- when to choose it instead: Never, unless a known CVE forces an update (which `cargo deny` didn't report).
- trade-offs: Increases churn and risk of regressions for zero benefit.

## ✅ Decision

I choose Option A: generate a learning PR. The core pipeline dependencies are extremely clean, and forcing a change would be counterproductive.

## 🧱 Changes made (SRP)

- Generated learning PR artifacts.
- No source code or manifest modifications made.

## 🧪 Verification receipts

```text
# checked for unused dependencies
cargo machete crates/tokmd-types/ crates/tokmd-scan/ crates/tokmd-model/ crates/tokmd-format/
# checked feature drift
cargo tree -e features -p tokmd-types
cargo tree -e features -p tokmd-scan
cargo tree -e features -p tokmd-model
cargo tree -e features -p tokmd-format
# ran gates
cargo deny --all-features check
cargo xtask version-consistency
```

## 🧭 Telemetry

- Change shape: Learning PR
- Blast radius (API / IO / docs / schema / concurrency / compatibility / dependencies): None
- Risk class + why: None (No code changes)
- Rollback: N/A
- Gates run: `cargo machete`, `cargo deny --all-features check`, `cargo xtask version-consistency`

## 🗂️ .jules artifacts

- `.jules/runs/auditor_core_manifests/envelope.json`
- `.jules/runs/auditor_core_manifests/decision.md`
- `.jules/runs/auditor_core_manifests/receipts.jsonl`
- `.jules/runs/auditor_core_manifests/result.json`
- `.jules/runs/auditor_core_manifests/pr_body.md`
- `.jules/friction/open/auditor_no_drift_core_pipeline.md`

## 🔜 Follow-ups

- See friction item regarding zero drift in core pipeline dependency hygiene.