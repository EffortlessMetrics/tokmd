# Cartographer Decision: ROADMAP.md vs Shipped Reality Alignment

## Option A: Align ROADMAP.md with shipped `tools` truth
- **What it is**: The `tokmd tools` subcommand already outputs CLI schema as JSON for AI agents (OpenAI, Anthropic, JSONSchema). The `ROADMAP.md` lists MCP server (v2.0) as planned, but doesn't explicitly document the AI tool schema generation that *already shipped*. I will document this reality in the `tools` command coverage in `ROADMAP.md` or `docs/reference-cli.md`.
- **Why it fits this repo and shard**: Aligning roadmap/docs with shipped truth.
- **Trade-offs**: Corrects factual drift without adding new features.

## Option B: Clean up ROADMAP.md completed milestones
- **What it is**: Move v1.0.0 - v1.7.x into an archive or summarize them, keeping only the recent v1.8.0 and v1.9.0.
- **Why it fits**: Standard docs cleanup.
- **Trade-offs**: Doesn't strictly address "roadmap drift from shipped reality" as strongly as Option A.

## Decision: Option A
I will update `ROADMAP.md` (and possibly `ARCHITECTURE.md` or `docs/reference-cli.md`) to explicitly mention the `tools` command and its role in AI agent integration, as this is a "real architectural/design choice" and "shipped reality" that seems missing from the roadmap's AI integration story. Let's look closer at `ROADMAP.md` to see where `tools` should be documented.
