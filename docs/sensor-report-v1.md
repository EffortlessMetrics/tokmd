# sensor.report.v1 Protocol Specification

> **Status**: Implemented (v1.5+)
>
> This document specifies the standardized sensor report envelope for integrating tokmd with multi-sensor CI governance systems.

## Overview

The `sensor.report.v1` protocol defines a standardized JSON envelope format that enables tokmd to integrate with external orchestrators ("directors") that aggregate reports from multiple code quality sensors into a unified PR view.

```
┌─────────────────────────────────────────────────────────────┐
│                    Cockpit Director                         │
│                                                             │
│  Aggregates sensor reports into unified PR context          │
│                                                             │
└──────────────┬──────────────┬──────────────┬───────────────┘
               │              │              │
        ┌──────┴──────┐ ┌─────┴─────┐ ┌──────┴──────┐
        │   tokmd     │ │  coverage │ │   linter    │
        │   sensor    │ │   sensor  │ │   sensor    │
        └─────────────┘ └───────────┘ └─────────────┘
               │              │              │
               ▼              ▼              ▼
        report.json     report.json    report.json
        (envelope v1)   (envelope v1)  (envelope v1)
```

## Design Principles

1. **Stable top-level, rich underneath**: The envelope schema is minimal and stable; tool-specific richness lives under `data`
2. **Verdict-first**: Quick pass/fail/warn determination without parsing tool-specific data
3. **Findings are portable**: Common finding structure for cross-tool aggregation
4. **Self-describing**: Schema version and tool metadata enable forward compatibility
5. **No Green By Omission**: Capabilities block explicitly reports what ran vs. what was skipped

## Envelope Schema

The formal JSON Schema is available at:
- `contracts/sensor.report.v1/schema.json` (canonical)
- `crates/tokmd/schemas/sensor.report.v1.schema.json` (for tests)

### Example Envelope

```json
{
  "schema": "sensor.report.v1",
  "tool": {
    "name": "tokmd",
    "version": "1.6.0",
    "mode": "cockpit"
  },
  "generated_at": "2026-02-07T12:00:00Z",
  "verdict": "warn",
  "summary": "3 risk signals, 1 evidence gate pending",
  "findings": [
    {
      "check_id": "risk",
      "code": "hotspot",
      "severity": "warn",
      "title": "High-churn file modified",
      "message": "src/parser.rs has 47 commits in 90 days",
      "location": {
        "path": "src/parser.rs"
      }
    }
  ],
  "artifacts": [
    {
      "type": "comment",
      "path": "artifacts/tokmd/comment.md"
    },
    {
      "type": "receipt",
      "path": "artifacts/tokmd/cockpit.json"
    }
  ],
  "capabilities": {
    "mutation": { "status": "available" },
    "diff_coverage": { "status": "unavailable", "reason": "no coverage artifact found" },
    "contracts": { "status": "available" },
    "determinism": { "status": "skipped", "reason": "no baseline available" }
  },
  "data": {
    "gates": {
      "status": "pending",
      "items": [
        {
          "id": "mutation",
          "status": "pending",
          "reason": "CI artifact not found"
        }
      ]
    }
  }
}
```

## Field Definitions

### Top-Level Fields

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `schema` | string | Yes | Schema identifier: `"sensor.report.v1"` |
| `tool` | object | Yes | Tool identification |
| `generated_at` | string (ISO 8601) | Yes | Generation timestamp |
| `verdict` | enum | Yes | Overall result: `pass`, `fail`, `warn`, `skip`, `pending` |
| `summary` | string | Yes | Human-readable one-line summary |
| `findings` | array | Yes | List of findings (may be empty) |
| `artifacts` | array | No | Related artifact paths |
| `capabilities` | object | No | Capability availability for "No Green By Omission" |
| `data` | object | No | Tool-specific payload (opaque to director) |

### Tool Object

```json
{
  "name": "tokmd",
  "version": "1.6.0",
  "mode": "cockpit"
}
```

### Verdict Enum

| Value | Meaning | Aggregation Priority |
|-------|---------|---------------------|
| `fail` | Hard failure (gate failed, policy violation) | 1 (highest) |
| `pending` | Awaiting external data (CI artifacts, etc.) | 2 |
| `warn` | Soft warnings present, review recommended | 3 |
| `pass` | All checks passed, no significant findings | 4 |
| `skip` | Sensor skipped (missing inputs, not applicable) | 5 (lowest) |

Directors aggregate: highest-priority verdict wins.

### Finding Object

```json
{
  "check_id": "risk",
  "code": "hotspot",
  "severity": "warn",
  "title": "High-churn file modified",
  "message": "src/parser.rs has 47 commits in 90 days",
  "location": {
    "path": "src/parser.rs",
    "line": 42,
    "column": 1
  },
  "evidence": {
    "commits": 47,
    "window_days": 90
  },
  "docs_url": "https://tokmd.dev/findings/risk-hotspot"
}
```

#### Finding Identity

Findings use `(check_id, code)` for identity. Combined with `tool.name`, this forms the triple `(tool, check_id, code)` for buildfix routing.

#### tokmd Finding Registry

