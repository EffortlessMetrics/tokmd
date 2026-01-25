# tokmd

> **Current Status**: v0.2.0 (Pre-v1.0). See [ROADMAP.md](ROADMAP.md) for the path to v1.0 and [TODO.md](TODO.md) for current tasks.

`tokmd` is a tiny, cross-platform wrapper around the **tokei** library that produces
repo **inventory receipts**:

- **Markdown/TSV** summaries for humans (paste into ChatGPT / issues / PRs)
- **JSON / JSONL / CSV** datasets for pipelines and tooling

It’s designed to be **one command**: run once, copy/paste once. No `jq`, no `column`, no
PowerShell quoting gymnastics.

This is **not** a productivity metric tool. LOC/PR velocity are easy to game and have
lost meaning in AI-native repos. Use `tokmd` to understand repo *shape*, not to grade
people.

## Install

From crates.io:

```bash
cargo install tokmd
```

Optional: also install a `tok` alias (via an opt-in feature):

```bash
cargo install tokmd --features alias-tok
```

From a local checkout:

```bash
cargo install --path .
```

## Quick start

Language summary (Markdown by default):

```bash
tokmd
tokmd --top 12
tokmd --files
tokmd --format tsv
tokmd --format json

# If your repo has embedded languages (e.g., code blocks), you can break them out:
tokmd --children separate
```

Module summary (good for modular repos):

```bash
tokmd module
tokmd module --top 15
tokmd module --module-roots crates,packages --module-depth 2

# Include embedded languages (default):
tokmd module --children separate
# Ignore embedded languages:
tokmd module --children parents-only
```

Export a file-level dataset (good for analysis or feeding into other tools):

```bash
# JSONL is the default (one record per line).
tokmd export > tokmd.jsonl

tokmd export --format csv  --out tokmd.csv
tokmd export --format json --out tokmd.json

# Include a meta record (default for JSON/JSONL) describing how the scan was run:
tokmd export --meta true
# Or emit rows only:
tokmd export --meta false

# Safer copy/paste into LLMs:
tokmd export --redact paths
tokmd export --redact all

# Drop tiny rows / cap output size:
tokmd export --min-code 10 --max-rows 500
```

Initialize a starter `.tokeignore`:

```bash
tokmd init
# Preview the template:
tokmd init --print
# Overwrite if one already exists:
tokmd init --force

# Pick a profile:
tokmd init --profile rust
tokmd init --profile node
tokmd init --profile mono
tokmd init --profile python
tokmd init --profile go
tokmd init --profile cpp
```

## Notes

- `tokmd` links to the `tokei` crate (it does **not** shell out to the `tokei` binary).
- Ignore behavior is controlled by `tokei` and respects `.gitignore`, `.ignore`,
  and `.tokeignore` by default.
- Config files: `tokmd` reads `tokei.toml` / `.tokeirc` by default. Use `--config none`
  to ignore config files.
- Use `--exclude PATTERN` (alias `--ignore`) to add extra ignore patterns.
- Windows: in PowerShell, `\` is a path separator, not a line-continuation character.
  If you copy multi-line examples, use backtick (`) for line continuation—or just keep
  it single-line (`tokmd` is designed for that).

## Documentation

- **[Recipe Book](docs/recipes.md)**: Real-world examples (LLM contexts, auditing vendors, CI tracking).
- **[Receipt Schema](docs/SCHEMA.md)**: Detailed specification of the JSON output format.
- **[Formal JSON Schema](docs/schema.json)**: Machine-readable schema definition.
- **[Roadmap](ROADMAP.md)**: Path to v1.0 and future plans.

## License

Dual MIT or Apache-2.0.
