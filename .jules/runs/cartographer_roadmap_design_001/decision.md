# Decision

## Observation
I explored `ROADMAP.md` and `docs/implementation-plan.md` to identify factual drift between shipped reality and roadmap/design docs.
I found that `CHANGELOG.md` mentions `[1.12.0]` release has already shipped on `2026-06-04`. Also `docs/releases/1.12.md` and `docs/releases/1.12-ledger.md` exist and describe what shipped in 1.12.0.
However, both `ROADMAP.md` (around line 608) and `docs/implementation-plan.md` still list `v1.12.x` as a future horizon ("Potential lanes" and "Selection-First Product and Evidence Work").
Furthermore, the `Status Summary` table in `ROADMAP.md` only lists up to `v1.11.0`, and does not have an entry for `v1.12.0`.

There is factual drift: 1.12.0 shipped, but the roadmap still treats 1.12.x as a potential future lane and misses it in the status table.

## Option A
Update `ROADMAP.md` and `docs/implementation-plan.md` to reflect that `v1.12.0` has shipped.
- Add `v1.12.0` to the Status Summary table in `ROADMAP.md`.
- Move the `v1.12.x` section from "Later Horizons" / "Future Horizons" into the completed/shipped sections.
- Add the `v1.13.x` or next equivalent as the current selection-first active mode.

## Option B
Update only `docs/implementation-plan.md` and leave `ROADMAP.md` as is. This makes less sense since `ROADMAP.md` contains a Status Summary table.

## Decision
Option A. It fits the Cartographer persona's mission to keep roadmap docs aligned with shipped reality. I will update `ROADMAP.md` and `docs/implementation-plan.md` to properly document the shipped v1.12.0 release, reflecting its true status.
