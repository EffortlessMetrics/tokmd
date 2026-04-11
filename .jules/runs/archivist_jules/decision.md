# Investigation
Inspected the persona definitions under `.jules/personas/*/README.md` and noticed that every single persona includes an identical `## Notes` section about the use of the `notes/` directory:
> Use this persona's `notes/` directory only for **reusable learnings** that later runs can benefit from.
> Do not write per-run summaries here; per-run packets belong under `.jules/runs/<run-id>/`.

This duplication violates DRY principles and creates noise in the specific mission/target documentation of each persona. It aligns perfectly with Archivist's mission item:
> 4. move duplicated persona-local conventions into neutral shared guidance

# Options considered
## Option A (recommended)
Remove the duplicated `## Notes` section from all `.jules/personas/*/README.md` files and consolidate the rule into the shared `.jules/README.md` under a new section `### Persona Notes Directory`.

This simplifies the per-persona docs and makes the shared architecture rule universally visible.

## Option B
Create a new file `.jules/policy/persona_notes.md` containing the rule, and link to it from each persona file.

This moves the duplication but still requires all personas to carry the boilerplate link. Option A is cleaner.

# Decision
Chose Option A. It removes boilerplate across 16 persona README files and places the structural rule into the root `.jules/README.md` where other directory structural rules are defined.
