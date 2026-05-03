## Option A (recommended)
Update `docs/NOW.md` to reflect the `1.10.0` release reality. The document currently asserts `1.9.0` is the active release, but the workspace is on `1.10.0` and has completed the Phase 5b milestone. We will update the NOW section to reference the `1.10.0` CI control plane and proof stability, and update the NEXT section to reflect the `v1.11.0` browser runtime polish targets defined in the implementation plan.

Trade-offs: Structure / Velocity / Governance - High alignment with governance, requires a small patch, minimal velocity cost. Keeps contributors aware of the actual current horizon.

## Option B
Delete `docs/NOW.md` and consolidate into `ROADMAP.md`.

Trade-offs: `NOW.md` is an established convention in this repository for single-screen operational truth. Removing it could violate expected contributor habits and reduce velocity.

## Decision
Option A. It is a precise, highly-aligned fix for a factual drift between the shipped reality (`v1.10.0`) and the operational truth documentation.
