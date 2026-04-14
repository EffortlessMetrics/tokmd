## 💡 Summary
This is a learning PR. The `auditor_core_manifests` run investigated `crates/tokmd-types`, `crates/tokmd-scan`, `crates/tokmd-model`, and `crates/tokmd-format` for dependency hygiene improvements but found no actionable drift. The dependencies are properly minimized and well-aligned with their requirements.

## 🎯 Why
The Auditor persona focuses on dependency hygiene, specifically targeting unused dependencies or redundant features. After running discovery tools and inspecting the manifests, no justifiable code patches were found, so a learning PR is submitted instead of forcing fake churn.

## 🔎 Evidence
- `crates/tokmd-types/Cargo.toml`, `crates/tokmd-scan/Cargo.toml`, `crates/tokmd-model/Cargo.toml`, `crates/tokmd-format/Cargo.toml`
- `cargo machete` reported no unused dependencies.
- `cargo deny --all-features check` reported `advisories ok, bans ok, licenses ok, sources ok`.
- `cargo xtask version-consistency` passed.

## 🧭 Options considered

### Option A (recommended)
- what it is: Produce a learning PR.
- why it fits this repo and shard: Avoids forcing a fake fix or churn on a stable core pipeline when no actionable dependency hygiene drift exists.
- trade-offs: Structure / Velocity / Governance: Lowers churn, maintains high signal-to-noise ratio in commits.

### Option B
- what it is: Force a patch-level bump or attempt to remove a dependency used by a test, risking breakage.
- when to choose it instead: Never, unless a known CVE forces an update.
- trade-offs: Increases churn and risk of regressions for zero benefit.

## ✅ Decision
Option A was chosen. The core pipeline dependencies are extremely clean, and forcing a change would be counterproductive.

## 🧱 Changes made (SRP)
- Generated learning PR artifacts under `.jules/runs/auditor_core_manifests/`.
- Created a friction item `.jules/friction/open/auditor_no_drift_core_pipeline.md`.

## 🧪 Verification receipts

```text
cargo-machete didn't find any unused dependencies in crates/tokmd-types. Good job!
cargo-machete didn't find any unused dependencies in crates/tokmd-scan. Good job!
cargo-machete didn't find any unused dependencies in crates/tokmd-model. Good job!
cargo-machete didn't find any unused dependencies in crates/tokmd-format. Good job!
```

```text
advisories ok, bans ok, licenses ok, sources ok
```

```text
Checking version consistency against workspace version 1.9.0

  ✓ Cargo crate versions match 1.9.0.
  ✓ Cargo workspace dependency versions match 1.9.0.
  ✓ Node package manifest versions match 1.9.0.
  ✓ No case-insensitive tracked-path collisions detected.
Version consistency checks passed.
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
- Check the generated friction item.