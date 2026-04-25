## 💡 Summary
Updated the `ROADMAP.md` document to reflect the shipped reality of AI tool schema integration. The `tokmd tools` subcommand (which outputs OpenAI, Anthropic, and JSON Schema formats) is now correctly documented under the "AI Agent Integration & MCP Server Mode" section, replacing the misleading implication that all AI tool integration was purely future work.

## 🎯 Why
The ROADMAP.md document exhibited factual drift. It listed "MCP server ... tools" as entirely planned for v2.0, completely ignoring that the project already shipped robust LLM tool definition generation (`tokmd tools`) in v1.3.0. This gap could mislead users or agents looking for AI integration points into thinking none existed yet.

## 🔎 Evidence
- **Finding**: `ROADMAP.md` section "v2.0 — Platform Evolution -> B. MCP Server Mode" mentioned tools as purely planned.
- **Reality**: `tokmd tools --help` outputs: "Output CLI schema as JSON for AI agents" with options for `openai`, `anthropic`, and `jsonschema`.
- **Receipts**: `cargo run -- tools --format openai` succeeds and generates the schema.

## 🧭 Options considered
### Option A (recommended)
- what it is: Update `ROADMAP.md` to explicitly document the already-shipped `tokmd tools` command in the AI integration section.
- why it fits this repo and shard: Directly aligns the roadmap's design intent with the shipped reality, falling perfectly under the Cartographer persona's mandate.
- trade-offs: Structure / Velocity / Governance: Low risk, high value. It corrects the factual drift without requiring code changes or creating new architectural surfaces.

### Option B
- what it is: Mass-archive older completed milestones in the roadmap.
- when to choose it instead: If the entire roadmap structure was the primary friction point rather than factual drift on specific features.
- trade-offs: Doesn't address the specific gap in explaining the existing AI integration surface.

## ✅ Decision
Chose Option A to correct the factual drift regarding AI agent integration in the `ROADMAP.md`.

## 🧱 Changes made (SRP)
- `ROADMAP.md`: Updated "B. MCP Server Mode" to "B. AI Agent Integration & MCP Server Mode" and added a bullet point noting that `tokmd tools` (OpenAI, Anthropic, JSON Schema) is already shipped.

## 🧪 Verification receipts
```text
$ cargo xtask docs --check
Documentation is up to date.

$ cargo fmt -- --check

$ cargo clippy -- -D warnings
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 25.42s
```

## 🧭 Telemetry
- Change shape: Docs update
- Blast radius: `ROADMAP.md` only (docs)
- Risk class: Safe / Documentation only
- Rollback: `git checkout HEAD -- ROADMAP.md`
- Gates run: `cargo xtask docs --check`, `cargo fmt -- --check`, `cargo clippy -- -D warnings`

## 🗂️ .jules artifacts
- `.jules/runs/cartographer_roadmap_design/envelope.json`
- `.jules/runs/cartographer_roadmap_design/decision.md`
- `.jules/runs/cartographer_roadmap_design/receipts.jsonl`
- `.jules/runs/cartographer_roadmap_design/result.json`
- `.jules/runs/cartographer_roadmap_design/pr_body.md`

## 🔜 Follow-ups
None.
