# tokmd Specification by Behavior

This specification defines product behavior using BDD-style scenarios and links each behavior to implementation modules and executable tests.

## How to read this document

Each behavior includes:
- **Given/When/Then** scenarios (human contract)
- **Implementation links** (where behavior lives)
- **Verification links** (which tests enforce it)

## Spec-01: Deterministic receipts

### Scenario: identical inputs produce stable receipts
- **Given** the same repository contents, flags, and schema version
- **When** `tokmd run`, `tokmd lang`, or `tokmd export` is executed repeatedly
- **Then** output ordering and normalized paths are stable and deterministic

**Implementation**
- `tokmd-model` aggregation and ordering rules.  
- `tokmd-format` rendering and stable row output.

**Verification**
- Property tests for normalization and ordering invariants.  
- Snapshot tests validating stable rendered output.

## Spec-02: Schema-versioned JSON contracts

### Scenario: machine consumers can validate outputs
- **Given** JSON-producing commands
- **When** a receipt is emitted
- **Then** it includes schema metadata and conforms to documented schema families

**Implementation**
- `tokmd-types` schema constants and DTO envelopes.  
- `docs/schema.json` and related schema docs.

**Verification**
- CLI schema validation tests in integration suite.

## Spec-03: PR risk and evidence gates

### Scenario: cockpit highlights review risk with hard gates
- **Given** base/head refs and optional evidence artifacts
- **When** `tokmd cockpit` runs
- **Then** it returns review-focused metrics and gate outcomes with explainable reasons

**Implementation**
- `tokmd-git`, `tokmd-analysis`, and `tokmd-gate` integrations in cockpit workflow.

**Verification**
- `cockpit_integration` tests for coverage, supply-chain, contract, determinism, and complexity gates.

## Spec-04: LLM-safe context packing

### Scenario: bounded context selection for AI workflows
- **Given** a token budget and path selection rules
- **When** `tokmd context` is executed
- **Then** selected files fit budget constraints and include explicit truncation markers when needed

**Implementation**
- Context selection logic in `tokmd-core`/CLI command flow.

**Verification**
- Context-focused integration tests and deterministic output assertions.

## Spec-05: Redaction and privacy guarantees

### Scenario: shareable receipts avoid sensitive paths
- **Given** redaction mode (`paths` or `all`)
- **When** artifacts are generated
- **Then** sensitive path material is replaced deterministically using stable hashing

**Implementation**
- Redaction helpers and rendering hooks in `tokmd-format`.

**Verification**
- Unit + property tests for deterministic redaction.

## Traceability matrix (BDD → code → tests)

| Spec | BDD focus | Implementation anchors | Primary tests |
|---|---|---|---|
| Spec-01 | Deterministic receipts | `crates/tokmd-model`, `crates/tokmd-format` | crate property tests, snapshot suites |
| Spec-02 | Schema contracts | `crates/tokmd-types`, `docs/schema.json` | `crates/tokmd/tests/schema_validation.rs` |
| Spec-03 | Cockpit evidence gates | `crates/tokmd/src/cmd/cockpit.rs`, `crates/tokmd-gate` | `crates/tokmd/tests/cockpit_integration.rs` |
| Spec-04 | Context budget behavior | `crates/tokmd/src/cmd/context.rs`, `crates/tokmd-core` | context-related integration coverage |
| Spec-05 | Redaction determinism | `crates/tokmd-format/src/redact/` | tokmd-format redaction tests + properties |

## ADR index

- [ADR-0001: Specification-as-Behavior with Executable Traceability](adr/0001-specification-traceability.md)
- [ADR-0002: BDD Mapping Standard for Code and Tests](adr/0002-bdd-mapping-standard.md)
