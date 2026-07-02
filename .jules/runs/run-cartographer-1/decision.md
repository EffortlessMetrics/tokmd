## Problem
The `docs/implementation-plan.md` outlines "Phase 6: MCP Server Mode (v2.0)" and "Phase 7: Tree-sitter Integration", but `ROADMAP.md` only mentions "v2.0 - Platform Evolution" containing MCP, Streaming, Plugin, and Intelligence Features. It also mentions "v3.0 - Tree-sitter Integration". However, `ROADMAP.md` is out of sync with `docs/implementation-plan.md` which lists Phase 6 as MCP Server and Phase 7 as Tree-sitter. More importantly, the recently shipped GHCR runtime work for the GitHub Action (shipped in 1.14.0) is not fully documented in the `docs/implementation-plan.md` for `Phase 5g: PR Evidence Packet Workflow (v1.14.0)` - it only says `[x] Add runtime: container Action support for the GHCR runtime.` but doesn't explain the verification gate and mutable tag constraints that were implemented.

Looking at `docs/NOW.md`:
"Packet workflow GHCR runtime: the packet CLI and mode: packet Action shipped in v1.14.0 on the prebuilt-binary runtime; the runtime: container GHCR path is now wired for verification-gated tags (currently 1.14.0, with mutable tags rejected). Extending the supported-tag set per new stable tag and the Cargo fallback story are the remaining support-model work."

And `docs/specs/packet-ghcr-runtime.md` explains the strict gate for `runtime: container`.

`docs/implementation-plan.md` is missing the `v1.15.x — Selection-First Product and Evidence Work` horizon that `ROADMAP.md` has.

Also `ROADMAP.md` under `Future Horizons` -> `v1.15.x — Selection-First Product and Evidence Work` lists "Release/distribution verification", "CLI and adoption UX", "Review evidence consumption", "Measured performance and CI feedback", "Browser/WASM rootless capability expansion", "AST shadow evidence expansion".

I will align `docs/implementation-plan.md` with `ROADMAP.md` by adding Phase 5h or updating the transition to v1.15.x. I'll also clarify the GHCR runtime support model in Phase 5g.

### Option A (recommended)
Update `docs/implementation-plan.md` to reflect `v1.15.x — Selection-First Product and Evidence Work` as a distinct phase before `v2.0`. Expand `Phase 5g` to document that `runtime: container` GHCR Action support was shipped with verification-gate enforcement for tags. Rename `Phase 6` and `Phase 7` to match the exact v2.0 and v3.0 horizons in `ROADMAP.md` and clarify their names.

- Fits shard because it is fixing roadmap/design/requirements drift from shipped reality and stale implementation-plan sections.
- Trade-offs: Structure is improved, governance is tightened.

### Option B
Only update `ROADMAP.md` to use the phase numbers from `docs/implementation-plan.md`.

- Doesn't fix the missing v1.15.x block in `docs/implementation-plan.md`.

## ✅ Decision
Option A. It accurately reflects the `v1.15.x` pause/selection phase and clarifies the strict GHCR gate in Phase 5g, aligning `docs/implementation-plan.md` with `ROADMAP.md` and the shipped reality.
