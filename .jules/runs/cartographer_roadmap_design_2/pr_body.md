## 💡 Summary
Fixed a small factual drift in `docs/architecture.md` where the `fun` feature flag was incorrectly documented as mapping to `tokmd-format/fun` instead of `tokmd-core/fun`. Additionally, added a formal `architecture_docs` scope to `ci/proof.toml` to prevent CI failures from unknown files.

## 🎯 Why
The `docs/architecture.md` file serves as a reference for the workspace's feature flags. Stale or incorrect feature flag documentation can mislead contributors trying to understand the dependency boundaries. The CI pipeline strictly enforces known files mapped to scopes in `ci/proof.toml`. The architecture docs were unmapped.

## 🔎 Evidence
- `docs/architecture.md` (prior to fix)
- `crates/tokmd/Cargo.toml`
- Checked `crates/tokmd/Cargo.toml` and found: `fun = ["tokmd-analysis/fun", "tokmd-core/fun"]`
- Checked `docs/architecture.md` and found the stale mapping: `fun = ["tokmd-analysis/fun", "tokmd-format/fun"]`
- `cargo xtask affected` CI checks failed indicating an unknown file `docs/architecture.md`.

## 🧭 Options considered
### Option A (recommended)
- Update `docs/architecture.md` to match the actual implementation in `crates/tokmd/Cargo.toml`. Also map the file to a CI scope.
- Fits the `tooling-governance` shard by maintaining workspace documentation alignment.
- Trade-offs: Structure is improved, Velocity impact is negligible, Governance is maintained.

### Option B
- Ignore the drift and create a learning PR documenting that the docs are otherwise well-aligned with the v1.10.0 release.
- Choose this if no actionable drift could be found.
- Trade-offs: Misses fixing a real, easily fixable factual error.

## ✅ Decision
Option A, because it directly resolves a small but real factual drift between the architecture documentation and the shipped workspace features, and resolves CI workflow issues.

## 🧱 Changes made (SRP)
- `docs/architecture.md`: Updated `fun` feature flag mapping from `tokmd-format/fun` to `tokmd-core/fun`.
- `ci/proof.toml`: Added the `architecture_docs` scope.
- `xtask/tests/proof_policy_w90.rs`: Updated test to account for the new scope block count.

## 🧪 Verification receipts
```text
{"ts_utc": "2026-05-07T11:13:54Z", "phase": "investigation", "cwd": ".", "cmd": "cat crates/tokmd/Cargo.toml | grep -A 10 \"\\[features\\]\"", "status": "success", "summary": "Inspected Cargo.toml features", "artifacts": []}
{"ts_utc": "2026-05-07T11:13:54Z", "phase": "investigation", "cwd": ".", "cmd": "cat docs/architecture.md | grep -A 10 \"\\[features\\]\"", "status": "success", "summary": "Inspected architecture.md features", "artifacts": []}
{"ts_utc": "2026-05-07T11:13:54Z", "phase": "execution", "cwd": ".", "cmd": "cargo xtask affected --base origin/main --head HEAD --json", "status": "success", "summary": "Verified affected proof scope no longer fails on unknown file", "artifacts": []}
```

## 🧭 Telemetry
- Change shape: Documentation patch with CI mapping
- Blast radius (API / IO / docs / schema / concurrency / compatibility / dependencies): Docs only. No code or schema impact.
- Risk class + why: Low. Just a documentation typo fix and CI scoping.
- Rollback: `git checkout -- docs/architecture.md ci/proof.toml xtask/tests/proof_policy_w90.rs`
- Gates run: `cargo xtask docs --check`, `cargo fmt -- --check`, `cargo test -p xtask`

## 🗂️ .jules artifacts
- `.jules/runs/cartographer_roadmap_design_2/envelope.json`
- `.jules/runs/cartographer_roadmap_design_2/decision.md`
- `.jules/runs/cartographer_roadmap_design_2/receipts.jsonl`
- `.jules/runs/cartographer_roadmap_design_2/result.json`
- `.jules/runs/cartographer_roadmap_design_2/pr_body.md`

## 🔜 Follow-ups
None.
