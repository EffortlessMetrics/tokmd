# Decision

## Context
The `workspace-wide` shard involves structural or meta work across the entire repository. The `Archivist` persona is tasked with improving Jules itself by consolidating run packets, friction, learnings, and shared scaffolding. Specifically, target #4 states: "move only neutral shared conventions into shared guidance; keep prompt-critical persona instructions in the individual persona README files."

Currently, 16 different persona README files under `.jules/personas/*/README.md` contain an identical `## Notes` section (or duplicated lines) that instructs the agent:
`Use this persona's notes/ directory only for **reusable learnings** that later runs can benefit from.`
`Do not write per-run summaries here; per-run packets belong under .jules/runs/<run-id>/.`

These instructions are neutral and shared, not persona-specific. However, the `archivist` persona also contains a prompt-critical instruction in its `## Notes` section:
`Do not remove prompt-critical instructions from persona README files just because they also appear in shared docs; personas are sent individually.`

## Options

### Option A: Consolidate shared `notes/` instructions into `.jules/README.md` (Recommended)
Remove the identical duplicated lines about `notes/` and `runs/` directory usage from all 16 persona README files. For personas like `archivist` that have additional prompt-critical instructions in the `## Notes` section, retain those specific instructions. Ensure `.jules/README.md` and `.jules/runbooks/RUN_PACKET.md` already clearly specify these rules (which they do, mostly, but we can make it explicit in `.jules/README.md` under "Storage rules").

- **Structure**: Reduces duplication and centralizes neutral shared policy in the shared `.jules/README.md`. Follows target #4 directly.
- **Velocity**: Future personas won't need to copy-paste this boilerplate.
- **Governance**: Policy updates regarding run packets or notes happen in one place.

### Option B: Leave as-is and add a new policy file
Create a new file like `.jules/policy/notes.md` detailing how to use notes, but leave the duplicated lines in the persona READMEs.

- **Structure**: Increases documentation fragmentation without removing the duplication.
- **Velocity**: Requires maintaining two sources of truth for the same policy.
- **Governance**: Agents might get confused between the shared policy and the persona README.

## Decision
**Option A**. It directly fulfills target #4 of the Archivist persona's mission ("move only neutral shared conventions into shared guidance; keep prompt-critical persona instructions in the individual persona README files"). It reduces noise in 16 files while preserving the prompt-critical instruction for the `archivist` persona.
