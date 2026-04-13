# Decision

## Context
The goal was to improve Jules itself by consolidating run packets, friction, learnings, and shared scaffolding (Archivist mission).
During exploration, it was found that every persona README (e.g. `.jules/personas/archivist/README.md`) contains an identical `## Notes` section at the bottom.
The initial attempt was to deduplicate this into `.jules/README.md` (Option A).

However, PR feedback indicated that "prompt-critical guidance must stay in the individual persona files because Jules receives personas individually".

## Option A
- Revert the deduplication patch and force a code change elsewhere.
- **Trade-offs:** Finding a different change might be forced or hallucinated, violating the "No tool cargo-culting" and "Output honesty" rules.

## Option B
- Accept the PR feedback. Since no honest code/docs/test patch is justified for this specific issue anymore, finish with a learning PR instead.
- Write the per-run packet, friction items, and persona notes explaining the learning.

## Selected: Option B
Option B is the correct path for a prompt-to-PR pipeline when a patch is no longer viable. It records the learning that persona-specific rules must stay in the persona files, even if they appear duplicated, because of how agents consume them.
