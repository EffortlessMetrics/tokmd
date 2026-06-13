## 💡 Summary
Updated `docs/NOW.md` and `docs/architecture.md` to reflect the shipped reality of `v1.11.0` browser runtime polish.

## 🎯 Why
`docs/NOW.md` mentioned that in-browser receipt generation shipped in `1.9.0` as its current state, making it feel stale given `1.11.0` just completed the "Browser Runtime Polish" milestone. `docs/architecture.md` was also not explicitly referencing the `1.11.0` constraints. This aligns the planning docs with the actual shipped capabilities.

## 🔎 Evidence
- `docs/NOW.md` under `LATER (roadmap)` referred to the browser runner as having shipped in `1.9.0`.
- `ROADMAP.md` shows that `v1.11.0` browser runtime polish is ✅ Complete.
- `docs/architecture.md` lacked direct reference to the `1.11.0` constraints under "Current browser constraints".

## 🧭 Options considered
### Option A (recommended)
- Update `docs/NOW.md` to reframe the browser runner state to current reality.
- **Why it fits:** Keeps the source-of-truth operational doc aligned with the actual shipped reality of the repo, addressing drift.
- **Trade-offs:** Fast, zero-risk doc change. Low friction, high structural clarity.

### Option B
- Look for other gaps in ROADMAP.md.
- **Why it fits:** The prompt targets roadmap drift.
- **Trade-offs:** We already found obvious drift. Delaying to search more may result in lower quality finds.

## ✅ Decision
Option A was chosen. It targets factual drift directly within a key structural operational doc (`docs/NOW.md`) and architecture constraints doc.

## 🧱 Changes made (SRP)
- `docs/NOW.md`: Updated the "Browser/WASM product continuation" bullet to reflect that in-browser analysis has shipped and was polished in `v1.11.0`.
- `docs/architecture.md`: Updated the "Current browser constraints" section to acknowledge the polish delivered in `v1.11.0`.

## 🧪 Verification receipts
```text
$ cat ROADMAP.md docs/design.md docs/architecture.md docs/SCHEMA.md 2>/dev/null
[Output truncated]
$ grep -i -C 5 "roadmap" ROADMAP.md
[Output truncated]
$ cat docs/NOW.md
[Output truncated]
$ patch docs/NOW.md patch_now.diff
patching file docs/NOW.md
Hunk #1 succeeded at 25 (offset 2 lines).
$ patch docs/architecture.md patch_arch.diff
patching file docs/architecture.md
```

## 🧭 Telemetry
- Change shape: Docs update.
- Blast radius: None (documentation only).
- Risk class: Low (addressing factual drift in docs).
- Rollback: Revert changes to `docs/NOW.md` and `docs/architecture.md`.
- Gates run: `cargo xtask docs --check`, `cargo fmt -- --check`.

## 🗂️ .jules artifacts
- `.jules/runs/cartographer_roadmap_design/envelope.json`
- `.jules/runs/cartographer_roadmap_design/decision.md`
- `.jules/runs/cartographer_roadmap_design/receipts.jsonl`
- `.jules/runs/cartographer_roadmap_design/result.json`
- `.jules/runs/cartographer_roadmap_design/pr_body.md`

## 🔜 Follow-ups
None.
