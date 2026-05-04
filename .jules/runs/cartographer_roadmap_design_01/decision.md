# Decision

## Option A (recommended)
Update `docs/design.md` to remove "future" qualifiers from in-memory and WASM documentation.
- The `MemFs` and in-memory execution paths, as well as WASM productization, have already shipped as part of v1.9.0 and v1.10.0 (as documented in `ROADMAP.md`).
- Keeping "future" on these surfaces creates factual drift.
- Updating `docs/design.md` keeps the architectural documentation aligned with the current shipped reality.

## Option B
Update `ROADMAP.md` or other docs.
- The roadmap correctly marks these items as "✅ Complete".
- `docs/design.md` represents the architectural truth and has a clear case of factual drift.

## Decision
Option A. I will update `docs/design.md` to remove the "future" qualifiers from the `MemFs` and `WASM` descriptions, as they are now fully shipped features in the `tokmd-io-port` design.
