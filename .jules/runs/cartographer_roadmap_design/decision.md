# Decision

## Investigated
I investigated the alignment between the documentation (ROADMAP, docs/PRODUCT.md, docs/implementation-plan.md, and docs/requirements.md) and the actual shipped CLI regarding the "MCP Server Mode" and "tokmd serve" command.

The documentation makes references to `tokmd serve` as part of "Phase 6: MCP Server Mode (v2.0)". However, I found that the `tokmd tools` command exists in the CLI to generate OpenAI, Anthropic, and JSON Schema definitions for AI agents, while `tokmd serve` has not been implemented.

There is a factual drift between what the roadmap/design/implementation plans state about the approach to MCP integration (using a dedicated server `tokmd serve` vs generating tool schemas with `tokmd tools`).

## Option A (Recommended)
Update the roadmap, product, and implementation plan documents to reflect the shipped reality: MCP and AI Agent integration is primarily achieved via the `tokmd tools` command, which generates tool definitions, rather than a dedicated `tokmd serve` MCP server.

- **Why it fits**: It resolves a factual drift between shipped reality and the roadmap/design docs, which is the primary mission of the Cartographer persona in the `tooling-governance` shard.
- **Trade-offs**: Structure (improves documentation accuracy), Velocity (takes some effort to find and replace references), Governance (prevents future contributors from looking for or relying on a non-existent `tokmd serve` command).

## Option B
Create a learning PR documenting this documentation gap without fixing it.

- **When to choose**: If the documentation drift was minor or if fixing it required extensive changes outside the allowed paths.
- **Trade-offs**: Misses an opportunity to improve the repo's truth sources and leaves the documentation in a misleading state.

## Decision
Selected **Option A**. The drift is clear, actionable, and within the allowed paths of the `tooling-governance` shard. Fixing the documentation aligns it with the shipped reality of `tokmd tools` and prevents confusion around a non-existent `tokmd serve` command.
