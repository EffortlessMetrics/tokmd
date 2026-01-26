# tokmd

> **Current Status**: v1.0.0 (Release Candidate). See [ROADMAP.md](ROADMAP.md) for details.

**tokmd turns *tokei’s* scan into a *receipt*: a compact, deterministic artifact humans can paste into PRs/chats and pipelines/LLMs can parse without shell glue.**

It is a Tokei-backed, cross-platform tool that produces one-command outputs:
- **Markdown/TSV** for humans (paste into PRs, issues, or ChatGPT).
- **JSONL/CSV** for pipelines and LLMs.

## Mental Model

- **tokei counts.**
- **tokmd packages counts into receipts.**
- **Pipelines/LLMs consume receipts.**

## Where tokmd fits

Think of this space as three layers:

- **Counting engines**: `tokei`, `cloc`, `scc`  
  They count. Their output is mostly terminal-shaped.

- **Receipt / packaging**: `tokmd`  
  Turns counts into stable artifacts (maps) for humans and machines.

- **Content packers for LLMs**: `repomix`, `files-to-prompt`  
  They bundle file *contents* for prompts.

**tokmd is the map-maker.** Run it first to see the territory. Use a packer only after you know what you want to read.

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

## Why not just use `tokei`?

| Problem | Scripts (`tokei \| jq`) | tokmd |
|---|---|---|
| **Cross-platform** | breaks on shells/tools | one binary |
| **Output contract** | ad-hoc | `schema_version` + schema |
| **Safety** | hard to do right | built-in redaction |
| **Stability** | no tests | golden tests + CI |

## Stability Contract

- JSON/JSONL outputs are **schema-versioned** (`schema_version`).
- Sorting and path normalization are deterministic.
- Breaking output changes bump `schema_version` (and semver major when applicable).
- tokei’s counting semantics are upstream. tokmd’s guarantee is packaging + determinism.

## Safety Note (Redaction)

`--redact` hashes identifiers to reduce accidental leakage in copy/paste workflows.

It does **not** make data anonymous. You still leak shape (counts, file extensions, relative sizes). **If you wouldn’t email it, don’t paste it.**

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
