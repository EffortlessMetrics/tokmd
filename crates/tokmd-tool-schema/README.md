# tokmd-tool-schema

Generate AI tool schemas from clap command trees.

## Problem

AI agents need stable tool definitions from the same CLI tree humans use.

## What it gives you

- `build_tool_schema(&Command) -> ToolSchemaOutput`
- `render_output(..., ToolSchemaFormat, pretty) -> Result<String>`
- `ToolSchemaFormat::{Openai, Anthropic, Jsonschema, Clap}`

## Quick use / integration notes

`build_tool_schema` walks the root command and subcommands, skips generated `help` and `version` args, and captures defaults plus enum values.

`render_output` can serialize the same schema as `openai`, `anthropic`, `jsonschema`, or raw `clap`.

## Go deeper

### How-to

- `../../docs/reference-cli.md`

### Reference

- `src/lib.rs`

### Explanation

- `../../docs/explanation.md`
