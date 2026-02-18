# tokmd-envelope

## Purpose

Cross-fleet `SensorReport` contract for multi-sensor integration. This is a **Tier 0** crate.

## Responsibility

- Define the `SensorReport` envelope (`sensor.report.v1`)
- Define `Finding`, `FindingSeverity`, `FindingLocation` types
- Define `Verdict` enum for pass/fail/warn/skip/pending
- Define `GateResults` and `GateItem` for evidence gate sections
- Define `Artifact` for report artifact references
- Define `CapabilityStatus` for "No Green By Omission"
- Provide BLAKE3-based fingerprints for finding deduplication
- Provide builder-pattern API for constructing reports
- **NOT** for tokmd-specific analysis types (use `tokmd-analysis-types`)
- **NOT** for I/O operations or business logic

## Public API

### Core Types

```rust
pub struct SensorReport { schema, tool, generated_at, verdict, summary, findings, artifacts, capabilities, data }
pub struct ToolMeta { name, version, mode }
pub enum Verdict { Pass, Fail, Warn, Skip, Pending }
pub struct Finding { check_id, code, severity, title, message, location, evidence, docs_url, fingerprint }
pub enum FindingSeverity { Error, Warn, Info }
pub struct FindingLocation { path, line, column }
pub struct GateResults { status, items }
pub struct GateItem { id, status, threshold, actual, reason, source, artifact_path }
pub struct Artifact { id, artifact_type, path, mime }
pub struct CapabilityStatus { status, reason }
pub enum CapabilityState { Available, Unavailable, Skipped }
```

### Finding ID Constants

Finding ID constants are in `findings` module (e.g., `findings::risk::HOTSPOT`).

### Builder Pattern

```rust
let report = SensorReport::new(tool, generated_at, verdict, summary)
    .with_artifacts(vec![...])
    .with_data(json!({...}))
    .with_capabilities(caps);

let finding = Finding::new(check_id, code, severity, title, message)
    .with_location(loc)
    .with_evidence(json!({...}))
    .with_fingerprint("tokmd");
```

## Implementation Details

- BLAKE3 fingerprints use `(tool_name, check_id, code, path)` identity tuple
- Fingerprints are truncated to first 16 bytes (32 hex chars)
- `Verdict` ordering for director aggregation: `fail > pending > warn > pass > skip`
- All optional fields use `#[serde(skip_serializing_if = "Option::is_none")]`
- `BTreeMap` used for capabilities to ensure deterministic ordering

## Dependencies

- `blake3` (fingerprint hashing)
- `serde` / `serde_json` (serialization)

## Testing

```bash
cargo test -p tokmd-envelope
```

Tests cover:
- Serde roundtrips for all types
- Finding fingerprint computation and stability
- Builder pattern for reports, findings, artifacts, gate items
- Verdict and severity Display/serde consistency
- Capability status construction and roundtrip

## Do NOT

- Add tokmd-specific analysis types (use tokmd-analysis-types)
- Add I/O or business logic (this is a pure data contract)
- Change the schema identifier without updating SENSOR_REPORT_SCHEMA
- Modify fingerprint algorithm without considering cross-version compatibility
