## 💡 Summary
Removed the identical `## Notes` boilerplate from all 16 `.jules/personas/*/README.md` files and consolidated the instructions into a new `### Persona Notes Directory` section in `.jules/README.md`.

## 🎯 Why
Every persona's `README.md` included the exact same note about how to use the `notes/` directory versus the `runs/` directory. This violates DRY and distracts from the core mission definition of each persona. As Archivist, consolidating duplicated conventions into neutral shared guidance is an explicit mission goal.

## 🔎 Evidence
Observation: Every persona file contained this block:
```markdown
## Notes
Use this persona's `notes/` directory only for **reusable learnings** that later runs can benefit from.
Do not write per-run summaries here; per-run packets belong under `.jules/runs/<run-id>/`.
```

Command showing the duplication:
```bash
grep -A 3 "## Notes" .jules/personas/*/README.md
```

## 🧭 Options considered
### Option A (recommended)
- Consolidate the notes instructions directly into `.jules/README.md`.
- Fits this repo and shard because `.jules/README.md` is the central source of truth for `.jules/` directory structure and storage rules.
- Trade-offs: Structure/Velocity/Governance - Reduces noise in 16 files, improves structural clarity.

### Option B
- Add a `.jules/policy/persona_notes.md` file and keep a link in every persona README.
- When to choose it instead: If the rule was incredibly lengthy.
- Trade-offs: Still leaves boilerplate in 16 files.

## ✅ Decision
Chose Option A to minimize duplication and centralize directory storage rules in `.jules/README.md`.

## 🧱 Changes made (SRP)
- `.jules/README.md`: Added `### Persona Notes Directory` to document `notes/` usage.
- `.jules/personas/*/README.md`: Removed the duplicated `## Notes` section from all 16 personas.

## 🧪 Verification receipts
```text
$ cargo xtask docs --check
Documentation is up to date.

$ cargo fmt -- --check
```

## 🧭 Telemetry
- Change shape: Docs refactor (scaffolding improvement)
- Blast radius: `.jules/` documentation only
- Risk class: Low
- Rollback: Revert the `.jules` README modifications.
- Gates run: `cargo xtask docs --check`, `cargo fmt -- --check`

## 🗂️ .jules artifacts
- `.jules/runs/archivist_jules/envelope.json`
- `.jules/runs/archivist_jules/decision.md`
- `.jules/runs/archivist_jules/receipts.jsonl`
- `.jules/runs/archivist_jules/result.json`
- `.jules/runs/archivist_jules/pr_body.md`

## 🔜 Follow-ups
None.
