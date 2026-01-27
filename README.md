# tokmd

> **tokmd turns tokei scans into deterministic receipts (Markdown/TSV/JSONL/CSV) for PRs, CI, and LLM workflows.**

![License](https://img.shields.io/crates/l/tokmd.svg)
![Version](https://img.shields.io/crates/v/tokmd.svg)
![CI](https://github.com/EffortlessMetrics/tokmd/workflows/CI/badge.svg)

## The Pain

Repo stats are easy to compute (cloc, tokei) but annoying to **use**.
* Shell scripts to pipe `tokei` output are fragile.
* Different OSs (Windows/Linux) produce slightly different outputs.
* Pasting raw line counts into LLMs (ChatGPT/Claude) is messy and unstructured.
* Checking "did this PR bloat the codebase?" requires deterministic diffs.

## The Product

`tokmd` is a stable wrapper around the excellent [`tokei`](https://github.com/XAMPPRocky/tokei) library. It produces **receipts**: schema'd, normalized artifacts that represent the "shape" of your code.

* **Humans**: Markdown summaries, TSV tables.
* **Machines**: JSON/JSONL/CSV datasets.
* **LLMs**: "Map" your repo structure before asking questions.

## Workflow

1. **Map**: Generate a receipt of your repo (files, sizes, languages).
2. **Select**: Feed this map to an LLM to decide what context is relevant.
3. **Pack**: Use tools like `repomix` or `files-to-prompt` to pack the selected files.

## Quickstart

```bash
# Install (recommended)
nix profile install github:EffortlessMetrics/tokmd

# Or install with Cargo
cargo install tokmd

# 1. Markdown summary (great for PR descriptions)
tokmd > summary.md

# 2. Module breakdown (great for monorepos)
tokmd module --module-roots crates,packages

# 3. File inventory (great for LLM context planning)
tokmd export --format jsonl > inventory.jsonl
```

## Use Cases

### 1. PR Summaries
Add a `tokmd` summary to your Pull Request to show reviewers exactly what changed in the repo structure.

### 2. CI Artifacts
Save `inventory.jsonl` in your CI pipeline. Diff it against the `main` branch to detect massive deletions or unexpected file additions.

### 3. LLM Context Map
Before dumping 50 files into Claude, run `tokmd export`. Paste the list. Ask: "Based on this file list and token counts, which files are relevant to feature X?"

## Safety
`tokmd` supports **redaction** for sharing receipts publicly or with untrusted models.

```bash
# Hashed paths, but keeps structure/sizes
tokmd export --redact all
```

*Note: Redaction hashes filenames but preserves structure. It is not true anonymity against determined analysis.*

## Schema Contract
`tokmd` guarantees output stability for automation.
* All JSON outputs include `schema_version`.
* Changes to the schema increment this version.
* Fields are additive within a version.

## Commands

| Command | Purpose |
| :--- | :--- |
| `tokmd` (default) | Language summary (lines, files, bytes). |
| `tokmd module` | Group stats by top-level folders (`crates/`, `src/`). |
| `tokmd export` | File-level dataset for downstream tools. |
| `tokmd run` | Execute a full scan and save artifacts to a run directory. |
| `tokmd diff` | Compare two runs or receipts. |
| `tokmd init` | Generate a `.tokeignore` file. |

## Why not just tokei?

| Feature | `tokei` | `tokmd` |
| :--- | :--- | :--- |
| **Core Value** | Fast, accurate counting | Usage & Workflow |
| **Output** | Human terminal tables | Structured Receipts (Md/JSONL/TSV) |
| **Integrations** | CLI flags | GitHub Action, Config Profiles |
| **Stability** | Output varies by version | Strict Schema Versioning |
| **LLM Ready** | No | Yes (Bytes, Token Estimates) |

## Installation

### Nix (recommended)
```bash
# Run without installing
nix run github:EffortlessMetrics/tokmd -- --version

# Install to your profile
nix profile install github:EffortlessMetrics/tokmd

# Build from source
nix build
```

### From crates.io
```bash
cargo install tokmd
```

### GitHub Action
```yaml
- uses: EffortlessMetrics/tokmd@v1
  with:
    paths: '.'
```

## License
MIT or Apache-2.0.
