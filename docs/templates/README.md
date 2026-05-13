# Source-of-Truth Templates

These templates are starter shapes for new source-of-truth artifacts. They are
not source-of-truth documents by themselves; copy one into the owning directory
and fill in repo-specific content before linking to it from a plan, spec, ADR,
or `.jules/goals/active.toml`.

Use:

- `proposal.md` for exploratory rationale and alternatives;
- `spec.md` for testable behavior contracts and proof requirements;
- `adr.md` for durable architecture decisions;
- `plan.md` for PR sequencing and validation commands;
- `active-goal.toml` for the small machine-readable active lane.

Run `cargo xtask doc-artifacts --check` after creating or changing active
source-of-truth artifacts.
