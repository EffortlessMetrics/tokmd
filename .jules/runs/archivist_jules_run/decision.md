# Decision

## Context
The goal is to improve Jules itself by consolidating run packets, friction, learnings, and shared scaffolding (Archivist mission).
During exploration, it was found that every persona README (e.g. `.jules/personas/archivist/README.md`) contains an identical `## Notes` section at the bottom:

```markdown
## Notes
Use this persona's `notes/` directory only for **reusable learnings** that later runs can benefit from.
Do not write per-run summaries here; per-run packets belong under `.jules/runs/<run-id>/`.
```

This violates the principle of "move duplicated persona-local conventions into neutral shared guidance" which is ranking #4 on the Archivist task list.

## Option A
- Move this guidance to `.jules/README.md` under a `## Persona Notes` section.
- Remove the duplicated `## Notes` section from all persona READMEs.
- This consolidates the duplicate policy into a single central location.

## Option B
- Add a new runbook for persona guidelines in `.jules/runbooks/PERSONA_NOTES.md` and link it in the READMEs.
- Leaves boilerplate in place but points elsewhere.

## Selected: Option A
Option A is chosen because it perfectly aligns with the target "move duplicated persona-local conventions into neutral shared guidance". The root `.jules/README.md` is the primary truth source for directory purposes, making it the correct place for this guidance.
