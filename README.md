# tokmd

> **Current Status**: v1.0.0 (Release Candidate). See [ROADMAP.md](ROADMAP.md) for details.

**tokmd turns *tokei’s* scan into a *receipt*: a compact, deterministic artifact humans can paste into PRs/chats and pipelines/LLMs can parse without shell glue.**

It is a Tokei-backed, cross-platform tool that produces one-command outputs:
- **Markdown/TSV** for humans (paste into PRs, issues, or ChatGPT).
- **JSONL/CSV** for pipelines and LLMs.

## The Mental Model

- **tokei counts.**
- **tokmd packages counts into receipts.**
- **Pipelines/LLMs consume receipts.**
- **Humans review the summary, not the whole repo.**

## What It Is (And Isn't)

**It IS:**
- A **Sensor**: Emits receipts for languages, modules, and file inventories.
- **Schema-Bound**: Outputs are strict contracts (`schema_version`) that pipelines can trust.
- **Safe**: Offers redaction to support "If you wouldn't email it, don't paste."

**It is NOT:**
- **A Productivity Metric**: LOC is for shape, not grading people.
- **A Quality Judge**: It doesn't lint or test.
- **A TUI**: It generates receipts, it doesn't offer interactive exploration.

## Why Use It? (The "What We Add" List)

1.  **One command, no glue**: Replaces `tokei | jq | column` with a single binary.
2.  **Receipts**: Stable, versioned JSON envelopes for tooling.
3.  **Module View**: Groups code by directory (`crates/`, `packages/`) for monorepo reality checks.
4.  **Export Dataset**: Streaming JSONL/CSV for LLMs and automation.
5.  **Safety**: Redaction (`--redact paths`) allows sharing repo shape without leaking internal names.

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

The documentation is structured into four parts:

- **[Tutorials](docs/tutorial.md)**: Learning-oriented lessons. Start here!
- **[How-To Guides](docs/recipes.md)**: Problem-oriented recipes (LLM contexts, auditing vendors, CI tracking).
- **[Reference](docs/reference-cli.md)**: Technical descriptions of CLI commands and [Schemas](docs/SCHEMA.md).
- **[Explanation](docs/explanation.md)**: Understanding the "Receipt" philosophy.

## License

Dual MIT or Apache-2.0.
