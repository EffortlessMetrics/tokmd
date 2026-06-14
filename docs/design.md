# tokmd Design

## Design Principles

### 1. Receipts Are the Bus
Schemaed outputs are the record, not logs. Every operation produces a versioned, machine-verifiable receipt.

### 2. Determinism Is UX
Stable ordering and budgets prevent "comment churn" in PR workflows:
- Same inputs → byte-identical outputs
- Explicit truncation markers, not silent drops
- Normalized paths regardless of OS

### 3. Signals, Not Scores
Analysis provides information, not judgments:
- "Doc density is 12%" — not "Documentation is poor"
- "File changed 47 times" — not "This is a problem file"
- Users interpret signals in their context

### 4. Shape, Not Grade
tokmd is a sensor for inventory, distribution, risk signals, and blast radius. It is explicitly **not** a productivity metric tool.

### 5. One Scan, Many Views
Run the scan once. Derive all views (lang, module, export, analysis) from that single source of truth.

### 6. Progressive Disclosure
- Quick scans return fast summaries
- Deep analysis is opt-in via presets
- Feature flags control compilation footprint

### 7. No Green By Omission
The `capabilities` field explicitly reports which checks were available, unavailable, or skipped. This lets directors distinguish "all checks passed" from "nothing ran" instead of treating unavailable evidence as success.

## System Context

### Standalone Mode

```
┌─────────────────────────────────────────────────────────────┐
│                      tokmd                                  │
│                                                             │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌──────────┐    │
│  │   CLI    │  │  Python  │  │  Node.js │  │  Library │    │
│  │ (tokmd)  │  │ Bindings │  │ Bindings │  │   API    │    │
│  └────┬─────┘  └────┬─────┘  └────┬─────┘  └────┬─────┘    │
│       │             │             │             │           │
│       └─────────────┴─────────────┴─────────────┘           │
│                           │                                 │
│                    ┌──────┴──────┐                         │
│                    │  tokmd-core │                         │
│                    │  (facade)   │                         │
│                    └──────┬──────┘                         │
│                           │                                 │
│  ┌────────────────────────┴────────────────────────┐       │
│  │                                                  │       │
│  │  tokmd-scan → tokmd-model → tokmd-format        │       │
│  │       ↓                          ↓              │       │
│  │  tokmd-analysis → tokmd-format::analysis        │       │
│  │                                                  │       │
│  └──────────────────────────────────────────────────┘       │
└─────────────────────────────────────────────────────────────┘
                              ↓
                     Receipts (JSON/JSONL/CSV)
```

## Data Model

### Receipt Envelope

Every JSON receipt includes:
```json
{
  "schema_version": 2,
  "tool": "tokmd",
  "tool_version": "1.11.0",
  "generated_at_ms": 1706886000000,
  "mode": "lang",
  "scan": { ... },
  "totals": { ... },
  "rows": [ ... ],
  "integrity": "blake3:..."
}
```

tokmd is a **code-evidence producer**: it produces receipts, review packets,
proof-routing inputs, and context bundles, not the whole evidence backplane.
External directors and `evidencebus` can aggregate tokmd receipts with evidence
from `mergecode`, CI sensors, gates, perf tools, and other producers.

### Schema Versioning

Separate versions per receipt family:
- Core receipts: `SCHEMA_VERSION = 2`
- Analysis receipts: `ANALYSIS_SCHEMA_VERSION = 9`
- Cockpit receipts: `COCKPIT_SCHEMA_VERSION = 3`
- Handoff manifests: `HANDOFF_SCHEMA_VERSION = 5`
- Context receipts: `CONTEXT_SCHEMA_VERSION = 4`
- Context bundles: `CONTEXT_BUNDLE_SCHEMA_VERSION = 2`
- Tool schemas: `TOOL_SCHEMA_VERSION = 1`
- Sensor reports: semantic schema id `sensor.report.v1`

