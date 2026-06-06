## 💡 Summary
Updated `ROADMAP.md` to reflect the shipped reality of `v1.12.0` and `v1.11.0`. The roadmap had drifted by leaving `v1.12.x` in "Future Horizons" despite `v1.12.0` already shipping on 2026-06-04 with the Bun UB preset and swarm workbench.

## 🎯 Why
The `governance-release` shard requires us to keep roadmap and design docs aligned with the shipped reality. `CHANGELOG.md` correctly recorded `v1.12.0` landing and `v1.11.0`'s true focus (evidence surfaces and review packets), while the `ROADMAP.md` table and "Future Horizons" section had grown stale, misleading contributors on what is current vs. what is planned.

## 🔎 Evidence
Minimal proof:
- `ROADMAP.md` still listed `### v1.12.x — Selection-First Product and Evidence Work` under `## Future Horizons`.
- `CHANGELOG.md` explicitly logged `## [1.12.0] - 2026-06-04` containing `bun-ub` and `tokmd-swarm`.
- `ROADMAP.md`'s Status Summary table omitted `1.12.0` and incorrectly summarized `1.11.0` solely as "Browser runtime polish" instead of capturing its evidence consumption work.

## 🧭 Options considered
### Option A (recommended)
- what it is: Update `ROADMAP.md` to reflect that `v1.12.0` has actually shipped, marking it `Complete` in the Status Summary table, and moving its detail section out of "Future Horizons" into "Completed: v1.12.0". It also updates the `v1.11.0` status table entry to reflect the true shipped reality.
- why it fits this repo and shard: The `governance-release` shard explicitly targets "Release and governance surfaces such as docs drift, publish plan, version consistency, and metadata alignment."
- trade-offs: Structure: Keeps the roadmap in sync with the changelog. Velocity: Small change, but removes confusion. Governance: High alignment with truth sources.

### Option B
- what it is: Update `docs/architecture.md` to document the `bun-ub` preset.
- when to choose it instead: If the roadmap was already accurate but the architectural implications were missing.
- trade-offs: More exploratory, might overlap with other specs. The roadmap drift is a much clearer target.

## ✅ Decision
Option A. The `ROADMAP.md` drift is explicit and is exactly the kind of documentation artifact drift Cartographer targets.

## 🧱 Changes made (SRP)
- `ROADMAP.md`: Updated Status Summary table for `1.11.0` and added `1.12.0`. Moved `1.12.0` out of Future Horizons into Completed.

## 🧪 Verification receipts
```text
$ cargo xtask publish --plan --verbose
$ cargo xtask version-consistency
  ✓ Cargo crate versions match 1.12.0.
  ✓ Cargo workspace dependency versions match 1.12.0.
  ✓ Node package manifest versions match 1.12.0.
  ✓ No case-insensitive tracked-path collisions detected.
$ cargo xtask docs --check
Documentation is up to date.
$ cargo fmt -- --check
$ cargo clippy -- -D warnings
```

## 🧭 Telemetry
- Change shape: Documentation update.
- Blast radius: Docs.
- Risk class: Low, aligns text with code reality.
- Rollback: Revert markdown changes.
- Gates run: `publish-surface`, `version-consistency`, `docs`, `fmt`, `clippy`

## 🗂️ .jules artifacts
- `.jules/runs/run_carto_builder_01/envelope.json`
- `.jules/runs/run_carto_builder_01/decision.md`
- `.jules/runs/run_carto_builder_01/receipts.jsonl`
- `.jules/runs/run_carto_builder_01/result.json`
- `.jules/runs/run_carto_builder_01/pr_body.md`

## 🔜 Follow-ups
None.
