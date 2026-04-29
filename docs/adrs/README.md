# ADR Index and Process

ADRs capture architecture decisions that affect tokmd across crates or surfaces.

## Naming

- File format: `NNNN-short-title.md`
- Example: `0001-adr-program-and-format.md`

## Status Values

- `proposed`
- `accepted`
- `superseded`
- `deprecated`

## Lifecycle

1. Add ADR with `proposed` status.
2. Discuss during PR review.
3. Mark `accepted` when merged.
4. If replaced, mark old ADR as `superseded` and reference the new one.

## ADR List

- [0001: Establish ADR program and canonical template](./0001-adr-program-and-format.md)
- [0002: Keep schema versions per receipt family](./0002-schema-versions-per-receipt-family.md)
