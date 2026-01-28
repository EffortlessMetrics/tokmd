# Designing Jules Scheduled Tasks (tokmd)

Scheduled Tasks create a recurring contributor. A good prompt is an operating contract.

## Goal

Maximize SRP-quality improvement per reviewer minute.

This is a mix of:
- reduce review time (boring PRs)
- increase quality delta per PR (meaningful SRP wins)

## Non-negotiables

### 1) SRP scope
One coherent improvement per run. No grab-bag cleanups.

### 2) Two-lane selection
- Lane A: friction backlog (compounding known pain)
- Lane B: scout discovery (preserve Jules’ unique finds)

Constrain blast radius and verification. Do not constrain curiosity.

### 3) No “ask first”
Scheduled work cannot ask. It must:
- provide options A/B
- choose
- document
- proceed

### 4) Receipts (staged)
Do not promise tests. Prove them.
- Write a run envelope early.
- Append command results immediately after running them.
- Copy PR receipts from the envelope.

### 5) Verification is blast-radius aware
Default to repo merge-confidence gates. Downshift only if clearly justified.

### 6) Encode standards in-repo
PR layout and friction templates live in `.jules/runbooks/`.

## Preference knobs

### Selection strategy (random vs priority)
This is a preference and a scaling tactic.
- `random`: reduces collisions across concurrent runs
- `priority`: concentrates effort; multiple solutions can be reconciled later

Set this in `.jules/policy/scheduled_tasks.json`, not per prompt.
