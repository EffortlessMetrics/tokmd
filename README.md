# tokmd

> Deterministic repository receipts, analysis, review artifacts, and CI gates.

[![Crates.io](https://img.shields.io/crates/v/tokmd)](https://crates.io/crates/tokmd)
[![GitHub Release](https://img.shields.io/github/v/release/EffortlessMetrics/tokmd?display_name=tag)](https://github.com/EffortlessMetrics/tokmd/releases)
[![Docs.rs](https://img.shields.io/docsrs/tokmd)](https://docs.rs/tokmd)
[![CI](https://github.com/EffortlessMetrics/tokmd/actions/workflows/ci.yml/badge.svg)](https://github.com/EffortlessMetrics/tokmd/actions/workflows/ci.yml)
[![License](https://img.shields.io/crates/l/tokmd)](https://crates.io/crates/tokmd)
[![Downloads](https://img.shields.io/crates/d/tokmd)](https://crates.io/crates/tokmd)

`tokmd` turns a source tree into stable Markdown and JSON artifacts: language and module summaries, file receipts, analysis reports, diffs, policy gates, baselines, sensor reports, and LLM-ready context bundles.

Use it from the CLI first; wire the same surfaces into CI with the GitHub Action when you want automated receipts, comments, and gates.

## CLI Quick Start

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

# Save deterministic run artifacts
tokmd run --analysis receipt --output-dir .runs/current

# Compare two states
tokmd diff main HEAD

# Analyze risk
tokmd analyze --preset risk --format md

# Pack code for an LLM budget
tokmd context --budget 128k --mode bundle --output context.txt
```

## What tokmd Produces

| Surface | Output |
| :------ | :----- |
| Repository summary | Markdown tables for languages and modules |
| Receipts | JSON, JSONL, CSV, CycloneDX, HTML, SVG, Mermaid |
| Analysis | Risk, effort, complexity, duplication, git, and API-surface reports |
| Review reports | Cockpit reports, sensor reports, gate verdicts |
| Baselines | Ratchet-ready baseline JSON |
| LLM context | Bounded bundles, redaction, handoff directories |

## Choose a Path

| If you need to... | Start with... | Typical output |
| :---------------- | :------------ | :------------- |
| summarize a repo | `tokmd`, `module`, `export` | Markdown summary, file receipt |
| compare states | `diff`, `run` | deterministic diff and receipts |
| analyze code health | `analyze` | risk, effort, complexity reports |
| review a PR | `cockpit`, GitHub Action | review report |
| gate policy in CI | `gate`, `baseline`, `sensor` | verdicts, ratchets, sensor envelope |
| pack LLM context | `context`, `handoff` | bounded bundle, handoff directory |

## GitHub Action

Use the root composite Action when you want `tokmd` receipts, PR summaries, artifacts, or gates in CI.

```yaml
name: tokmd receipt

on:
  pull_request:

permissions:
  contents: read
  pull-requests: write

jobs:
  receipt:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v6

      - uses: EffortlessMetrics/tokmd@v1
        with:
          version: '1.10.0'
          paths: .
          artifact: 'true'
          comment: 'true'
```

By default, this writes:

- `tokmd-summary.md`
- `tokmd-receipt.json`

For gates:

```yaml
- uses: EffortlessMetrics/tokmd@v1
  with:
    version: '1.10.0'
    mode: gate
    paths: .
    artifact: 'true'
    comment: 'false'
```

For all Action modes, inputs, outputs, artifacts, checkout guidance, and release-asset behavior, see [GitHub Action reference](docs/github-action.md).

## Why It Exists

Raw LOC output is easy to generate and hard to reuse.

CI needs artifacts and gates. Review workflows need stable before/after surfaces. LLM workflows need bounded context instead of pasted terminal output.

`tokmd` makes repository shape repeatable, diffable, and machine-readable.

## What It Looks Like

Representative summary output:

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

`tokmd-wasm` and `web/runner` expose a narrower browser-safe slice:

- `lang`
- `module`
- `export`
- browser-safe `analyze` presets on ordered in-memory inputs

Native filesystem flows, Git-history enrichers, `gate`, `cockpit`, `sensor`, `baseline`, `context`, and `handoff` remain native-first.

Browser cache semantics, progress events, retry/rate-limit UX, and authenticated fetch are planned follow-up work.

The machine-readable capability contract lives in [`docs/capabilities/wasm.json`](docs/capabilities/wasm.json).

## What tokmd Is Not

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

- [GitHub Action reference](docs/github-action.md) for Action inputs, outputs, modes, and release assets
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
