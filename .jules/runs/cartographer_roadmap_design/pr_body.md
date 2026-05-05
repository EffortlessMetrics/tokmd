## 💡 Summary
This is a learning PR. Attempted to resolve documentation drift regarding the MCP Server Mode. The initial patch recast `tokmd serve` to the existing `tokmd tools` command, but this was rejected (superseded by #1588) because the `tokmd serve` functionality remains planned future work.

## 🎯 Why
The documentation currently lists `tokmd serve` as part of Phase 6, leading to confusion since it isn't shipped yet, while `tokmd tools` is shipped. Trying to map the planned server feature entirely to the shipped tools feature was incorrect. This learning PR captures that friction.

## 🔎 Evidence
- PR rejection comment: "Closed in favor of #1588. tokmd tools is shipped tool-schema generation, but tokmd serve / MCP server resources remain future work, so this PR's MCP-server recast should not land as written."
- `ROADMAP.md` and `docs/implementation-plan.md` still mention `tokmd serve`.

## 🧭 Options considered
### Option A (recommended)
- Create a learning PR to document the friction and rollback the incorrect documentation changes.
- Fits the `tooling-governance` shard by preserving context when a fix is superseded or incorrect.
- Trade-offs: Structure (captures learning without incorrect changes), Velocity (safely aborts redundant work).

### Option B
- Force the incorrect documentation changes anyway.
- When to choose: Never.
- Trade-offs: Violates the reviewer's feedback and introduces inaccurate information into the truth sources.

## ✅ Decision
Option A was chosen. Since the intended patch was superseded and based on a misunderstanding of the roadmap (treating future work as already shipped via a different command), the correct action is to record the learning and abort the patch.

## 🧱 Changes made (SRP)
- Created `.jules/friction/open/FRIC-20231027-001.md` to track the documentation clarity issue.
- Restored original documentation files (`docs/PRODUCT.md`, `docs/implementation-plan.md`, `ROADMAP.md`).

## 🧪 Verification receipts
```text
git restore docs/PRODUCT.md docs/implementation-plan.md ROADMAP.md
```

## 🧭 Telemetry
- Change shape: Learning PR and Friction Item
- Blast radius: `.jules/` artifacts
- Risk class: None (no functional changes)
- Rollback: None needed.
- Gates run: None applicable to the learning PR.

## 🗂️ .jules artifacts
- `.jules/runs/cartographer_roadmap_design/envelope.json`
- `.jules/runs/cartographer_roadmap_design/decision.md`
- `.jules/runs/cartographer_roadmap_design/receipts.jsonl`
- `.jules/runs/cartographer_roadmap_design/result.json`
- `.jules/runs/cartographer_roadmap_design/pr_body.md`
- `.jules/friction/open/FRIC-20231027-001.md`

## 🔜 Follow-ups
- Address the clarity of MCP Server vs Tool generation in documentation: `FRIC-20231027-001`
