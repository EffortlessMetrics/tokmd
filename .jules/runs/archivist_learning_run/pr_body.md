## 💡 Summary
Recorded a learning that persona-specific rules must stay in the persona files, even if they appear duplicated, because of how agents consume them.

## 🎯 Why
During an attempt to deduplicate the `## Notes` section from 16 persona READMEs into `.jules/README.md`, PR feedback indicated that "prompt-critical guidance must stay in the individual persona files because Jules receives personas individually." This PR records that learning to prevent future runs from making the same mistake.

## 🔎 Evidence
- `.jules/personas/*/README.md` all have identical `## Notes` sections.
- Attempted to deduplicate in PR, but reviewer rejected it: "prompt-critical guidance must stay in the individual persona files because Jules receives personas individually."

## 🧭 Options considered
### Option A
- Revert the deduplication patch and force a code change elsewhere.
- **Trade-offs:** Finding a different change might be forced or hallucinated, violating the "No tool cargo-culting" and "Output honesty" rules.

### Option B (recommended)
- Accept the PR feedback. Since no honest code/docs/test patch is justified for this specific issue anymore, finish with a learning PR instead.
- Write the per-run packet, friction items, and persona notes explaining the learning.

## ✅ Decision
Option B is the correct path for a prompt-to-PR pipeline when a patch is no longer viable.

## 🧱 Changes made (SRP)
- Add friction item `FRIC-20260406-001`
- Add persona note `persona_duplication.md`

## 🧪 Verification receipts
```json
{"ts_utc": "2026-04-06T13:30:00Z", "phase": "review", "cwd": "/", "cmd": "read_pr_comments", "status": "PASS", "summary": "Read PR feedback indicating persona notes must remain duplicated.", "key_lines": [], "artifacts": []}
```

## 🧭 Telemetry
- Change shape: Learning PR
- Blast radius: `.jules/` documentation only
- Risk class: Low
- Rollback: Revert commit
- Gates run: `cargo xtask gate --check`

## 🗂️ .jules artifacts
- Run packet written to `.jules/runs/archivist_learning_run/`
- Friction item written to `.jules/friction/open/FRIC-20260406-001.md`
- Persona note written to `.jules/personas/archivist/notes/persona_duplication.md`

## 🔜 Follow-ups
None.
