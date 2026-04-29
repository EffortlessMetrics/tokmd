# ADR-0000: ADR and spec governance

- **Status:** accepted
- **Date:** 2026-04-29

## Context

tokmd currently records durable decisions, behavioral contracts, and release summaries across multiple documents. Some documents mix architecture rationale (why) with contract details (what/how), which makes it harder to track long-lived decisions and enforceable behavior separately.

## Decision

- ADRs record why a durable boundary, policy, or release rule exists.
- Specs record the exact, testable contract and behavior.
- Release notes, changelog entries, and roadmap material summarize delivery; they do not replace ADRs.

House style for ADRs:

- **Status:** `proposed` | `accepted` | `superseded` | `retired`
- Required sections:
  - context
  - decision
  - consequences
  - alternatives
  - enforcement
  - related specs

## Consequences

- Architectural intent and policy rationale stay stable and auditable.
- Behavioral contracts can evolve in spec docs without rewriting decision history.
- Release communication becomes easier to verify against ADR intent and spec behavior.

## Alternatives

- Keep mixed ADR/spec/policy content in single docs. Rejected because it obscures rationale versus contract and makes policy drift harder to detect.

## Enforcement

- New durable boundary/policy decisions should be captured as ADRs under `docs/adr/`.
- New externally consumed behavior should be captured or updated in a spec document.
- Documentation reviews should reject major architecture or policy changes that do not include ADR/spec updates.

## Related specs

- `docs/publish-surface.md` (current mixed policy/spec source to progressively split).
