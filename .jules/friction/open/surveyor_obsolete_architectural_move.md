# Friction: Obsolete Architectural Move

The prompt explicitly asked to use the `Surveyor` persona to improve architecture and structural coherence in the workspace-wide shard, specifically citing crate boundary/layering issues.

I attempted to address what looked like a tier boundary violation: moving `tokmd-analysis::source_complexity` directly into `tokmd-cockpit` (its sole consumer).

However, the PR was rejected as obsolete with the following context: "This reverses the current ownership decision recorded after #1785/#999: cockpit delegates function-scoped Rust source complexity to tokmd-analysis::source_complexity, and docs/NEXT.md plus docs/architecture-consolidation-plan.md now treat that as the active architecture. A future change here should start from a fresh spec/plan update rather than moving the heuristic back into cockpit."

This indicates a friction point where the immediate interpretation of "crate boundary hygiene" conflicts with a recent, explicit ownership decision that was not fully obvious from the code layout alone without consulting `docs/NEXT.md` and related history.

Future work in this area should prioritize updating architectural specs/plans first before attempting code moves that reverse established decisions.
