## 💡 Summary
Moved duplicated `## Notes` section from 16 persona READMEs into neutral shared guidance in `.jules/README.md`.

## 🎯 Why
This addresses Archivist target #4: "move duplicated persona-local conventions into neutral shared guidance." Every persona README had identical boilerplate instructing agents not to write per-run summaries into the `notes/` directory, which is better served as a centralized storage rule.

## 🔎 Evidence
- `.jules/README.md`
- `.jules/personas/*/README.md`
- The same 3-line note existed in 16 files and is now only in 1 central location.

## 🧭 Options considered
### Option A (recommended)
- Move guidance to `.jules/README.md` under `## Persona Notes`.
- Remove from all persona READMEs.
- **Trade-offs:** Makes persona READMEs leaner, centralizes policy.

### Option B
- Write a new shared runbook `PERSONA_NOTES.md`.
- **Trade-offs:** Adds unnecessary indirection. The root README is already the directory guide.

## ✅ Decision
Option A was chosen because `.jules/README.md` already defines storage rules, making it the perfect location.

## 🧱 Changes made (SRP)
- Add `## Persona Notes` section to `.jules/README.md`
- Remove `## Notes` section from `.jules/personas/*/README.md` (16 files)

## 🧪 Verification receipts
```json
{"ts_utc": "2024-04-06T12:00:00Z", "phase": "implementing", "cwd": "/", "cmd": "python3 patch_readmes.py", "status": "PASS", "summary": "Removed redundant `## Notes` section from all persona READMEs"}
{"ts_utc": "2024-04-06T12:00:00Z", "phase": "implementing", "cwd": "/", "cmd": "python3 patch_jules_readme.py", "status": "PASS", "summary": "Added centralized `## Persona Notes` section to `.jules/README.md`"}
```

## 🧭 Telemetry
- Change shape: Docs deduplication
- Blast radius: `.jules/` documentation only
- Risk class: Low
- Rollback: Revert commit
- Gates run: `cargo xtask gate --check`

## 🗂️ .jules artifacts
- Run packet written to `.jules/runs/archivist_jules_run/`

## 🔜 Follow-ups
None.
