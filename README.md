# tokmd

> Deterministic repo receipts, analysis, and review artifacts for humans, CI, and LLM workflows.

[![Crates.io](https://img.shields.io/crates/v/tokmd)](https://crates.io/crates/tokmd)
[![GitHub Release](https://img.shields.io/github/v/release/EffortlessMetrics/tokmd?display_name=tag)](https://github.com/EffortlessMetrics/tokmd/releases)
[![Docs.rs](https://img.shields.io/docsrs/tokmd)](https://docs.rs/tokmd)
[![CI](https://github.com/EffortlessMetrics/tokmd/actions/workflows/ci.yml/badge.svg)](https://github.com/EffortlessMetrics/tokmd/actions/workflows/ci.yml)
[![License](https://img.shields.io/crates/l/tokmd)](https://crates.io/crates/tokmd)
[![Downloads](https://img.shields.io/crates/d/tokmd)](https://crates.io/crates/tokmd)

`tokmd` turns a source tree into stable receipts you can diff, analyze, archive, gate, and pack for downstream automation. It starts with code inventory, then keeps going: saved artifacts, derived metrics, PR review surfaces, policy checks, sensor outputs, and browser-safe context bundles.

## The Problem

Raw LOC counts are easy to produce and hard to reuse.

- Terminal output is awkward to diff and archive.
- CI needs artifacts and gates, not screenshots of a table.
- LLM workflows need bounded, deterministic context instead of pasted summaries.
- Review workflows need stable before/after surfaces, not one-off shell output.

`tokmd` exists to turn repository shape into repeatable, machine-friendly truth.

## What `tokmd` Gives You

- Deterministic receipts for `lang`, `module`, `export`, `run`, `diff`, and `analyze`.
- Review surfaces such as `cockpit`, `gate`, `baseline`, and `sensor`.
- Saved artifacts in Markdown, JSON, JSONL, CSV, CycloneDX, HTML, SVG, Mermaid, and tree formats.
- LLM-oriented workflows through `context`, `handoff`, redaction, token budgeting, and tool-schema generation.
- Multiple product surfaces: CLI, Rust facade, Python bindings, Node bindings, and a browser/WASM slice.

## Start Here

Install:

```bash
cargo install tokmd --locked
# or
nix run github:EffortlessMetrics/tokmd -- --version
```

Run the common paths:

```bash
# Summarize the current repo
tokmd --format md --top 8

# Save a deterministic run directory for CI or later diffing
tokmd run --analysis receipt --output-dir .runs/current

# Compare two states
tokmd diff main HEAD

# Generate a risk-oriented analysis view
tokmd analyze --preset risk --format md

# Pack code for an LLM budget
tokmd context --budget 128k --mode bundle --output context.txt
```

## Choose a Path

| If you need to... | Start with... | Typical output |
| :---------------- | :------------ | :------------- |
| summarize a repo or PR | `tokmd`, `diff`, `cockpit` | Markdown summary, diff tables, review plan |
| save deterministic artifacts | `run`, `export` | JSON/JSONL/CSV/CycloneDX receipts |
| analyze code health or risk | `analyze` | Markdown, JSON, HTML, SVG, Mermaid |
| estimate effort between refs | `analyze --preset estimate` | effort report with optional base/head delta |
| gate policy in CI | `gate`, `baseline`, `sensor` | policy verdicts, ratchets, `sensor.report.v1` |
| pack context for an LLM | `context`, `handoff` | bounded bundle text, JSON receipts, handoff directory |

## What It Looks Like

These are live GitHub Actions badges from this repository:

[![CI](https://github.com/EffortlessMetrics/tokmd/actions/workflows/ci.yml/badge.svg)](https://github.com/EffortlessMetrics/tokmd/actions/workflows/ci.yml)
[![Release Workflow](https://github.com/EffortlessMetrics/tokmd/actions/workflows/release.yml/badge.svg)](https://github.com/EffortlessMetrics/tokmd/actions/workflows/release.yml)

Example summary output from this repository:

```md
|Lang|Code|Lines|Bytes|Tokens|
|---|---:|---:|---:|---:|
|Rust|377263|470341|15334354|3833170|
|JSON|5405|5405|284012|70997|
|Markdown|3067|17273|567919|141930|
|JavaScript|1979|2233|64463|16111|
|TOML|1947|2387|71514|17855|
|Other|1978|2758|78806|19691|
|**Total**|391639|500397|16401068|4099754|
```

## Generated Badges

The badges above are GitHub-hosted workflow badges. `tokmd badge` produces repo-local SVG badges from your own code data:

```bash
tokmd badge --metric lines --output badge-lines.svg
tokmd badge --metric hotspot --preset risk --output badge-hotspot.svg
```

Embed them in your own README:

```markdown
![Lines of Code](badge-lines.svg)
![Hotspot](badge-hotspot.svg)
```

## Command Surface

| Command | Purpose |
| :------ | :------ |
| `tokmd` | Language summary for a repo or path set |
| `tokmd module` | Group stats by module roots such as `crates/` or `packages/` |
| `tokmd export` | File-level dataset for downstream pipelines |
| `tokmd run` | Save a full receipt set to a run directory |
| `tokmd analyze` | Derived metrics and enrichments |
| `tokmd badge` | Render SVG badges from receipt metrics |
| `tokmd diff` | Compare two runs, receipts, or refs deterministically |
| `tokmd context` | Pack code into an LLM context window |
| `tokmd handoff` | Build an LLM handoff bundle |
| `tokmd cockpit` | PR-review metrics with risk and evidence gates |
| `tokmd gate` | Evaluate TOML policy rules and ratchets |
| `tokmd baseline` | Capture a baseline for later ratchet comparisons |
| `tokmd sensor` | Emit a `sensor.report.v1` envelope |
| `tokmd tools` | Generate tool definitions for OpenAI, Anthropic, and JSON Schema consumers |
| `tokmd init` | Generate a `.tokeignore` template |
| `tokmd check-ignore` | Explain why a path is being ignored |
| `tokmd completions` | Generate shell completions |

## Browser And WASM

`tokmd-wasm` and `web/runner` expose a narrower browser-safe slice of the product.

- Supported today: `lang`, `module`, `export`, and browser-safe `analyze` presets on ordered in-memory inputs.
- Public repo ingestion uses GitHub tree and contents APIs, not zipball-first fetches.
- Git-history enrichers and full native filesystem flows remain native-first.

## What `tokmd` Is Not

- It is not a formatter, linter, or build system.
- It is not a developer-scoring tool.
- It is not a vulnerability database or SAST replacement.
- It does not ask you to trust prose where a receipt can be emitted instead.

## Go Deeper

### Tutorial

- [Tutorial](docs/tutorial.md) for first-run setup and basic workflows

### How-To

- [Recipes](docs/recipes.md) for practical usage patterns
- [Troubleshooting](docs/troubleshooting.md) for common problems and fixes
- [Contributing](CONTRIBUTING.md) for local development and release work

### Reference

- [CLI Reference](docs/reference-cli.md) for flags, formats, and config
- [Schema](docs/SCHEMA.md) for receipt contracts
- [tokmd responsibilities](tokmd-role.md) for the wider sensor/receipt stack

### Explanation

- [Philosophy](docs/explanation.md) for the design stance
- [Architecture](docs/architecture.md) for the crate graph and boundaries
- [Design](docs/design.md) for system concepts and invariants
- [Roadmap](ROADMAP.md) for the active horizon

## License

MIT or Apache-2.0.
