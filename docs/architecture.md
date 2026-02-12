# tokmd Architecture

This document describes the internal architecture of tokmd for contributors and library consumers.

See also: [tokmd responsibilities](../tokmd-role.md) - tokmd's position in the sensors -> receipts -> cockpit stack.

## Design Principles

1. **Receipts are the bus**: Schemaed outputs are the record, not logs
2. **Determinism is UX**: Stable ordering and budgets prevent "comment churn"
3. **Truth-layer discipline**: tokmd stays repo/diff truth; build-truth consumers live elsewhere
4. **One scan, many views**: Single scan produces lang/module/export/analysis views

## Crate Hierarchy

tokmd follows a tiered microcrate architecture with strict dependency rules.

```
Tier 0 (Contracts)     tokmd-types, tokmd-analysis-types, tokmd-settings,
                       tokmd-envelope, tokmd-substrate
         ↓
Tier 1 (Core)          tokmd-scan, tokmd-model, tokmd-redact, tokmd-tokeignore,
                       tokmd-sensor
         ↓
Tier 2 (Adapters)      tokmd-format, tokmd-walk, tokmd-content, tokmd-git
         ↓
Tier 3 (Orchestration) tokmd-analysis, tokmd-analysis-format, tokmd-fun, tokmd-gate
         ↓
Tier 4 (Facade)        tokmd-config, tokmd-core
         ↓
Tier 5 (Products)      tokmd (CLI), tokmd-python, tokmd-node
```

### Tier 0: Contracts (Pure Data)

| Crate | Purpose | Dependencies |
|-------|---------|--------------|
| `tokmd-types` | Core receipt DTOs (`LangRow`, `ModuleRow`, `FileRow`, `Totals`) | `serde` only |
| `tokmd-analysis-types` | Analysis receipt DTOs | `serde`, `tokmd-types` |
| `tokmd-settings` | Clap-free settings types (`ScanOptions`, `LangSettings`, etc.) | `serde`, `tokmd-types` |
| `tokmd-envelope` | Cross-fleet `SensorReport` contract (`Verdict`, `Finding`, `GateResults`) | `serde`, `serde_json` |
| `tokmd-substrate` | Shared repo context (`RepoSubstrate`, `SubstrateFile`, `DiffRange`) | `serde` only |

**Schema Versions** (separate per family):
- Core receipts: `SCHEMA_VERSION = 2` (lang, module, export, diff, context, run)
- Analysis receipts: `ANALYSIS_SCHEMA_VERSION = 5`
- Cockpit receipts: `SCHEMA_VERSION = 3`

### Tier 1: Core Processing

| Crate | Purpose |
|-------|---------|
| `tokmd-scan` | Wraps tokei library for code scanning |
| `tokmd-model` | Aggregation logic: tokei results → tokmd receipts |
| `tokmd-redact` | BLAKE3-based path hashing and redaction |
| `tokmd-tokeignore` | `.tokeignore` template generation |
| `tokmd-sensor` | `EffortlessSensor` trait + `build_substrate()` builder |

### Tier 2: Adapters

| Crate | Purpose | Feature Flag |
|-------|---------|--------------|
| `tokmd-format` | Output rendering (Markdown, TSV, JSON, CSV, JSONL, CycloneDX) | — |
| `tokmd-walk` | Filesystem traversal with gitignore support | `walk` |
| `tokmd-content` | File content scanning (entropy, tags, hashing) | `content` |
| `tokmd-git` | Git history analysis via shell `git log` | `git` |

### Tier 3: Orchestration

| Crate | Purpose |
|-------|---------|
| `tokmd-analysis` | Analysis orchestration with preset system |
| `tokmd-analysis-format` | Analysis output rendering (Markdown, JSON, SVG, HTML, etc.) |
| `tokmd-fun` | Novelty outputs (eco-label, MIDI, OBJ) |
| `tokmd-gate` | Policy evaluation with JSON pointer rules |

### Tier 4: Facade

| Crate | Purpose |
|-------|---------|
| `tokmd-config` | CLI parsing (clap) + configuration loading |
| `tokmd-core` | Library facade with FFI layer |

### Tier 5: Products

| Crate | Purpose |
|-------|---------|
| `tokmd` | CLI binary |
| `tokmd-python` | PyO3 bindings for Python |
| `tokmd-node` | napi-rs bindings for Node.js |

