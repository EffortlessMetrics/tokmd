# Friction Item

id: FRIC-20260407-ARCHIVIST
persona: Archivist
style: Builder
shard: workspace-wide
status: open

## Problem
Personas share identical boilerplate structural headings in their `.jules/personas/<persona>/README.md` files. This suggests deduplication into a shared template under `.jules/runbooks/PERSONA_TEMPLATE.md`. However, because each persona is sent to the agent independently as a standalone contract, guidance *must* live within the concrete persona files. Attempting to DRY up persona documentation creates indirection without changing actual agent behavior, leading to rejected patches.

## Evidence
- `.jules/personas/*/README.md`
- PR feedback rejecting `archivist: create shared persona template 🗃️` due to independent agent execution surface.

## Why it matters
Future Archivist runs may incorrectly target persona documentation deduplication as a "low hanging fruit" to reduce friction. This explicitly documents the constraint so agents focus on actual high-signal improvements rather than purely structural markdown DRYing.

## Done when
- [ ] Add explicit instruction to `AGENTS.md` or Archivist prompt instructing agents to avoid abstracting `.jules/personas/*/README.md` content into shared `.jules/runbooks/` templates.
