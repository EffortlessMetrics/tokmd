# Behavior-Driven Development (BDD)

This document tracks behavior-level scenarios for tokmd capabilities and links each scenario set to implementation specifications and ADR rationale.

## Linked Design Inputs

- Specification: [`docs/specifications/implementation-spec.md`](./specifications/implementation-spec.md)
- ADR Index: [`docs/adr/README.md`](./adr/README.md)
- ADR-0001: Deterministic ordering and serialization
- ADR-0002: CLI/Core/bindings parity

## Feature: Deterministic Receipts

**Given** a repository and fixed scan settings  
**When** I run `tokmd lang --format json` repeatedly  
**Then** the serialized output is byte-identical for equivalent inputs.

**Given** a repository with embedded languages  
**When** I run with children mode `collapse`  
**Then** embedded counts are merged into parent totals consistently.

**Given** a repository with embedded languages  
**When** I run with children mode `separate`  
**Then** embedded rows are emitted consistently and clearly labeled.

_References_: Implementation Spec §1-§3, ADR-0001.

## Feature: Cross-Interface Contract Parity

**Given** equivalent arguments  
**When** I execute a workflow from CLI and `tokmd-core`  
**Then** both results represent the same domain semantics.

**Given** a valid `run_json` invocation  
**When** a mode completes  
**Then** response envelope fields include `ok`, `data`, and `error` consistently.

**Given** Python/Node bindings for a workflow  
**When** the same logical options are supplied  
**Then** results remain semantically equivalent to the core workflow.

_References_: Implementation Spec §4, ADR-0002.

## Feature: Policy Gate Behavior

**Given** policy inputs and evidence receipts  
**When** all required checks pass  
**Then** exit code is `0` and verdict is pass.

**Given** a policy violation  
**When** `tokmd gate` evaluates rules  
**Then** exit code is `2` with machine-readable failure reasons.

**Given** missing optional evidence  
**When** a policy is evaluated  
**Then** the result is explicit skip/unknown rather than silent success.

_References_: Implementation Spec §5.

## Maintenance Rules

- Add/adjust BDD scenarios whenever behavior changes.
- Link behavior changes to relevant ADRs (new or existing).
- Treat this file as the top-level behavior index and keep it current with implementation contracts.
