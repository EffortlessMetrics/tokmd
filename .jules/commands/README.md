# .jules

This directory is **tokmd’s repo-local control plane** for scheduled agents.

The point is simple:

**Maximize SRP-quality improvement per reviewer minute.**

That means two things at once:
- keep PRs reviewable (boring, legible, receipt-backed)
- make each PR *worth* reviewing (real delta, not lint confetti)

## Non-negotiable rule

**If it isn’t written, it didn’t happen.**

Scheduled work is unattended. Durable artifacts keep truth cheap.

## Mental model

- Scheduled Tasks create a recurring contributor.
- We constrain **shape** (SRP, receipts, verification, telemetry), not curiosity.
- We want “review artifacts, not chats.”

## Directory map

- `.jules/policy/`
  - `scheduled_tasks.json` — knobs (selection strategy, default gates, PR title style).

- `.jules/runbooks/`
  - `PR_GLASS_COCKPIT.md` — PR body template (source of truth).
  - `FRICTION_ITEM.md` — friction item template.
  - `SCHEDULED_TASK_PROMPTS.md` — how to write scheduled prompts.
  - `MAINTAINER_TAKEOVER.md` — reconcile/replace Jules PRs into one clean PR.
  - `ISSUE_STEWARD.md` — implementation-ready issue updates (issue comments).

- `.jules/friction/`
  - `open/` — one file per actionable pain point.
  - `done/` — moved here when resolved.

- `.jules/{palette,bolt,security,quality,docs,compat,deps}/`
  - `README.md` — domain heuristics.
  - `ledger.json` — append-only run index (machine-readable).
  - `envelopes/` — run envelopes (JSON receipts written as commands run).
  - `runs/` — short run logs (human-readable; kept small).
  - `notes/` — atomic reusable learnings (Zettelkasten-ish).

- `.jules/scheduled/`
  - repo-tuned prompts to paste into Jules.

- `.jules/repo/`
  - small repo facts, so prompts can stay short.

## Conventions

### Receipts (truth mechanism)
- Each run writes a JSON envelope early.
- Append command results immediately after running them.
- PR descriptions copy receipts from the envelope, not “memory.”

### Notes (Zettelkasten-ish)
Notes exist to prevent re-learning the same thing.
- Write only when reusable across future runs.
- Name: `YYYYMMDDTHHMMZ--short-title.md`.
- Include: context, evidence pointers, links.

### Friction items
- One issue per file.
- Must include: pain, evidence, done-when.
- Selection strategy (random vs priority) is a knob in `policy/`.

This folder is intentionally boring. That is the point.
