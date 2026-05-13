# Proposals

Proposals are exploratory design documents. They explain why a change might be
worth doing, what alternatives were considered, and what questions remain before
the work becomes a spec, ADR, or implementation plan.

## Use This Directory For

- product or architecture ideas that need comparison before commitment;
- research summaries that should survive beyond a chat thread;
- tradeoff analysis for future lanes;
- proposed policy or proof-control changes before they become checked rules.

## Do Not Use It For

- accepted behavior contracts;
- schema definitions;
- PR-by-PR implementation sequencing;
- raw run logs or terminal transcripts.

## Suggested Shape

```md
# Proposal: <title>

- Status: draft | proposed | accepted | superseded | withdrawn
- Owner:
- Related issues:
- Related specs:
- Related ADRs:

## Problem

## Goals

## Non-goals

## Options

## Recommendation

## Open Questions
```

When a proposal is accepted, move durable behavior into `docs/specs/`, durable
architecture decisions into `docs/adr/`, and implementation sequencing into
`docs/plans/`.
