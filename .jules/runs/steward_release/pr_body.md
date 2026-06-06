## 💡 Summary
Updated remaining mentions of `1.11.0` to `1.12.0` in the repository documentation and `README.md`.

## 🎯 Why
The repository workspace version is `1.12.0`, but many workflow examples, action quickstarts, and design documents still referenced `1.11.0` and `1.11.0-rc.1`. This fixes the drift to ensure the upcoming release doesn't deploy confusing examples.

## 🔎 Evidence
- File paths: `README.md`, `docs/github-action.md`, `docs/start-here.md`, `docs/SCHEMA.md`, etc.
- Observed behavior: `grep -rn "1.11.0" docs/ README.md` returned numerous hits in example workflows and schema documents.
- Receipt: `cargo run -p xtask -- version-consistency` confirms the workspace is `1.12.0`.

## 🧭 Options considered
### Option A (recommended)
- what it is: Update `1.11.0` and `1.11.0-rc.1` to `1.12.0` and `1.12.0-rc.1` across docs and `README.md`.
- why it fits this repo and shard: Directly addresses release/governance drift for the current release cycle.
- trade-offs: Structure (none) / Velocity (high) / Governance (aligns user-facing examples with the new release).

### Option B
- what it is: Leave documentation targeting the older release.
- when to choose it instead: If `1.12.0` is an internal-only release with no public facing API/Action changes (not the case).
- trade-offs: Increases support friction for users copying outdated version tags.

## ✅ Decision
Option A. It's a low-risk, high-confidence release hygiene fix.

## 🧱 Changes made (SRP)
- `README.md`
- `docs/SCHEMA.md`
- `docs/action-quickstart.md`
- `docs/browser-to-native.md`
- `docs/design.md`
- `docs/github-action.md`
- `docs/install-and-try.md`
- `docs/install.md`
- `docs/recipes.md`
- `docs/sensor-report-v1.md`
- `docs/start-here.md`
- `docs/tokmd-in-cockpit.md`

## 🧪 Verification receipts
```text
$ cargo run -p xtask -- docs --check
Documentation is up to date.
doc artifacts ok: 2 required doc(s), 53 family file(s), 1 active goal(s), 18 spec-index artifact(s), 0 spec-index lane(s)

$ cargo run -p xtask -- version-consistency
Checking version consistency against workspace version 1.12.0
  ✓ Cargo crate versions match 1.12.0.
  ✓ Cargo workspace dependency versions match 1.12.0.
  ✓ Node package manifest versions match 1.12.0.
  ✓ No case-insensitive tracked-path collisions detected.
Version consistency checks passed.
```

## 🧭 Telemetry
- Change shape: Documentation update
- Blast radius: docs
- Risk class: Low
- Rollback: `git revert`
- Gates run: `cargo xtask docs --check`, `cargo xtask version-consistency`

## 🗂️ .jules artifacts
- `.jules/runs/steward_release/envelope.json`
- `.jules/runs/steward_release/decision.md`
- `.jules/runs/steward_release/receipts.jsonl`
- `.jules/runs/steward_release/result.json`
- `.jules/runs/steward_release/pr_body.md`

## 🔜 Follow-ups
None.
