# tokmd Requirements

## Purpose

tokmd produces deterministic code inventory receipts and PR-focused context for:
- PR descriptions and code review
- CI artifacts and diffs
- LLM workflows (context packing + tool schemas)

## Primary User Stories

### Reviewer-Shaped
1. **Change surface clarity**: Given base/head refs, show what changed and where risk is concentrated
2. **Risk indicators**: List top files needing review with reasons (hotspots, complexity, coupling)
3. **Evidence gates**: Surface mutation testing, coverage, contract changes as hard gates

### Automation-Shaped
1. **Stable receipts**: Same inputs → byte-identical JSON/JSONL/CSV artifacts
2. **Machine-verifiable**: Schema-versioned outputs with integrity hashes
3. **Diffable**: Receipt comparison for trend tracking

### LLM-Shaped
1. **Context packing**: Budget-aware file selection within token limits
2. **Tool schemas**: Generate OpenAI/Anthropic/JSON Schema tool definitions
3. **Inventory-first**: Provide map before dump (what languages, which modules, will it fit?)

## Interfaces

### CLI (Stable Surfaces)

| Command | Purpose |
|---------|---------|
| `lang` | Language summary table |
| `module` | Module breakdown by directory |
| `export` | File-level inventory (JSONL/CSV/JSON/CycloneDX) |
| `run` | Full scan with artifact output |
| `diff` | Compare two receipts |
| `analyze` | Derived metrics with preset system |
| `cockpit` | PR metrics with evidence gates |
| `gate` | Policy evaluation over receipts |
| `context` | LLM context packing within budget |
| `tools` | LLM tool definition generation |
| `badge` | SVG metric badges |
| `init` | `.tokeignore` template generation |
| `check-ignore` | Explain ignored files |
| `completions` | Shell completions |

### Receipt Contracts

Output MUST be stable and schemaed when `--format json/jsonl/csv`:
- **Core receipts** (lang, module, export, diff, context, run): Schema v2
- **Analysis receipts**: Schema v4
- **Cockpit receipts**: Schema v3

### Library API (tokmd-core)

Clap-free facade for embedding:
- Workflow functions: `lang_workflow()`, `module_workflow()`, `export_workflow()`, `diff_workflow()`
- FFI layer: `run_json(mode, args_json) -> String`
- Python/Node bindings wrap FFI

## Determinism Requirements

Same inputs MUST produce byte-stable outputs:
- Ordered structures (BTreeMap, BTreeSet)
- Stable sorting (code lines desc, name asc)
- Path normalization (forward slashes)
- Stable truncation with explicit markers

## Failure Semantics

| Scenario | Exit Code | Behavior |
|----------|-----------|----------|
| Success | 0 | Full receipt |
| Tool/runtime error | 1 | Partial receipt when possible |
| Policy failure | 2 | Receipt with failure reason |
| Missing optional input | — | Skip verdict, not silent pass |

## Performance Requirements

- Repo truth commands: Sub-second to low seconds on typical workspaces
- Avoid unnecessary allocations in hot loops
- Use feature flags to keep dependency surface minimal
- Stream where feasible (JSONL exports)

## Security and Privacy

- Redaction is first-class (`--redact paths|all`)
- Redaction MUST be deterministic (same input → same BLAKE3 hash)
- No absolute paths in shareable receipts
- No network access by default
- Resource limits: `--max-files`, `--max-bytes`, `--max-commits`

## Compatibility

- Cross-platform: Linux/macOS/Windows
- Nix-native workflows supported (flake checks)
- Schema evolution: Additive within vN, breaking requires vN+1

## Quality Bar

tokmd is a gatekeeper-class tool. Minimum evidence:
- Unit tests for domain logic
- Integration tests for CLI contract
- Golden tests for output rendering
- Property tests for determinism invariants
- Fuzz testing for parsers and path handling
- Mutation testing gates for critical crates

## Ecosystem Integration (Planned v1.5+)

tokmd SHOULD support integration with multi-sensor cockpit systems:

### Role
- tokmd is a **sensor**, not a director
- Integration via **receipts**, not library linking
- tokmd MUST remain useful standalone

### Ecosystem Envelope
- `tokmd sensor cockpit` emits standardized envelope at `artifacts/tokmd/report.json`
- Envelope provides verdict + findings + minimal metadata
- tokmd-native cockpit receipt embedded under `data` field
- See [ecosystem-envelope.md](ecosystem-envelope.md) for specification

### Finding Identity
- Findings MUST use stable namespaced IDs: `<tool>.<category>.<code>`
- Example: `tokmd.risk.hotspot`, `tokmd.gate.mutation_failed`
- Finding registry documented in ecosystem-envelope.md

### Budget Compliance
- Respect `--findings-limit` for display budget enforcement
- Stable truncation with explicit markers
- Directors aggregate; sensors respect allocation

## Non-Goals

- **Not a director**: tokmd is a sensor, not an orchestrator
- **Not a linter**: Use rustfmt, eslint for formatting
- **Not a vulnerability scanner**: Use cargo-audit, npm audit
- **Not a test runner**: Use cargo test, pytest
- **Not a scorer**: Provides signals, not judgments
