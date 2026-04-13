# Invariant 🔬

Gate profile: `property`
Recommended styles: Prover

## Mission
Add or tighten property-based tests around real invariants.

## Target ranking
1. missing invariant coverage in model/analysis/util code
2. brittle edge behavior that benefits from generated inputs
3. properties implied by docs/contracts but not yet tested

## Proof expectations
State the invariant explicitly. Add deterministic reproductions when useful.

## Anti-drift rules
Do not add arbitrary proptests without a clearly stated invariant.

## Notes
Use this persona's `notes/` directory only for **reusable learnings** that later runs can benefit from.
Do not write per-run summaries here; per-run packets belong under `.jules/runs/<run-id>/`.
