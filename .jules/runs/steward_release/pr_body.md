## 💡 Summary
Fixed workspace version consistency drift in `tokmd-cockpit` by standardizing its local path dependency on `tokmd-analysis`.

## 🎯 Why
During release preparation, manual inline versions inside workspace members for local paths pose a drift risk (e.g., `version = "1.11.0"` found when the workspace was `1.13.1`). By letting cargo inherit versions implicitly for local path dependencies, we enforce better release consistency and ensure `cargo xtask version-consistency` accurately passes across the entire project structure.

## 🔎 Evidence
- `crates/tokmd-cockpit/Cargo.toml` contained `version = "1.11.0"` for the `tokmd-analysis` dependency.
- Replaced with standard path dependency without inline version.
- Verified workspace building via `cargo check` and `cargo xtask version-consistency`.

## 🧭 Options considered
### Option A (recommended)
- Remove `version = "1.11.0"` from `tokmd-cockpit/Cargo.toml` for the `tokmd-analysis` dependency.
- Fixes the version drift risk securely without affecting behavior since the dependency is sourced via local path `path = "../tokmd-analysis"`.
- Trade-offs: Structure / Velocity / Governance: Improves governance and maintainability.

### Option B
- Change `version = "1.11.0"` to `version = "1.13.1"`.
- More maintenance burden on future versions and keeps duplicate information.
- Trade-offs: Requires a change every release manually for path dependencies, bypassing workspace capabilities.

## ✅ Decision
Option A. We remove the hardcoded inline version to rely on cargo path semantics, preventing future release drift.

## 🧱 Changes made (SRP)
- `crates/tokmd-cockpit/Cargo.toml`

## 🧪 Verification receipts
```text
$ cargo xtask version-consistency
Checking version consistency against workspace version 1.13.1

  ✓ Cargo crate versions match 1.13.1.
  ✓ Cargo workspace dependency versions match 1.13.1.
  ✓ Node package manifest versions match 1.13.1.
  ✓ No case-insensitive tracked-path collisions detected.
Version consistency checks passed.

$ cargo xtask docs --check
Documentation is up to date.
doc artifacts ok: 2 required doc(s), 54 family file(s), 1 active goal(s), 19 spec-index artifact(s), 0 spec-index lane(s)

$ cargo check -p tokmd-cockpit
    Checking tokmd-cockpit v1.13.1 (/app/crates/tokmd-cockpit)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 20.55s
```

## 🧭 Telemetry
- Change shape: Metadata update.
- Blast radius: None (internal cargo manifest change).
- Risk class: Low - release metadata fix.
- Rollback: Revert `Cargo.toml`.
- Gates run: `cargo check`, `cargo xtask version-consistency`, `cargo xtask docs --check`, `cargo xtask publish --plan`.

## 🗂️ .jules artifacts
- `.jules/runs/steward_release/envelope.json`
- `.jules/runs/steward_release/decision.md`
- `.jules/runs/steward_release/receipts.jsonl`
- `.jules/runs/steward_release/result.json`
- `.jules/runs/steward_release/pr_body.md`

## 🔜 Follow-ups
None.
