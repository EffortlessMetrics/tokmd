# Investigation

I searched the project for instances of "fake fix" and identified multiple instances in past `.jules/runs/*` outputs where agents encountered conflicts. The shared instructions forbid "forcing a fake fix" while other documentation simultaneously forbids producing a "learning PR" when no issue is found (e.g., instructing the agent to stop without an artifact and just output a zero-drift result).

Specifically, these files had conflicting exit guidance:
- `.jules/personas/mutant/README.md`
- `.jules/personas/steward/README.md`
- `.jules/personas/librarian/README.md`

They told the agent to "stop with a zero-drift result. Do not... [create] a learning PR just to satisfy the expectation of a proof-improvement patch." But the main `RUN_PACKET.md` and standard Jules prompts say "If no honest code/docs/test patch is justified, finish with a learning PR instead of forcing a fake fix." The orchestrator requires a PR outcome.

# Options considered

### Option A (recommended)
Add a unified `Shared Zero-Drift Guidance` section to `.jules/runbooks/RUN_PACKET.md` that standardizes the learning PR fallback mechanism, explicitly superseding older "stop without an artifact" rules. Update `mutant`, `steward`, and `librarian` persona docs to reference this shared guidance.
- Fits because it directly aligns with the Archivist mission to "move duplicated persona-local conventions into neutral shared guidance" and removes prompt-level contradictions.
- Structure: Improves consistency.
- Velocity: Unblocks agents that were failing due to conflicting constraints.
- Governance: Clarifies the official zero-drift policy.

### Option B
Remove the exit sections from the persona files entirely and only document it in `RUN_PACKET.md`.
- While cleaner, this might make agents overlook the zero-drift possibility entirely when reading their specific persona documentation, leading them to hallucinate patches anyway.

# Decision
Option A. It's the safest way to maintain persona-specific context (knowing *when* a surface is already tight) while unifying the mechanical outcome (what to *do* when it's tight) in the shared runbook.
