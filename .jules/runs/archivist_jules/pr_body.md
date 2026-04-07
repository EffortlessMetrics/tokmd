## 💡 Summary
Extracted the duplicated persona README structure into a shared neutral template to reduce friction when creating or updating personas.

## 🎯 Why
Currently, every persona duplicates the exact same structural headings (`Gate profile`, `Mission`, `Target ranking`, `Proof expectations`, `Anti-drift rules`). Consolidating this expected shape into `.jules/runbooks/PERSONA_TEMPLATE.md` satisfies the Archivist goal to move duplicated persona-local conventions into neutral shared guidance.

## 🔎 Evidence
- `.jules/runbooks/PERSONA_TEMPLATE.md`

## 🧭 Options considered
### Option A (recommended)
- What it is: Create `.jules/runbooks/PERSONA_TEMPLATE.md` detailing the required markdown structure.
- Why it fits this repo and shard: Directly fulfills Archivist target ranking 1 and 4.
- Trade-offs: Structure / Velocity / Governance: Improves governance and structure with minimal velocity hit.

### Option B
- What it is: Do nothing and write a learning PR.
- When to choose it instead: If no structural duplication existed.
- Trade-offs: Misses an easy structural win.

## ✅ Decision
Option A. It's a low-risk, high-value extraction of implicit shared state into explicit scaffolding.

## 🧱 Changes made (SRP)
- Created `.jules/runbooks/PERSONA_TEMPLATE.md`

## 🧪 Verification receipts
```text
{"ts_utc": "2024-04-07T16:00:00Z", "phase": "patch", "cwd": ".jules", "cmd": "write_file PERSONA_TEMPLATE.md", "status": "success", "summary": "Created shared persona template"}
{"ts_utc": "2024-04-07T16:01:00Z", "phase": "gates", "cwd": ".", "cmd": "cargo xtask version-consistency && cargo xtask docs --check && cargo fmt -- --check", "status": "success", "summary": "Checked versions, docs, and format"}
```

## 🧭 Telemetry
- Change shape: Docs addition
- Blast radius: Internal Jules scaffolding
- Risk class: Low
- Rollback: rm .jules/runbooks/PERSONA_TEMPLATE.md
- Gates run: version-consistency, docs --check, fmt --check

## 🗂️ .jules artifacts
- `.jules/runs/archivist_jules/envelope.json`
- `.jules/runs/archivist_jules/decision.md`
- `.jules/runs/archivist_jules/receipts.jsonl`
- `.jules/runs/archivist_jules/result.json`
- `.jules/runs/archivist_jules/pr_body.md`

## 🔜 Follow-ups
Update existing personas to rely on the shared template.
