# tokmd-config

Define tokmd CLI arguments and config files in one schema.

## Problem

The CLI, config file, and tool-schema generation need to agree on the same option surface.

## What it gives you

- `Cli`, `Commands`, and `GlobalArgs`
- command-specific args such as `CliLangArgs`, `CliAnalyzeArgs`, `CliGateArgs`, `CockpitArgs`, `HandoffArgs`, and `SensorArgs`
- config models: `UserConfig` and `Profile`
- re-exports from `tokmd-types` plus `ToolSchemaFormat`

## Quick use / integration notes

`Cli::parse()` reads the clap surface. `UserConfig` and `Profile` model `tokmd.toml`.

```toml
[profiles.ci]
top = 10
format = "json"
```

## Go deeper

### Tutorial

- `../../docs/tutorial.md`

### How-to

- `../../docs/reference-cli.md`

### Reference

- `src/lib.rs`

### Explanation

- `../../docs/explanation.md`
