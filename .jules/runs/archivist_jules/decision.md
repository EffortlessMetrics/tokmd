# Decision

## Inspected
- `.jules/personas/*/README.md`
- `.jules/policy/agent_profiles.json`

## Options
### Option A (Recommended)
Create `.jules/runbooks/PERSONA_TEMPLATE.md` to formalize the expected structure of persona READMEs. Every persona duplicates the identical headings (`Gate profile`, `Recommended styles`, `## Mission`, `## Target ranking`, `## Proof expectations`, `## Anti-drift rules`). Documenting this as a shared template reduces recurring friction for adding or updating personas.

### Option B
Do nothing and output a learning PR.

## Decision
**Option A**. It perfectly satisfies Archivist target ranking #1 ("consolidate recurring friction themes into better templates") and #4 ("move duplicated persona-local conventions into neutral shared guidance").