Evolution rules:
- Additive changes within vN (new optional fields)
- Breaking changes require vN+1 with migration notes

### Determinism Guarantees

1. **Ordered structures**: `BTreeMap`/`BTreeSet` at all boundaries
2. **Stable sorting**: Descending by code lines, then ascending by name
3. **Path normalization**: Forward slashes (`/`) regardless of OS
4. **Redaction determinism**: Same input → same BLAKE3 hash

## Adapter Boundaries

### Scanning Adapter (tokmd-scan)

Wraps tokei library:
```
ScanOptions → tokei Config
tokei Languages → tokmd receipts (via tokmd-model)
```

### Git Adapter (tokmd-git)

Uses shell `git log` (not git2):
```
git log --numstat → CommitHistory
Respects --max-commits, --max-commit-files
```

### Content Helpers (tokmd-analysis::content)

File content analysis:
```
File bytes → entropy (Shannon bits/byte)
File bytes → tag counts (TODO, FIXME)
File bytes → BLAKE3 hash
```

### Walk Helpers (tokmd-scan::walk)

Filesystem traversal:
```
Tries git ls-files first
Falls back to ignore crate
Respects .gitignore, .tokeignore
```

### I/O Port Contract (tokmd-io-port)

Host-abstracted file access for in-memory and WASM execution:
```
ReadFs trait → HostFs (native std::fs)
             → MemFs (tests / in-memory substrates)
```

## Analysis Architecture

### Preset System

Presets bundle enrichers for common use cases:

| Preset | Enrichers |
|--------|-----------|
| `receipt` | derived + dup + git + complexity + API surface |
| `estimate` | `receipt` + effort estimation and optional base/head delta |
| `bun-ub` | scoped bun-ub review evidence |
| `health` | derived + content (TODOs) + complexity + Halstead |
| `risk` | `health` + git (hotspots, coupling, freshness) |
| `supply` | derived + walk (assets) + content (deps) |
| `architecture` | derived + content (imports) |
| `topics` | semantic topic clouds (TF-IDF) |
| `security` | license radar + entropy profiling |
| `identity` | archetype detection + corporate fingerprint |
| `git` | predictive churn + advanced git metrics |
| `deep` | all enrichers (except fun) |
| `fun` | eco-label, novelty outputs |

### Feature-Gated Enrichers

```rust
#[cfg(feature = "git")]
fn run_git_enrichers() { ... }

#[cfg(feature = "content")]
fn run_content_enrichers() { ... }

#[cfg(feature = "walk")]
fn run_walk_enrichers() { ... }
```

## Error Handling

### Error Types

- **ScanError**: File access, tokei failures
- **AnalysisError**: Enricher failures
- **GateError**: Policy evaluation failures
- **ConfigError**: Configuration parsing failures

### Failure Modes

1. **Graceful degradation**: Missing optional inputs → skip verdict
2. **Partial receipts**: Runtime errors → emit what's available
3. **Explicit failures**: Policy violations → exit code 2

## Budgets and Truncation

All PR-facing outputs are budgeted:
- Max highlights per section
- Max files in review plan
- Stable truncation indicators

Context packing respects token budgets:
```
--budget 128k → Select files that fit
--strategy greedy|spread → Selection order
--rank-by code|tokens|churn|hotspot → File priority signal
Explicit [truncated] markers
```

## Testing Strategy (Design Level)

### Invariant Classes

1. **Determinism**: Same inputs → same outputs
2. **Idempotency**: Repeated operations → same results
3. **Composition**: Filters compose predictably
4. **Monotonicity**: More input → proportionally more output

### Test Boundaries

- **Unit tests**: Domain logic (sorting, aggregation, hash computation)
- **Integration tests**: CLI contract (flags, outputs, exit codes)
- **Golden tests**: Output format stability
- **Property tests**: Invariant verification
- **Fuzz tests**: Parser robustness
- **Mutation tests**: Test quality verification
