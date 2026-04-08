## 💡 Summary
Moved duplicated `## Notes` guidance out of 16 individual persona README files and centralized it into `.jules/README.md`. This directly addresses Archivist target #4 ("move duplicated persona-local conventions into neutral shared guidance").

## 🎯 Why
Every persona had the exact same instruction text at the bottom telling agents not to put run packets in the `notes/` directory. Centralizing this reduces boilerplate and makes `.jules/README.md` the authoritative source for run artifact storage rules.

## 🔎 Evidence
- **Files**: `.jules/README.md`, `.jules/personas/*/README.md`
- **Issue**: Repetitive boilerplate across 16 files.
- **Proof**: `grep -r "## Notes" .jules/personas/` now returns empty.

## 🧭 Options considered
### Option A (recommended)
- **What it is**: Remove duplicated `## Notes` from persona files and add the rule to `.jules/README.md`.
- **Why it fits**: Directly fulfills Archivist target #4.
- **Trade-offs**: Centralizes governance rules in one place, making the individual persona files smaller.

### Option B
- **What it is**: Build a Python tool to roll up per-run summaries into `.jules/index/generated/rollups.md`.
- **When to choose it instead**: If generating summary indexes was more urgent than cleaning up existing copy-pasted docs.
- **Trade-offs**: Introduces generated artifacts that might drift over time.

## ✅ Decision
Selected **Option A**. It's a clean, zero-risk structural cleanup that centralizes policy and removes 48 lines of repetitive boilerplate.

## 🧱 Changes made (SRP)
- Modified `.jules/README.md` to include notes directory usage rules.
- Removed `## Notes` section from `.jules/personas/archivist/README.md`
- Removed `## Notes` section from `.jules/personas/auditor/README.md`
- Removed `## Notes` section from `.jules/personas/bolt/README.md`
- Removed `## Notes` section from `.jules/personas/bridge/README.md`
- Removed `## Notes` section from `.jules/personas/cartographer/README.md`
- Removed `## Notes` section from `.jules/personas/compat/README.md`
- Removed `## Notes` section from `.jules/personas/fuzzer/README.md`
- Removed `## Notes` section from `.jules/personas/gatekeeper/README.md`
- Removed `## Notes` section from `.jules/personas/invariant/README.md`
- Removed `## Notes` section from `.jules/personas/librarian/README.md`
- Removed `## Notes` section from `.jules/personas/mutant/README.md`
- Removed `## Notes` section from `.jules/personas/palette/README.md`
- Removed `## Notes` section from `.jules/personas/sentinel/README.md`
- Removed `## Notes` section from `.jules/personas/specsmith/README.md`
- Removed `## Notes` section from `.jules/personas/steward/README.md`
- Removed `## Notes` section from `.jules/personas/surveyor/README.md`

## 🧪 Verification receipts
```text
$ grep -r "## Notes" .jules/personas/
<empty>
```

## 🧭 Telemetry
- **Change shape**: Documentation cleanup.
- **Blast radius**: `docs` (specifically `.jules/` scaffolding). Zero impact on API/IO/compatibility/dependencies.
- **Risk class**: Trivial. Non-executable files only.
- **Rollback**: `git revert`
- **Gates run**: `cargo xtask docs --check`, `cargo xtask version-consistency`, `cargo fmt -- --check`, `cargo clippy -- -D warnings`, `cargo test`

## 🗂️ .jules artifacts
- `.jules/runs/archivist_jules/envelope.json`
- `.jules/runs/archivist_jules/decision.md`
- `.jules/runs/archivist_jules/receipts.jsonl`
- `.jules/runs/archivist_jules/result.json`
- `.jules/runs/archivist_jules/pr_body.md`

## 🔜 Follow-ups
None.
