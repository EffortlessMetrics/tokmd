## 💡 Summary
Removed duplicated instructions regarding the `notes/` directory from 16 different persona README files. These identical rules were consolidated into the shared `.jules/README.md` to reduce duplication, while preserving any prompt-critical instructions that are specific to a persona.

## 🎯 Why
Target #4 for the Archivist persona is to "move only neutral shared conventions into shared guidance; keep prompt-critical persona instructions in the individual persona README files." The 16 persona README files had exact duplicated rules instructing the agent to use `notes/` for reusable learnings. Centralizing this rule removes boilerplate and consolidates shared policy where it belongs.

## 🔎 Evidence
- file paths: `.jules/personas/*/README.md` and `.jules/README.md`
- command: `grep -rn "Use this persona's \`notes/\` directory" .jules/personas/` returned 16 matches before the change, and 0 matches after the change.

## 🧭 Options considered
### Option A (recommended)
- what it is: Consolidate the duplicated `notes/` directory rule from the persona READMEs into a single shared rule in `.jules/README.md`, retaining prompt-critical lines.
- why it fits this repo and shard: It directly addresses the Archivist target #4 and fits the `workspace-wide` shard mandate for meta/structural improvements.
- trade-offs: Structure: High, removes 32 lines of duplicated boilerplate. Velocity: High, future personas don't need this boilerplate. Governance: High, policy updates happen in one place.

### Option B
- what it is: Add a new document like `.jules/policy/notes.md` explaining how to use `notes/` but keep the duplicated text in the READMEs.
- when to choose it instead: If the shared docs were getting too large or if the duplicated text was fundamentally different per persona.
- trade-offs: Increases documentation fragmentation and requires maintaining two sources of truth.

## ✅ Decision
Option A was chosen because it directly fulfills the Archivist persona's mission to move neutral shared conventions into shared guidance while reducing duplication across 16 files.

## 🧱 Changes made (SRP)
- Modified `.jules/README.md` to include the shared rule about the `notes/` directory.
- Modified `.jules/personas/archivist/README.md` to remove the duplicated lines but keep its prompt-critical line.
- Modified `.jules/personas/auditor/README.md` to remove the duplicated lines.
- Modified `.jules/personas/bolt/README.md` to remove the duplicated lines.
- Modified `.jules/personas/bridge/README.md` to remove the duplicated lines.
- Modified `.jules/personas/cartographer/README.md` to remove the duplicated lines.
- Modified `.jules/personas/compat/README.md` to remove the duplicated lines.
- Modified `.jules/personas/fuzzer/README.md` to remove the duplicated lines.
- Modified `.jules/personas/gatekeeper/README.md` to remove the duplicated lines.
- Modified `.jules/personas/invariant/README.md` to remove the duplicated lines.
- Modified `.jules/personas/librarian/README.md` to remove the duplicated lines.
- Modified `.jules/personas/mutant/README.md` to remove the duplicated lines.
- Modified `.jules/personas/palette/README.md` to remove the duplicated lines.
- Modified `.jules/personas/sentinel/README.md` to remove the duplicated lines.
- Modified `.jules/personas/specsmith/README.md` to remove the duplicated lines.
- Modified `.jules/personas/steward/README.md` to remove the duplicated lines.
- Modified `.jules/personas/surveyor/README.md` to remove the duplicated lines.

## 🧪 Verification receipts
```text
$ grep -rn "Use this persona's \`notes/\` directory" .jules/personas/
<no output>

$ cat .jules/README.md | grep -A 5 "Agents may write"
### Agents may write
- unique per-run packet files
- friction items under `.jules/friction/open/`
- persona-local notes under `.jules/personas/<persona>/notes/`
  *(Use this directory only for **reusable learnings** that later runs can benefit from. Do not write per-run summaries here; per-run packets belong under `.jules/runs/<run-id>/`.)*
```

## 🧭 Telemetry
- Change shape: Documentation refactoring
- Blast radius: Jules documentation only (no code or runtime changes)
- Risk class: Low
- Rollback: `git restore .jules/README.md .jules/personas/*/README.md`
- Gates run: `cargo xtask docs --check`, `cargo fmt -- --check`

## 🗂️ .jules artifacts
- `.jules/runs/archivist_jules/envelope.json`
- `.jules/runs/archivist_jules/decision.md`
- `.jules/runs/archivist_jules/receipts.jsonl`
- `.jules/runs/archivist_jules/result.json`
- `.jules/runs/archivist_jules/pr_body.md`

## 🔜 Follow-ups
None.
