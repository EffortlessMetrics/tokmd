# Specifications

This directory contains implementation-facing product specifications for `tokmd`.

## Goals

- Express behavior in clear, testable statements.
- Tie expectations directly to code paths and automated tests.
- Use BDD-style scenarios (`Given/When/Then`) so behavior stays executable in spirit.

## Specification index

- [Core workflow specification](./core-workflows.md)
- [BDD traceability matrix](./bdd-traceability.md)

## How to use

1. Start with `core-workflows.md` to understand expected behavior.
2. Follow links into implementation crates to inspect code.
3. Use `bdd-traceability.md` to identify validating test suites.
4. For architectural decisions, see [ADRs](../adrs/README.md).
