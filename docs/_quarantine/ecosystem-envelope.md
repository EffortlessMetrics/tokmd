# Ecosystem Envelope Specification

> **Status**: Planned (v2.0+)
>
> This document specifies a standardized report envelope for integrating tokmd with multi-sensor cockpit systems.

## Overview

The ecosystem envelope is a standardized JSON format that allows tokmd to integrate with external orchestrators ("directors") that aggregate reports from multiple code quality sensors into a unified PR view.

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

## Envelope Schema (v1)

```json
{
  "$schema": "https://tokmd.dev/schemas/envelope.v1.json",
  "schema": "sensor.report.v1",
  "tool": {
    "name": "tokmd",
    "version": "1.5.0",
    "mode": "cockpit"
  },
  "generated_at": "2026-02-02T12:00:00Z",
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
      },
      "evidence": {
        "commits": 47,
        "window_days": 90
      }
    }
  ],
  "gates": {
    "status": "pending",
    "items": [
      {
        "id": "mutation",
        "status": "pending",
        "reason": "CI artifact not found"
      }
    ]
  },
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
  "data": {
    // Full tokmd-native cockpit receipt embedded here
    // Schema: tokmd cockpit v3
  }
}
```

## Field Definitions

### Top-Level Fields

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `schema` | string | Yes | Schema identifier (e.g., `"sensor.report.v1"`) |
| `tool` | object | Yes | Tool identification |
| `generated_at` | string (ISO 8601) | Yes | Generation timestamp |
| `verdict` | enum | Yes | Overall result: `pass`, `fail`, `warn`, `skip`, `pending` |
| `summary` | string | Yes | Human-readable one-line summary |
| `findings` | array | Yes | List of findings (may be empty) |
| `gates` | object | No | Evidence gate status |
| `artifacts` | array | No | Related artifact paths |
| `data` | object | No | Tool-specific payload (opaque to director) |

### Tool Object

```json
{
  "name": "tokmd",
  "version": "1.5.0",
  "mode": "cockpit"
}
```

### Verdict Enum

| Value | Meaning |
|-------|---------|
| `pass` | All checks passed, no significant findings |
| `fail` | Hard failure (evidence gate failed, policy violation) |
| `warn` | Soft warnings present, review recommended |
| `skip` | Sensor skipped (missing inputs, not applicable) |
| `pending` | Awaiting external data (CI artifacts, etc.) |

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
    "line": null,
    "column": null
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

tokmd findings:
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
| `gate` | `mutation_failed` | fail | Mutation testing threshold not met |
| `gate` | `coverage_failed` | fail | Diff coverage threshold not met |

#### Severity Levels

| Level | Meaning |
|-------|---------|
| `error` | Blocks merge (hard gate failure) |
| `warn` | Review recommended |
| `info` | Informational, no action required |

### Gates Object

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

## Artifact Paths

Canonical output location: `artifacts/<tool>/`

For tokmd:
```
artifacts/
└── tokmd/
    ├── report.json      # Ecosystem envelope
    ├── cockpit.json     # Full tokmd-native receipt
    ├── comment.md       # PR comment markdown
    └── badge.svg        # Optional badge
```

## CLI Integration

### Sensor Mode

New command to emit ecosystem envelope:

```bash
# Emit envelope to artifacts/tokmd/report.json
tokmd sensor cockpit --base main --head HEAD --output artifacts/tokmd/

# With explicit artifact directory
tokmd sensor cockpit --base v1.3.0 --head v1.4.0 --artifacts-dir ./ci-artifacts/
```

### Options

| Flag | Description |
|------|-------------|
| `--output DIR` | Output directory (default: `artifacts/tokmd/`) |
| `--findings-limit N` | Max findings in envelope (default: 20) |
| `--embed-data` | Embed full tokmd receipt in `data` field (default: true) |
| `--no-embed-data` | Reference receipt via `artifacts` instead of embedding |

## Director Integration

### Aggregation Rules

Directors should:
1. Collect `report.json` from each sensor's artifact directory
2. Aggregate verdicts: `fail` > `pending` > `warn` > `pass` > `skip`
3. Merge findings with deduplication by `id`
4. Respect per-tool `findings_limit` to prevent flood
5. Generate unified PR comment from aggregated data

### Budget Enforcement

Directors enforce display budgets:
```toml
# cockpit.toml (director config)
[display]
max_findings_total = 50
max_findings_per_tool = 15
max_summary_lines = 10
```

Tools should respect their allocation when `--findings-limit` is passed.

## Versioning

- Envelope schema uses string identifiers: `schema: "sensor.report.v1"`
- Additive changes (new optional fields) stay within v1
- Breaking changes require v2
- Tool-specific `data` follows tool's own schema version

## Migration from tokmd-native Cockpit

Current `tokmd cockpit` output remains unchanged. The envelope is an additional output format:

```bash
# Current (unchanged)
tokmd cockpit --base main --head HEAD --format json > cockpit.json

# New sensor mode (envelope)
tokmd sensor cockpit --base main --head HEAD --output artifacts/tokmd/
```

The `tokmd sensor` subcommand family handles envelope-wrapped outputs for ecosystem integration.

## Future Extensions

### Planned Finding Categories

- `tokmd.security.entropy_high` — High-entropy file (potential secrets)
- `tokmd.security.license_conflict` — License compatibility issue
- `tokmd.architecture.circular_dep` — Circular import detected
- `tokmd.architecture.layer_violation` — Architecture boundary crossed

### Cross-Tool Correlation

Directors may correlate findings across tools:
```json
{
  "correlation": {
    "files": ["src/parser.rs"],
    "tools": ["tokmd", "coverage", "linter"],
    "summary": "Hot file with low coverage and lint warnings"
  }
}
```

This is director-side logic, not part of the envelope spec.
