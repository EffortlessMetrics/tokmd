# tokmd-tool-schema

Single-responsibility microcrate for generating AI tool schemas from a clap command tree.

## What it does

- Builds a deterministic command/argument schema model from `clap::Command`.
- Renders the model as OpenAI, Anthropic, JSON Schema, or raw clap JSON.
- Provides a shared `ToolSchemaFormat` enum for CLI argument wiring.

## API

- `build_tool_schema(cmd: &clap::Command) -> ToolSchemaOutput`
- `render_output(schema: &ToolSchemaOutput, format: ToolSchemaFormat, pretty: bool) -> anyhow::Result<String>`
