# tokmd

CLI product surface for deterministic repo receipts and analysis.

## Problem

You need the full `tokmd` workflow from the terminal: summaries, saved receipts, analysis presets, diffs, policy gates, and LLM-oriented packing, without composing lower-tier crates by hand.

## What it gives you

- The `tokmd` CLI binary
- Commands such as `lang`, `module`, `export`, `run`, `analyze`, `badge`, `diff`, `context`, `handoff`, `cockpit`, `gate`, `baseline`, `sensor`, and `tools`
- Default feature set for git, walk, content, UI, novelty, topics, and archetype flows
- The `tok` alias binary when `alias-tok` is enabled

## Quick use / integration notes

Install it:

```bash
cargo install tokmd --locked
```

Run the common paths:

```bash
tokmd --format md --top 8
tokmd run --analysis receipt --output-dir .runs/current
tokmd analyze --preset risk --format md
tokmd diff main HEAD
tokmd context --budget 128k --mode bundle --output context.txt
```

Feature flags:

```toml
[dependencies]
tokmd = { workspace = true, features = ["git", "content"] }
```

Use `tokmd-core` instead when you need the same workflows embedded in Rust or FFI without the CLI layer.

## Go deeper

### Tutorial

- [Root README](../../README.md)
- [Tutorial](../../docs/tutorial.md)

### How-to

- [Recipes](../../docs/recipes.md)
- [Troubleshooting](../../docs/troubleshooting.md)

### Reference

- [CLI Reference](../../docs/reference-cli.md)
- [Schema](../../docs/SCHEMA.md)

### Explanation

- [Architecture](../../docs/architecture.md)
- [Design](../../docs/design.md)
