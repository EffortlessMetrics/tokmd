# sensor.report.v1

Standardized sensor report format for multi-sensor CI integration.

## Overview

The `sensor.report.v1` protocol defines a JSON envelope format that allows code quality sensors to integrate with external orchestrators ("directors") that aggregate reports from multiple sensors into a unified PR view.

## Design Principles

1. **Stable top-level, rich underneath**: Minimal stable envelope; tool-specific richness in `data`
2. **Verdict-first**: Quick pass/fail/warn determination without parsing tool-specific data
3. **Findings are portable**: Common finding structure for cross-tool aggregation
4. **Self-describing**: Schema version and tool metadata enable forward compatibility
5. **No Green By Omission**: Capabilities block distinguishes "all passed" from "nothing ran"

## Top-Level Fields

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `schema` | string | Yes | Always `"sensor.report.v1"` |
| `tool` | object | Yes | Tool identification (name, version, mode) |
| `generated_at` | string | Yes | ISO 8601 timestamp |
| `verdict` | string | Yes | Overall result: `pass`, `fail`, `warn`, `skip`, `pending` |
| `summary` | string | Yes | Human-readable one-line summary |
| `findings` | array | Yes | List of findings (may be empty) |
| `artifacts` | array | No | Related artifact paths |
| `capabilities` | object | No | Capability availability status |
| `data` | object | No | Tool-specific payload (opaque to director) |

## Verdict Aggregation

Directors aggregate verdicts using this precedence (highest first):
1. `fail` - Hard failure (evidence gate failed, policy violation)
2. `pending` - Awaiting external data (CI artifacts, etc.)
3. `warn` - Soft warnings present, review recommended
4. `pass` - All checks passed, no significant findings
5. `skip` - Sensor skipped (missing inputs, not applicable)

## Findings

Findings use a `(check_id, code)` tuple for identity. Combined with `tool.name`, this forms the triple `(tool, check_id, code)` used for routing and policy.

Example: `("tokmd", "risk", "hotspot")` identifies a hotspot finding from tokmd.

### Severity Levels

| Severity | Description |
|----------|-------------|
| `error` | Blocks merge (hard gate failure) |
| `warn` | Review recommended |
| `info` | Informational, no action required |

## Capabilities (No Green By Omission)

The `capabilities` field explicitly reports which checks were available, unavailable, or skipped. This enables directors to distinguish between:
- "All checks passed" (capabilities available, verdict pass)
- "Nothing ran" (capabilities unavailable/skipped)

```json
{
  "capabilities": {
    "mutation": { "status": "available" },
    "coverage": { "status": "unavailable", "reason": "no coverage artifact" },
    "semver": { "status": "skipped", "reason": "no API files changed" }
  }
}
```

## Example Usage

### Producing a Report (tokmd)

```bash
# Run tokmd cockpit in sensor mode
tokmd cockpit --sensor-mode --artifacts-dir artifacts/tokmd

# Output: artifacts/tokmd/report.json (sensor.report.v1 envelope)
# Output: artifacts/tokmd/comment.md (PR comment markdown)
```

### Consuming a Report (director)

```javascript
const report = JSON.parse(fs.readFileSync('artifacts/tokmd/report.json'));

// Quick verdict check
if (report.verdict === 'fail') {
  console.log('Sensor failed:', report.summary);
}

// Aggregate findings across sensors
for (const finding of report.findings) {
  const id = `${report.tool.name}.${finding.check_id}.${finding.code}`;
  console.log(`${id}: ${finding.title}`);
}

// Check capabilities for "No Green By Omission"
for (const [name, cap] of Object.entries(report.capabilities || {})) {
  if (cap.status === 'unavailable') {
    console.warn(`Capability ${name} unavailable: ${cap.reason}`);
  }
}
```

## Files in This Contract

- `schema.json` - JSON Schema Draft 7 definition
- `examples/pass.json` - Example passing envelope
- `examples/fail.json` - Example failing envelope

## Versioning

The schema identifier `sensor.report.v1` is immutable. Breaking changes require a new version (`sensor.report.v2`).

Non-breaking additions (new optional fields) are allowed within the same version.
