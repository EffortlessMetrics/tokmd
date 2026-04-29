# ADR-0002: Unified Contract Between CLI, Core, and Language Bindings

- **Status**: Accepted
- **Date**: 2026-04-29

## Context

tokmd behavior is surfaced via multiple interfaces (CLI, Rust core facade, FFI, Python, Node). Divergence across these layers creates inconsistent automation behavior and difficult debugging.

## Decision

We enforce contract parity:
- CLI modes map to equivalent core workflow semantics;
- FFI responses maintain a consistent envelope (`ok`, `data`, `error`);
- language bindings remain thin adapters that preserve core behavior.

## Consequences

### Positive
- Predictable semantics independent of entrypoint.
- Simpler test strategy through shared contract assertions.
- Lower support burden for multi-runtime users.

### Tradeoffs
- Interface-specific features require explicit justification and documentation.
- Some adapter-level ergonomics may be constrained by envelope compatibility.

## Compliance

When adding/changing behavior:
1. define expected parity in specs and BDD scenarios;
2. verify at least CLI + one non-CLI interface where practical;
3. document any intentional mismatch with rationale and migration guidance.
