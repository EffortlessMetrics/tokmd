# tokmd

> Deterministic repo receipts, analysis, and review artifacts for humans, CI, and LLM workflows.

[![Crates.io](https://img.shields.io/crates/v/tokmd)](https://crates.io/crates/tokmd)
[![GitHub Release](https://img.shields.io/github/v/release/EffortlessMetrics/tokmd?display_name=tag)](https://github.com/EffortlessMetrics/tokmd/releases)
[![Docs.rs](https://img.shields.io/docsrs/tokmd)](https://docs.rs/tokmd)
[![CI](https://github.com/EffortlessMetrics/tokmd/actions/workflows/ci.yml/badge.svg)](https://github.com/EffortlessMetrics/tokmd/actions/workflows/ci.yml)
[![License](https://img.shields.io/crates/l/tokmd)](https://crates.io/crates/tokmd)
[![Downloads](https://img.shields.io/crates/d/tokmd)](https://crates.io/crates/tokmd)

`tokmd` turns a source tree into stable receipts you can diff, analyze, archive, gate, and pack for downstream automation. It starts with code inventory, then keeps going: saved artifacts, derived metrics, PR review surfaces, policy checks, sensor outputs, LLM-ready context bundles, and a browser-safe in-memory analysis slice.

## GitHub Action

Use the root composite action when you want a workflow-friendly receipt and PR summary without scripting `tokmd` installation yourself.

- Installs a released `tokmd` binary for the current runner.
- By default, generates `tokmd-summary.md` from `tokmd module` and a structured receipt file from `tokmd export`.
- Can run a single explicit mode: `module`, `export`, `gate`, `cockpit`, `sensor`, or `baseline`.
- Optionally uploads generated files as workflow artifacts.
- Optionally posts the summary as a pull request comment.

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
      - uses: actions/checkout@v4

      - uses: EffortlessMetrics/tokmd@v1
        with:
          version: '1.10.0'
          paths: .
          module-roots: crates,packages
          top: '20'
          format: json
          artifact: 'true'
          comment: 'true'
```

For release-candidate smoke tests, pin both the Action ref and the downloaded binary:

```yaml
- uses: EffortlessMetrics/tokmd@v1.10.0-rc.1
  with:
    version: '1.10.0-rc.1'
    paths: .
    artifact: 'true'
    comment: 'false'
```

For stable workflows, use the stable Action ref and an explicit binary version:

```yaml
- uses: EffortlessMetrics/tokmd@v1
  with:
    version: '1.10.0'
    paths: .
    artifact: 'true'
    comment: 'true'
```

Inputs:

| Input | Required | Default | Purpose |
| :---- | :------- | :------ | :------ |
| `mode` | no | `(omitted)` | `tokmd` mode to run: `module`, `export`, `gate`, `cockpit`, `sensor`, or `baseline`. Omit it for the existing module + export flow. |
| `version` | no | `latest` | `tokmd` release to install. Pass an explicit version if you want the action ref and binary version to stay aligned. |
| `paths` | no | `.` | Paths to scan. Space/newline-delimited list; each entry is passed as a separate argument. |
| `module-roots` | no | `crates,packages` | Module root prefixes for `tokmd module` and `tokmd export`. |
| `top` | no | `20` | Number of rows shown in `tokmd-summary.md`. |
| `format` | no | `json` | Receipt export format: `json`, `jsonl`, or `csv`. |
| `base` | no | `(inferred)` | Base git ref for `mode: cockpit` and `mode: sensor`. Explicit values are used as provided. When omitted, pull request runs use `origin/$GITHUB_BASE_REF`; other runs use `origin/HEAD` when available. |
| `head` | no | `HEAD` | Head git ref for `mode: cockpit` and `mode: sensor`. |
| `artifact` | no | `true` | Upload generated tokmd files as workflow artifacts. |
| `comment` | no | `true` | Post the generated Markdown summary as a pull request comment when running on `pull_request` events. |

Outputs:

| Output | Description |
| :----- | :---------- |
| `receipt` | Path to the generated receipt file. |
| `summary` | Path to `tokmd-summary.md` or a mode-specific Markdown summary when one is generated. |
| `gate-verdict` | Path to `tokmd-gate-verdict.json` when `mode: gate` is used. |
| `cockpit-report` | Path to `tokmd-cockpit-report.json` when `mode: cockpit` is used. |
| `sensor-report` | Path to `tokmd-sensor-report.json` when `mode: sensor` is used. |
| `baseline-report` | Path to `tokmd-baseline.json` when `mode: baseline` is used. |

Notes:

- PR commenting needs `pull-requests: write` and only runs for `pull_request` events.
- `mode: gate` runs `tokmd gate --format json` and expects policy or ratchet rules from `tokmd.toml` in the checkout. A failing gate still writes `tokmd-gate-verdict.json` before the action fails.
- `mode: gate` accepts exactly one path; same-line or multiline multi-path inputs fail before `tokmd gate` runs.
- `mode: cockpit` runs `tokmd cockpit --format json` and writes `tokmd-cockpit-report.json`. If `base` is omitted, the action infers a repository-aware base from `origin/$GITHUB_BASE_REF` on pull requests or `origin/HEAD` on other events; if no base can be resolved, set `base` explicitly.
- `mode: sensor` runs `tokmd sensor --format json` and writes `tokmd-sensor-report.json`, `comment.md`, and the `extras/` sidecar directory. The `summary` output points to `comment.md`. It uses the same inferred `base` behavior as cockpit mode.
- `mode: baseline` runs `tokmd baseline --force` and writes `tokmd-baseline.json`. It accepts exactly one path; same-line or multiline multi-path inputs fail before `tokmd baseline` runs.
- For cockpit and sensor in external PR workflows, prefer `actions/checkout` with `fetch-depth: 0` so compare refs are available; otherwise set `base` explicitly.
- The action currently installs the latest `tokmd` release by default. If you publish the action under `@v1` and want a specific binary version, set `with: version: '1.10.0'` explicitly.
- Release asset support is Linux/macOS `amd64` and `arm64`, plus Windows `amd64`.
- To scan multiple paths, pass whitespace-separated values (for example, `paths: "src crates"`), or use a multiline input:

  ```yaml
  paths: |
    src
    packages
  ```

Gate mode:

```yaml
- uses: EffortlessMetrics/tokmd@v1
  with:
    mode: gate
    paths: .
    artifact: 'true'
    comment: 'false'
```

Cockpit mode:

```yaml
- uses: EffortlessMetrics/tokmd@v1
  with:
    mode: cockpit
    base: origin/main
    head: HEAD
    artifact: 'true'
    comment: 'false'
```

Sensor mode:

```yaml
- uses: EffortlessMetrics/tokmd@v1
  with:
    mode: sensor
    base: origin/main
    head: HEAD
    artifact: 'true'
    comment: 'false'
```

Baseline mode:

```yaml
- uses: EffortlessMetrics/tokmd@v1
  with:
    mode: baseline
    paths: .
    artifact: 'true'
    comment: 'false'
```

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
- Public repo ingestion uses GitHub tree and contents APIs with deterministic in-memory input materialization.
- Explicit cache behavior, progress events, retry/rate-limit UX, and authenticated fetch polish are follow-up browser-runtime work.
- Git-history enrichers and full native filesystem flows remain native-first.
- The machine-readable browser capability contract lives in `docs/capabilities/wasm.json` and records current runner support, not future browser candidates.

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
