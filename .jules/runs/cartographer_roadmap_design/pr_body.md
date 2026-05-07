## 💡 Summary
Fixed a small factual drift in `docs/architecture.md` where the `fun` feature flag was incorrectly documented as mapping to `tokmd-format/fun`. It has been updated to correctly map to `tokmd-core/fun`, matching the shipped reality in `crates/tokmd/Cargo.toml`.

## 🎯 Why
The `docs/architecture.md` file serves as a reference for the workspace's feature flags. Stale or incorrect feature flag documentation can mislead contributors trying to understand the dependency boundaries.

## 🔎 Evidence
- `docs/architecture.md` (prior to fix)
- `crates/tokmd/Cargo.toml`
- Checked `crates/tokmd/Cargo.toml` and found: `fun = ["tokmd-analysis/fun", "tokmd-core/fun"]`
- Checked `docs/architecture.md` and found the stale mapping: `fun = ["tokmd-analysis/fun", "tokmd-format/fun"]`

## 🧭 Options considered
### Option A (recommended)
- Update `docs/architecture.md` to match the actual implementation in `crates/tokmd/Cargo.toml`.
- Fits the `tooling-governance` shard by maintaining workspace documentation alignment.
- Trade-offs: Structure is improved, Velocity impact is negligible, Governance is maintained.

### Option B
- Ignore the drift and create a learning PR documenting that the docs are otherwise well-aligned with the v1.10.0 release.
- Choose this if no actionable drift could be found.
- Trade-offs: Misses fixing a real, easily fixable factual error.

## ✅ Decision
Option A, because it directly resolves a small but real factual drift between the architecture documentation and the shipped workspace features.

## 🧱 Changes made (SRP)
- `docs/architecture.md`: Updated `fun` feature flag mapping from `tokmd-format/fun` to `tokmd-core/fun`.

## 🧪 Verification receipts
```text
{"ts_utc": "2026-05-07T11:13:54Z", "phase": "investigation", "cwd": ".", "cmd": "cat crates/tokmd/Cargo.toml | grep -A 10 \"\\[features\\]\"", "status": "success", "summary": "Inspected Cargo.toml features", "artifacts": []}
{"ts_utc": "2026-05-07T11:13:54Z", "phase": "investigation", "cwd": ".", "cmd": "cat docs/architecture.md | grep -A 10 \"\\[features\\]\"", "status": "success", "summary": "Inspected architecture.md features", "artifacts": []}
{"ts_utc": "2026-05-07T11:13:54Z", "phase": "execution", "cwd": ".", "cmd": "git diff docs/architecture.md", "status": "success", "summary": "Verified drift fix", "artifacts": ["docs/architecture.md"]}
```

## 🧭 Telemetry
- Change shape: Documentation patch
- Blast radius (API / IO / docs / schema / concurrency / compatibility / dependencies): Docs only. No code or schema impact.
- Risk class + why: Low. Just a documentation typo fix.
- Rollback: `git checkout -- docs/architecture.md`
- Gates run: `cargo xtask docs --check`, `cargo fmt -- --check`, `cargo test -p xtask`

## 🗂️ .jules artifacts
- `.jules/runs/cartographer_roadmap_design/envelope.json`
- `.jules/runs/cartographer_roadmap_design/decision.md`
- `.jules/runs/cartographer_roadmap_design/receipts.jsonl`
- `.jules/runs/cartographer_roadmap_design/result.json`
- `.jules/runs/cartographer_roadmap_design/pr_body.md`

## 🔜 Follow-ups
None.
