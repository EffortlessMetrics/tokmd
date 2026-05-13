# Specs

Specs own testable behavior contracts. A spec should say what an artifact,
command, schema, workflow, or policy means and what proof keeps that meaning
honest.

Existing repository specs may still live as top-level documents such as
`docs/specification.md`, `docs/review-packet.md`, and schema-specific reference
files. This directory is the home for new focused specs that should not grow the
top-level docs list.

## Use This Directory For

- command behavior contracts;
- receipt and packet semantics;
- compatibility and migration rules;
- proof requirements for a behavior or artifact;
- accepted product surfaces that need tests or verifiers.

## Do Not Use It For

- exploratory rationale before a direction is chosen;
- ADR-style architecture decisions;
- release notes;
- per-PR task lists.

## Suggested Shape

```md
# Spec: <title>

- Status: draft | active | superseded | retired
- Schema family, if any:
- Related ADRs:
- Related proof scopes:

## Contract

## Inputs

## Outputs

## Compatibility

## Proof Requirements

## Open Questions
```

Specs should cite concrete commands, tests, schema files, or verifiers whenever
the contract is already enforced.
