## 💡 Summary
Moved the "Language Bindings (FFI)" documentation from the planned `v2.0` section to the completed `v1.4.0` section in the roadmap. This correctly aligns the roadmap with shipped reality and removes factual drift.

## 🎯 Why
The `ROADMAP.md` incorrectly categorized Python (`tokmd-python`) and Node.js (`tokmd-node`) bindings under `### v2.0 — Platform Evolution`, which is still marked as a future planned milestone. However, according to `CHANGELOG.md`, both language bindings were successfully implemented and released in `v1.4.0` (2026-01-31). Fixing this drift ensures contributors and users clearly understand what features are currently stable and what remains in the v2.0 horizon.

## 🔎 Evidence
- `ROADMAP.md`: Showed "Language Bindings (FFI)" under `### v2.0 — Platform Evolution`.
- `CHANGELOG.md` (lines 448-453):
  ```text
  ## [1.4.0] - 2026-01-31
  ### Added
  - **Node.js Bindings**: New `tokmd-node` crate with napi-rs bindings for npm
  - **Python Bindings**: New `tokmd-python` crate with PyO3 bindings for PyPI
  ```
- Command `grep -n Python CHANGELOG.md` verified bindings landed in v1.4.0.

## 🧭 Options considered
### Option A (recommended)
- Move the "Language Bindings (FFI)" block to the `v1.4.0` completed section.
- Why it fits: Accurately reflects historical shipments and immediately resolves the roadmap drift.
- Trade-offs: Minor structural change to `ROADMAP.md` sections, slightly shortening the perceived scope of v2.0 but improving Governance accuracy.

### Option B
- Delete the FFI bindings section completely from the roadmap.
- When to choose: If we only wanted the roadmap to reflect future items and completely delete past history.
- Trade-offs: We lose the historical context of when this major milestone was completed, which is inconsistent with how other completed v1.x milestones are tracked in the document.

## ✅ Decision
Chose Option A. Placed the language bindings precisely under the `v1.4.0` section to match the changelog. Adjusted the remaining v2.0 heading slightly to reflect the remaining planned AI & MCP work.

## 🧱 Changes made (SRP)
- `ROADMAP.md`: Moved "Language Bindings (FFI)" block to `v1.4.0`.

## 🧪 Verification receipts
```text
$ cargo xtask docs --check
Documentation is up to date.

$ cargo xtask version-consistency
Version consistency checks passed.

$ cargo deny --all-features check
advisories ok, bans ok, licenses ok, sources ok

$ cargo test --all-features -p tokmd-core
test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.50s
```

## 🧭 Telemetry
- Change shape: Docs update
- Blast radius: `docs` only
- Risk class: Low - factual correction
- Rollback: `git revert`
- Gates run: docs-check, version-consistency, deny, test core, fmt, clippy

## 🗂️ .jules artifacts
- `.jules/runs/cartographer_roadmap_design/envelope.json`
- `.jules/runs/cartographer_roadmap_design/decision.md`
- `.jules/runs/cartographer_roadmap_design/receipts.jsonl`
- `.jules/runs/cartographer_roadmap_design/result.json`
- `.jules/runs/cartographer_roadmap_design/pr_body.md`

## 🔜 Follow-ups
None.