## Dependency Rules

1. **Contracts MUST NOT depend on clap** — Keep `tokmd-types` and `tokmd-analysis-types` pure
2. **Lower tiers MUST NOT depend on higher tiers** — No upward dependencies
3. **Feature flags control optional adapters** — `git`, `walk`, `content` features
4. **IO adapters depend on domain/contracts, never reverse**

## Data Flow

### Flow A: Repository Inventory (lang/module/export)

```
Filesystem → tokmd-walk → tokmd-scan (tokei) → tokmd-model → tokmd-format → Output
                                ↓
                          BTreeMap (deterministic)
                                ↓
                    Receipt DTOs (tokmd-types)
```

### Flow B: Analysis (analyze/cockpit)

```
Receipt → tokmd-analysis → Enrichers → tokmd-analysis-format → Output
              ↓
    ┌─────────┴─────────┐
    ↓                   ↓
Optional:           Core:
- tokmd-git         - archetype
- tokmd-content     - derived
- tokmd-walk        - topics
```

### Flow C: Sensor Integration (tokmd-sensor)

```
ScanOptions → build_substrate() → RepoSubstrate (shared context)
                                       ↓
                            ┌──────────┴──────────┐
                            ↓                     ↓
                     Sensor A.run()         Sensor B.run()
                            ↓                     ↓
                      SensorReport          SensorReport
                            ↓                     ↓
                            └──────────┬──────────┘
                                       ↓
                              Director aggregates
```

"Substrate once, sensors many" — the scan runs once, then each `EffortlessSensor`
receives the same `RepoSubstrate` and produces a standardized `SensorReport` envelope.

### Flow D: Library API (tokmd-core)

```
Settings → Workflow Functions → Receipt → JSON
    ↓
run_json(mode, args_json) ─→ {"ok": true, "data": {...}}
    ↓
Python/Node bindings wrap FFI layer
```

## Determinism Guarantees

tokmd guarantees byte-stable output for identical inputs:

1. **Ordered structures**: `BTreeMap` and `BTreeSet` at all boundaries
2. **Stable sorting**: Descending by code lines, then ascending by name
3. **Path normalization**: Forward slashes (`/`) regardless of OS
4. **Timestamp normalization**: `generated_at_ms` normalized in tests
5. **Redaction determinism**: Same input → same BLAKE3 hash

## Error Handling

| Scenario | Exit Code | Receipt |
|----------|-----------|---------|
| Success | 0 | Full receipt |
| Tool/runtime error | 1 | Partial receipt when possible |
| Policy failure (gate) | 2 | Receipt with failure reason |
| Missing optional input | — | `skip` verdict with `missing_input` reason |

## Feature Flags

```toml
[features]
git = ["tokmd-git"]      # Git history analysis
content = ["tokmd-content"]  # File content scanning
walk = ["tokmd-walk"]    # Filesystem traversal
ui = ["dialoguer", "indicatif"]  # Interactive CLI
```

## Publishing Matrix

### Published to crates.io
- `tokmd` (binary)
- `tokmd-types`, `tokmd-analysis-types` (contracts)
- Core crates as stable

### Workspace-only (publish = false)
- `tokmd-python` (published to PyPI via maturin)
- `tokmd-node` (published to npm via napi-rs)
- Clap-facing argument models
- UI affordances

## Sensor Integration Architecture

The sensor subsystem enables multi-sensor pipelines where tokmd acts as one sensor
among many (cargo-deny, cargo-audit, etc.) in a CI/CD fleet.

### Key Crates

| Crate | Role |
|-------|------|
| `tokmd-substrate` | Shared scan context (files, languages, diff range) — built once |
| `tokmd-envelope` | Standardized report contract (`sensor.report.v1`) |
| `tokmd-settings` | Clap-free settings for library/FFI consumers |
| `tokmd-sensor` | `EffortlessSensor` trait + substrate builder |

### Design Principles

1. **Substrate once, sensors many**: A single I/O pass builds `RepoSubstrate`, eliminating redundant scans
2. **Standardized envelope**: All sensors emit `SensorReport` with findings, verdicts, and gates
3. **Clap-free settings**: Lower-tier crates use `ScanOptions` from `tokmd-settings`, not `GlobalArgs`
4. **Finding identity**: `(check_id, code)` tuples enable category-based routing for buildfix automation