| check_id | code | Severity | Description |
|----------|------|----------|-------------|
| `risk` | `hotspot` | warn | High-churn file modified |
| `risk` | `coupling` | warn | High-coupling file modified |
| `risk` | `bus_factor` | warn | Single-author file modified |
| `risk` | `complexity_high` | warn | Cyclomatic complexity > threshold |
| `risk` | `cognitive_high` | warn | Cognitive complexity > threshold |
| `contract` | `schema_changed` | info | Schema version changed |
| `contract` | `api_changed` | warn | Public API surface changed |
| `supply` | `lockfile_changed` | info | Dependency lockfile modified |
| `supply` | `new_dependency` | info | New dependency added |
| `gate` | `mutation_failed` | error | Mutation testing threshold not met |
| `gate` | `coverage_failed` | error | Diff coverage threshold not met |

#### Severity Levels

| Level | Meaning |
|-------|---------|
| `error` | Blocks merge (hard gate failure) |
| `warn` | Review recommended |
| `info` | Informational, no action required |

### Capabilities Object (No Green By Omission)

The `capabilities` field prevents false positives from missing checks. Directors can distinguish between "all checks passed" and "no checks ran".

```json
{
  "mutation": { "status": "available" },
  "diff_coverage": { "status": "unavailable", "reason": "no coverage artifact" },
  "contracts": { "status": "available" },
  "determinism": { "status": "skipped", "reason": "no baseline" }
}
```

#### Capability States

| Status | Meaning |
|--------|---------|
| `available` | Check was available and ran |
| `unavailable` | Check could not run (missing prerequisites) |
| `skipped` | Check was deliberately skipped |

The `reason` field is optional and explains why a capability is unavailable or skipped.

### Gates Object (inside `data`)

Gates are embedded in `data.gates`, not as a top-level field. This keeps the stable envelope surface minimal while allowing tool-specific gate structures.

```json
{
  "status": "pending",
  "items": [
    {
      "id": "mutation",
      "status": "pass",
      "threshold": 80,
      "actual": 85,
      "source": "ci_artifact",
      "artifact_path": ".tokmd/mutation-report.json"
    },
    {
      "id": "diff_coverage",
      "status": "pending",
      "reason": "Waiting for coverage report"
    }
  ]
}
```

Gate IDs:
- `mutation` — Mutation testing results
- `diff_coverage` — Coverage of changed lines
- `contracts` — API/schema contract stability
- `supply_chain` — Dependency audit
- `determinism` — Build reproducibility
- `complexity` — Complexity thresholds

## CLI Integration

### Sensor Mode in Cockpit

The `--sensor-mode` flag enables CI-friendly envelope output:

```bash
# Emit envelope alongside cockpit artifacts
tokmd cockpit --base main --head HEAD --sensor-mode --artifacts-dir artifacts/tokmd/
```

In sensor mode:
- Writes `report.json` (sensor.report.v1 envelope) to artifacts directory
- Always exits 0 if envelope was written successfully
- Uses `verdict` field instead of exit code to signal pass/fail
- Includes `capabilities` block showing what was available

### Standalone Sensor Command

The `tokmd sensor` command emits sensor.report.v1 envelope directly:

```bash
tokmd sensor --base main --head HEAD --output artifacts/tokmd/
```

### Artifact Layout

Canonical output location: `artifacts/<tool>/`

```
artifacts/
└── tokmd/
    ├── report.json      # sensor.report.v1 envelope
    ├── cockpit.json     # Full tokmd-native receipt
    ├── comment.md       # PR comment markdown
    └── badge.svg        # Optional badge
```

## Director Integration

### Aggregation Rules

Directors should:
1. Collect `report.json` from each sensor's artifact directory
2. Aggregate verdicts: `fail` > `pending` > `warn` > `pass` > `skip`
3. Merge findings with deduplication by `(tool, check_id, code)`
4. Check `capabilities` to detect silent failures
5. Generate unified PR comment from aggregated data

### No Green By Omission

Directors MUST check capabilities before treating `verdict: pass` as success:

```python
def is_truly_passing(report):
    if report["verdict"] != "pass":
        return False

    # Check that required capabilities ran
    required = {"mutation", "diff_coverage", "contracts"}
    capabilities = report.get("capabilities", {})

    for cap in required:
        status = capabilities.get(cap, {}).get("status")
        if status != "available":
            return False  # Missing required check

    return True
```

### Budget Enforcement

Directors enforce display budgets:
```toml
# cockpit.toml (director config)
[display]
max_findings_total = 50
max_findings_per_tool = 15
max_summary_lines = 10
```

## Versioning

- Envelope schema uses string identifiers: `schema: "sensor.report.v1"`
- Additive changes (new optional fields) stay within v1
- Breaking changes require v2
- Tool-specific `data` follows tool's own schema version

## Contract Location

The formal JSON Schema and examples are located at:

```
contracts/
└── sensor.report.v1/
    ├── schema.json       # JSON Schema Draft 7
    ├── README.md         # Protocol overview
    └── examples/
        ├── pass.json     # Passing envelope
        └── fail.json     # Failing envelope
```

## See Also

- [SCHEMA.md](SCHEMA.md) — All tokmd receipt schemas
- [reference-cli.md](reference-cli.md) — CLI flag reference
- [contracts/sensor.report.v1/README.md](../contracts/sensor.report.v1/README.md) — Contract documentation
