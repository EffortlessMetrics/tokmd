## 💡 Summary
Updated the documentation to align with shipped reality regarding MCP integration. The plan originally called for a dedicated `tokmd serve` command, but the functionality was actually delivered via the `tokmd tools` command which generates MCP-compatible tool definitions.

## 🎯 Why
The roadmap and implementation plan referenced a non-existent `tokmd serve` command. This drift creates confusion for users looking for the MCP server and contributors attempting to build upon it. Updating the docs ensures future work is based on the actual shipped surface.

## 🔎 Evidence
- `tokmd --help` and `tokmd tools --help` reveal the `tools` command exists for generating agent schemas, while `serve` does not exist.
- Found references to `tokmd serve` in `ROADMAP.md`, `docs/PRODUCT.md`, and `docs/implementation-plan.md`.

## 🧭 Options considered
### Option A (recommended)
- Update the documentation to reflect that MCP integration is achieved via tool schema generation (`tokmd tools`) instead of a dedicated server.
- Fits the `tooling-governance` shard by keeping design docs aligned with shipped reality.
- Trade-offs: Structure (improves accuracy), Governance (prevents reliance on non-existent commands).

### Option B
- Create a Learning PR without fixing the documents.
- When to choose: If the changes were outside the allowed paths or overly complex.
- Trade-offs: Leaves documentation misleading and doesn't resolve the factual drift.

## ✅ Decision
Option A was chosen. The factual drift is clear, and fixing the documentation aligns it with the shipped reality of `tokmd tools` and prevents confusion.

## 🧱 Changes made (SRP)
- `ROADMAP.md`: Updated AI Agent Integration section to focus on tool definitions instead of a server.
- `docs/PRODUCT.md`: Clarified MCP Server in Future Direction is via tool schema generation.
- `docs/implementation-plan.md`: Updated Phase 6 implementation to reflect `tokmd tools` command and definitions.

## 🧪 Verification receipts
```text
cargo xtask docs --check
cargo fmt -- --check
cargo check
cargo test -p xtask
```

## 🧭 Telemetry
- Change shape: Documentation update
- Blast radius: docs
- Risk class: Low (Documentation only)
- Rollback: `git checkout ROADMAP.md docs/PRODUCT.md docs/implementation-plan.md`
- Gates run: `cargo xtask docs --check`, `cargo fmt -- --check`, `cargo check`, `cargo test -p xtask`

## 🗂️ .jules artifacts
- `.jules/runs/cartographer_roadmap_design/envelope.json`
- `.jules/runs/cartographer_roadmap_design/decision.md`
- `.jules/runs/cartographer_roadmap_design/receipts.jsonl`
- `.jules/runs/cartographer_roadmap_design/result.json`
- `.jules/runs/cartographer_roadmap_design/pr_body.md`

## 🔜 Follow-ups
None.
