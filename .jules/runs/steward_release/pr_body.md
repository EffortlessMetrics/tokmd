## 💡 Summary
Consolidated the `toml` dependency into `[workspace.dependencies]` to fix drift. Previously `tokmd-gate` and `tokmd` pinned `1.1.2` directly, while `tokmd-settings` used `1.1`. This aligns all consumers to the workspace version.

## 🎯 Why
Release metadata and dependencies should be aligned to prevent version drift across crates in the workspace. Hardcoded dependencies lead to inconsistent dependency resolutions and violate dependency hygiene.

## 🔎 Evidence
Observed `toml = "1.1.2"` in `tokmd-gate` and `tokmd`, but `toml = "1.1"` in `tokmd-settings`. This drift violated metadata alignment.

## 🧭 Options considered
### Option A (recommended)
- Consolidate `toml` into `[workspace.dependencies]` with `version = "1.1.2"`.
- This ensures dependency hygiene and version consistency, perfectly matching the release governance focus of the Steward persona.
- Trade-offs: Structure is improved, no velocity impact, enforces workspace-wide governance.

### Option B
- Update versions individually in the respective crates without a workspace dependency.
- This creates future maintenance burden.
- Trade-offs: Increases long-term drift risk.

## ✅ Decision
Chose Option A to centralize the `toml` dependency in the workspace root, eliminating drift.

## 🧱 Changes made (SRP)
- `Cargo.toml`: Added `toml = "1.1.2"` to `[workspace.dependencies]`.
- `crates/tokmd-gate/Cargo.toml`: Updated to use `toml.workspace = true`.
- `crates/tokmd-settings/Cargo.toml`: Updated to use `toml.workspace = true`.
- `crates/tokmd/Cargo.toml`: Updated to use `{ workspace = true, optional = true }`.

## 🧪 Verification receipts
```text
$ cargo xtask version-consistency
Checking version consistency against workspace version 1.13.1

  ✓ Cargo crate versions match 1.13.1.
  ✓ Cargo workspace dependency versions match 1.13.1.
  ✓ Node package manifest versions match 1.13.1.
  ✓ No case-insensitive tracked-path collisions detected.
Version consistency checks passed.
```

## 🧭 Telemetry
- Change shape: Dependency hygiene / Manifest alignment
- Blast radius: Compilation (no logical changes)
- Risk class: Low risk. Uses same minor/patch version across all crates.
- Rollback: Revert the commit.
- Gates run: `cargo xtask version-consistency`, `cargo check --workspace`

## 🗂️ .jules artifacts
- `.jules/runs/steward_release/envelope.json`
- `.jules/runs/steward_release/decision.md`
- `.jules/runs/steward_release/receipts.jsonl`
- `.jules/runs/steward_release/result.json`
- `.jules/runs/steward_release/pr_body.md`

## 🔜 Follow-ups
None.
