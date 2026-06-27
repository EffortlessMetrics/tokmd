## 💡 Summary
Updated `docs/NOW.md` and `docs/architecture.md` to reflect the shipped state of the v1.11.0 browser runner polish. Stale references to `v1.9.0` non-goals were removed or updated to correctly represent current constraints without version drift.

## 🎯 Why
With the completion of v1.11.0 ("Browser Runtime Polish") and active focus on v1.12.0 ("Cockpit & Architecture Consolidation"), the `architecture.md` references to "Non-goals for v1.9.0" read as outdated history rather than the current truth of the system's browser boundaries. `NOW.md` also incorrectly framed in-browser capabilities using historical `1.9.0` phrasing.

## 🔎 Evidence
- `docs/architecture.md` explicitly listed `### Non-goals for v1.9.0`.
- `docs/NOW.md` stated `in-browser receipt generation shipped in 1.9.0` while actively in the `LATER` section, which creates confusing version timelines.

## 🧭 Options considered
### Option A (recommended)
- Update references to `v1.9.0` non-goals in `docs/architecture.md` to reflect the current ongoing state, and update `docs/NOW.md` to remove stale "shipped in 1.9.0" context, orienting instead to the active v1.12.0 architecture consolidation focus.
- Why it fits: The architecture document describes the current shape of the WASM/Browser Runner. Keeping non-goals scoped to `v1.9.0` when the repo is at `1.11.0` makes the architecture doc read as outdated. `docs/NOW.md` also referenced the v1.9.0 browser runner instead of the broader completed state.
- Trade-offs: Corrects drift without being overly broad. Velocity is fast. Structure is improved.

### Option B
- Redo the entire WASM section to focus solely on architecture, removing any milestone/versioning mentions entirely.
- When to choose: If the entire browser runner strategy changed dramatically.
- Trade-offs: Higher risk of removing important historical or structural context.

## ✅ Decision
Option A. It's precise, targets actual drift where docs lag behind the completed `v1.11.0` and active `v1.12.0` roadmap states, and fixes misleading references to older versions in structural/architecture files.

## 🧱 Changes made (SRP)
- `docs/architecture.md`: Renamed `### Non-goals for v1.9.0` to `### Current browser non-goals`.
- `docs/NOW.md`: Updated roadmap reference from `shipped in 1.9.0` to `has shipped`.

## 🧪 Verification receipts
```text
$ cargo xtask docs --check
Documentation is up to date.

$ cargo fmt -- --check && cargo clippy -- -D warnings
Finished `dev` profile [unoptimized + debuginfo] target(s) in 38.34s
```

## 🧭 Telemetry
- Change shape: Docs update
- Blast radius: Docs only
- Risk class: Zero risk.
- Rollback: git revert
- Gates run: `cargo xtask docs --check`, `cargo fmt -- --check`, `cargo clippy -- -D warnings`

## 🗂️ .jules artifacts
- `.jules/runs/cartographer_roadmap_design/envelope.json`
- `.jules/runs/cartographer_roadmap_design/decision.md`
- `.jules/runs/cartographer_roadmap_design/receipts.jsonl`
- `.jules/runs/cartographer_roadmap_design/result.json`
- `.jules/runs/cartographer_roadmap_design/pr_body.md`

## 🔜 Follow-ups
None.
