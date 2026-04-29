# tokmd Behavioral Specification

This document defines the implementation-facing behavioral specification for `tokmd`.
It is intentionally executable-by-convention: each section is written so it can be traced to BDD-style tests and architecture decisions.

## Scope

The specification covers:

- CLI behavioral contracts (`tokmd`, `tokmd lang`, `module`, `export`, `run`, `analyze`, `diff`, `cockpit`, `gate`, `sensor`, and related helpers).
- Receipt determinism and schema behavior.
- Feature-gated behavior (`git`, `content`, `walk`, `halstead`).
- Stability requirements for machine-facing outputs.

It does not replace the full command reference in `docs/reference-cli.md`; it defines normative behavior and acceptance intent.

## Specification Format

Each requirement includes:

- **Rule**: the normative behavioral statement.
- **Rationale**: why the rule exists.
- **Validation**: where the behavior should be covered in BDD/integration/property tests.
- **Decision links**: ADRs that justify tradeoffs.

## Core Behavioral Rules

### SPEC-001: Deterministic output ordering

- **Rule**: All machine outputs must be deterministic for equivalent inputs and settings.
- **Rationale**: Enables reproducible pipelines, snapshot tests, and policy-gate confidence.
- **Validation**: BDD + snapshot tests in format/model/types crates.
- **Decision links**: [ADR-0001](adrs/0001-deterministic-output-and-ordering.md).

### SPEC-002: Normalized path semantics

- **Rule**: Output paths are normalized to forward slashes before rendering or serialization.
- **Rationale**: Cross-platform consistency and stable module keys.
- **Validation**: BDD and property tests in `tokmd-model`, `tokmd-scan`, `tokmd-format`.
- **Decision links**: [ADR-0002](adrs/0002-path-normalization-contract.md).

### SPEC-003: Feature-gated capability behavior

- **Rule**: Commands that depend on optional capabilities must fail clearly or degrade explicitly when the feature is unavailable.
- **Rationale**: Keeps minimal builds viable while preserving user trust.
- **Validation**: BDD/integration tests for compile-time and runtime capability surfaces.
- **Decision links**: [ADR-0003](adrs/0003-feature-gated-capability-contract.md).

### SPEC-004: Schema version governance

- **Rule**: JSON receipt shape changes require corresponding schema-version and schema-doc updates for the affected receipt family.
- **Rationale**: Prevents silent contract breaks for downstream consumers.
- **Validation**: schema validation tests + BDD coverage for envelope metadata.
- **Decision links**: [ADR-0004](adrs/0004-receipt-schema-versioning-policy.md).

## Traceability to BDD

BDD tests are distributed across crates in `tests/bdd.rs` (and module-local `bdd.rs` for analysis subdomains).

The BDD corpus should trace back to SPEC IDs (for example via scenario names, comments, or commit messages) and validate both positive and negative paths.

See `docs/testing.md` for how BDD sits with integration, property, fuzz, and mutation layers.

## Change Process

When implementation behavior changes:

1. Update this spec (add/modify SPEC entries).
2. Add or update an ADR if the change introduces a durable design decision.
3. Update BDD scenarios that assert the behavior.
4. Update user-facing docs (`docs/reference-cli.md`, `docs/SCHEMA.md`) if externally visible.
