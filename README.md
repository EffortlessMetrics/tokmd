# tokmd

> **Current Status**: v1.0.0 (Release Candidate). See [ROADMAP.md](ROADMAP.md) for details.

`tokmd` is a tiny, cross-platform wrapper around the **tokei** library that produces **repo inventory receipts**:

- **Markdown/TSV** summaries for humans (paste into ChatGPT / issues / PRs)
- **JSON / JSONL / CSV** datasets for pipelines and tooling

It’s designed to be **one command**: run once, copy/paste once. No `jq`, no `column`, no PowerShell quoting gymnastics.

## Why use this?

This is **not** a productivity metric tool. LOC/PR velocity are easy to game and have lost meaning in AI-native repos. 

Use `tokmd` to understand repo *shape*, not to grade people. It helps you answer:
- "What does this repo look like to an LLM?"
- "Which modules are growing the fastest?"
- "How much of our code is actually vendored libraries?"

## Installation

From crates.io:
```bash
cargo install tokmd
```

## Quick Start (Tutorial)

### 1. Get a Language Summary
Run `tokmd` in any repo to get a Markdown table of languages.

```bash
tokmd
# Output:
# |Lang|Code|Lines|
# |---|---:|---:|
# |Rust|1200|1500|
# ...
```

### 2. Get a Module Summary
See where the code actually lives.

```bash
tokmd module
# Output:
# |Module|Code|Lines|
# |---|---:|---:|
# |crates/cli|500|600|
# |crates/core|700|900|
```

### 3. Generate a Data Receipt
Export a machine-readable list of files for analysis or LLM contexts.

```bash
tokmd export > repo_inventory.jsonl
```

## How-To Guides

- **[Feed a Codebase to an LLM](docs/recipes.md#1-feeding-a-codebase-to-an-llm)**
- **[Track Repo Growth in CI](docs/recipes.md#2-tracking-repo-growth-over-time)**
- **[Audit Vendor Dependencies](docs/recipes.md#3-auditing-vendor-dependencies)**
- **[Find "Heavy" Files](docs/recipes.md#4-finding-heavy-files)**

## Reference

### CLI Commands
- `tokmd` (alias `tokmd lang`): Language summary.
- `tokmd module`: Module-level summary.
- `tokmd export`: File-level inventory (JSONL/CSV).
- `tokmd init`: Generate `.tokeignore`.

### Output Schemas
- **[Receipt Schema](docs/SCHEMA.md)**: Human-readable guide to the JSON output.
- **[Formal JSON Schema](docs/schema.json)**: Machine-readable spec.

## Notes & Explanation

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
