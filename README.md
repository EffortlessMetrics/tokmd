# tokmd

> **Current Status**: v1.0.0 (Release Candidate). See [ROADMAP.md](ROADMAP.md) for details.

**tokmd turns *tokei’s* scan into a *receipt*: a compact, deterministic artifact humans can paste into PRs/chats and pipelines/LLMs can parse without shell glue.**

It is a Tokei-backed, cross-platform tool that produces one-command outputs:
- **Markdown/TSV** for humans (paste into PRs, issues, or ChatGPT).
- **JSONL/CSV** for pipelines and LLMs.

## What it does

tokmd runs a single tokei scan, then emits three “views” with a stable envelope:

* **lang**: language totals (good for “what’s in here?”)
* **module**: directory rollups (good for “where is the mass?”)
* **export**: file-level rows (good for “give me a dataset I can route/pivot/diff”)

Each view is designed to be:

* **copy/paste friendly** (Markdown/TSV)
* **machine-friendly** (JSON / JSONL / CSV)
* **deterministic** (stable ordering, normalized paths)
* **receipt-shaped** (schema_version + tool info + args + totals + rows)

## Why not just use `tokei`?

| Problem | Scripts (`tokei \| jq`) | tokmd |
|---|---|---|
| **Cross-platform** | breaks on shells/tools | one binary |
| **Output contract** | ad-hoc | `schema_version` + schema |
| **Safety** | hard to do right | built-in redaction |
| **Stability** | no tests | golden tests + CI |

## Use Cases

### For Humans (PRs & Docs)
*   “Give me a repo summary I can paste into a PR/ticket/chat in 10 seconds.”
*   “Tell me where the mass is in a modular tree.”
*   “Give me top-N plus totals, without 40 lines of long tail.”

### For LLM Workflows (AI-Native)
*   “I need a map before I dump code.”
*   “I need to cap context blast radius.”
*   “I need a safe-to-share mode when names matter.”

### For Pipelines (CI/CD)
*   “Give me a stable payload I can diff, store, validate.”
*   “Stop making me re-derive totals and re-normalize paths.”
*   “Let me build gates on receipts, not on parsing terminal output.”

## Workflow: map → select → pack (LLMs)

1. **Map the repo** (cheap):
   ```bash
   tokmd module --top 20
   tokmd export --min-code 20 --max-rows 300 --redact paths > map.jsonl
   ```

2. **Select paths**:
   Human or agent picks a small set of interesting files from the map.

3. **Pack contents** (expensive):
   Use a content packer (repomix, files-to-prompt) on the selected paths.

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
- `tokmd run`: Run a full scan and save artifacts to `.runs/`.
- `tokmd diff`: Compare two receipts or runs.
- `tokmd completions`: Generate shell completion scripts.

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

## Architecture

This project uses a "Microcrates" architecture to ensure clean boundaries and reusable components.

*   **`tokmd-core`**: The **recommended library entry point**. Use this if embedding tokmd.
*   **`tokmd-types`**: The **stable data contract**. Pure data structures, no logic.
*   **`tokmd-model`**: Aggregation logic.
*   **`tokmd-scan`**: Tokei adapter.
*   **`tokmd-format`**: Rendering logic.
*   **`tokmd-config`**: Configuration schemas.
*   **`tokmd`**: The CLI binary.

See [RELEASE.md](RELEASE.md) for the publishing order and strategy.

## License

Dual MIT or Apache-2.0.
