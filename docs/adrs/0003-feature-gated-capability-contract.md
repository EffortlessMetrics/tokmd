# ADR-0003: Feature-gated capability contract

- **Status**: Accepted
- **Date**: 2026-04-29

## Context

`tokmd` supports minimal and full builds through optional features (`git`, `content`, `walk`, and others). Users need explicit behavior when capabilities are unavailable.

## Decision

Define a capability contract:

- If a command strictly requires a disabled feature, return a clear, actionable error.
- If a feature is optional for a report section, return explicit null/skipped metadata instead of silent omission.
- Keep capability surfaces visible in machine outputs when relevant.

## Consequences

- Better UX for constrained builds.
- Stronger machine readability for partial capability runs.
- Additional testing requirements across feature combinations.
