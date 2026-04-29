# Specification & ADR Program

This document defines how tokmd captures implementation intent in **specifications** and durable design decisions in **ADRs** (Architecture Decision Records).

## Goals

- Keep major features traceable from requirement → implementation → validation.
- Make design tradeoffs explicit and reviewable.
- Prevent drift between CLI behavior, library APIs, and receipt schemas.

## Deliverables

- `docs/specs/` — implementation-level specifications.
- `docs/adrs/` — architecture decision records.
- `docs/adrs/README.md` — ADR lifecycle and status conventions.

## Specification Standard

Each spec should include:

1. **Problem Statement**
2. **Scope** (in/out)
3. **User-facing behavior**
4. **Data/API contracts**
5. **Implementation plan**
6. **Validation plan**
7. **Rollout and compatibility notes**

## ADR Standard

Each ADR should include:

1. **Status** (`proposed`, `accepted`, `superseded`, `deprecated`)
2. **Context**
3. **Decision**
4. **Consequences** (benefits, costs, follow-ups)
5. **Alternatives considered**

## Traceability Conventions

- Reference specs in PR descriptions and implementation comments where meaningful.
- Reference ADR IDs in affected docs and major modules.
- If output structure changes, increment the relevant schema version and update schema docs.

## Initial Scope

The initial pass covers:

- Receipt schema governance
- Cross-surface API consistency (CLI, core facade, FFI, Python/Node bindings)
- Deterministic output guarantees

## Maintenance

- New platform-level behavior should land with either:
  - a new spec + optional ADR, or
  - an ADR update if the behavior is a policy change.
- Superseded ADRs remain in-place for historical context and must link to successors.
