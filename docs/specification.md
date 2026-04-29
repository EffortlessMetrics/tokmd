# tokmd Implementation Specification

## Purpose

This specification defines the concrete implementation contracts for tokmd command surfaces, receipt outputs, determinism, and extension points. It complements `docs/requirements.md` (what) and `docs/design.md` (why/how) by pinning the operational behaviors expected from current implementations.

## Scope

This spec applies to:
- CLI command behavior and exit semantics
- Library facade (`tokmd-core`) contracts
- Receipt envelope invariants and schema family versioning
- Deterministic output requirements
- Feature-gated enrichers and graceful degradation behavior

It does **not** define orchestration policy for external systems.

## Implementation Contracts

### 1) Command Surface Contract

Implemented command families:
- Inventory: `lang`, `module`, `export`, `run`
- Analysis: `analyze`, `baseline`, `cockpit`, `gate`, `diff`
- LLM workflows: `context`, `handoff`, `tools`
- Utilities: `sensor`, `badge`, `init`, `check-ignore`, `completions`

Contract rules:
1. Commands MUST provide stable help semantics and parse errors via clap.
2. Commands producing machine outputs MUST support deterministic `json`/`jsonl`/`csv` modes where applicable.
3. Commands SHOULD preserve backwards-compatible flag semantics within a minor release line.

### 2) Receipt Envelope Contract

For JSON family outputs, the implementation MUST emit versioned envelopes with:
- tool identity (`tool`, `tool_version`)
- generation metadata (`generated_at_ms`, command mode)
- payload sections (`scan`, mode-specific sections)
- integrity metadata when enabled

Schema families and active versions:
- Core receipts (`lang`, `module`, `export`, `diff`, `run`): `SCHEMA_VERSION = 2`
- Analysis receipts: `ANALYSIS_SCHEMA_VERSION = 9`
- Cockpit receipts: `COCKPIT_SCHEMA_VERSION = 3`
- Handoff manifests: `HANDOFF_SCHEMA_VERSION = 5`
- Context receipts: `CONTEXT_SCHEMA_VERSION = 4`
- Context bundles: `CONTEXT_BUNDLE_SCHEMA_VERSION = 2`
- Sensor reports: semantic schema id `sensor.report.v1`

Any breaking structural change MUST increment the relevant schema family version.

### 3) Determinism Contract

Implementations MUST preserve byte-stable output for identical inputs and settings by enforcing:
- Ordered maps/sets (`BTreeMap`, `BTreeSet`) at serialization boundaries
- Stable sorting (`code desc`, `name asc` tie-break)
- Path normalization to `/` separators
- Stable truncation markers for budget-capped sections
- Deterministic redaction hashing (BLAKE3)

### 4) Execution and Exit Contract

Exit behavior:
- `0`: success
- `1`: runtime/tooling failure
- `2`: policy/gate failure

Degradation behavior:
- Missing optional evidence MUST produce explicit skipped/unknown semantics instead of silent pass.
- Partial receipts MAY be emitted when failures occur after useful data has been derived.

### 5) Feature-Gated Enricher Contract

Feature flags define optional enrichers:
- `git`: repository history and hotspot-derived signals
- `content`: entropy/import/tag/hash signals
- `walk`: asset and filesystem traversal signals

Contract rules:
1. Disabled features MUST not cause runtime panics.
2. Presets MUST degrade predictably when an optional feature is not compiled.
3. Output should explicitly indicate unavailable sections where relevant.

### 6) API and Binding Contract

`tokmd-core` provides clap-free workflow entrypoints and JSON FFI:
- Workflow functions return typed receipts.
- `ffi::run_json(mode, args_json)` returns a JSON envelope with `ok`, `data`, and `error` branches.

Bindings contract:
- Python bindings MUST return native Python dictionaries.
- Node bindings MUST expose async APIs and avoid event loop blocking.

## Validation Matrix

Minimum validation before release:
1. Workspace formatting and lint checks.
2. CLI integration tests for command contracts.
3. Snapshot tests for output stability.
4. Property tests for deterministic invariants.
5. Schema compatibility checks for modified receipt families.

## Change Management

When implementation behavior changes:
1. Update this specification if contract-level semantics changed.
2. Update corresponding ADRs under `docs/adr/` for architectural decisions.
3. Update `docs/schema.json` and schema version constants for breaking envelope changes.
4. Add migration notes in changelog/user-facing docs as needed.
