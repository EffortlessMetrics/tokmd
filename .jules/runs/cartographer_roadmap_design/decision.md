# Decision

## Problem
The `ROADMAP.md` currently places "Language Bindings (FFI)" (Python and Node.js) under the `### v2.0 — Platform Evolution` section in `## Future Horizons`. However, both of these bindings were marked as `✅ Complete` and were actually shipped in v1.4.0 (as evident from `CHANGELOG.md` lines 448-453).
This represents factual drift: shipped features are incorrectly categorised under a future planned major version (v2.0) instead of their actual shipped version in the completed milestones section (v1.4.0).

## Options considered
### Option A (recommended)
- Move the `#### A. Language Bindings (FFI) ✅ Complete` block from `### v2.0 — Platform Evolution` to the `## Completed: v1.4.0 — Complexity Metrics & PR Integration` section.
- Remove the FFI aspect from the v2.0 title/future horizon entirely, since it's shipped. The v2.0 Platform Evolution can focus strictly on what remains (e.g. `#### B. AI Agent Integration & MCP Server Mode` which could just be `#### AI Agent Integration & MCP Server Mode`).
- **Structure / Velocity / Governance:** This keeps the roadmap historically accurate (Governance) and clarifies the boundary between shipped v1 functionality and planned v2 functionality (Velocity & Structure).

### Option B
- Just rename the v2.0 heading or mark the whole block as part of v1, leaving it at the bottom.
- **Trade-offs:** Leaving completed items in the "Future Horizons" section defeats the purpose of separating "Completed Milestones" and "Future Horizons". It continues to mislead readers.

## Decision
Choose Option A. I will move the "Language Bindings (FFI)" section from `v2.0` to the `v1.4.0` section in `ROADMAP.md`.
