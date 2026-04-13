# Persona Template Learning

Do not attempt to consolidate or "DRY up" the structural headings (`## Mission`, `## Proof expectations`, `## Anti-drift rules`) found across the various `.jules/personas/<persona>/README.md` files into a single `.jules/runbooks/` template.

**Why:** Each persona file serves as an independent, active contract sent to the agent during a run. The guidance must live inside these concrete files to be effective. Abstracting this content introduces indirection that breaks the prompt pipeline without changing actual agent behavior.
