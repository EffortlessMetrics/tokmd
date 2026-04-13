# .jules/

State lives on disk. Written = real.

## Purpose

This directory stores the durable execution packets, friction items, reusable notes, and policy/runbook files that make Jules runs reviewable and improvable over time.

## Jules operating model

Jules runs in this repo are **async one-shot branch authors**.

That means each run:

- starts from a fresh clone
- cannot wait
- cannot ask first
- cannot inspect the live PR board
- cannot rely on PR ops
- must decide, document, support, and finish

## SRP meaning in this repo

**SRP means one coherent reviewer story, not one small diff.**

Large focused improvements are good if they stay one coherent story and keep tests/docs/contracts aligned.

## Valid outcomes for a Jules run

A successful Jules run may end with any of these:

1. **PR-ready patch**
2. **proof-improvement patch**
3. **learning PR** that contains:
   - the per-run packet
   - one or more friction items
   - optional persona notes

Waiting is failure.
Hallucinated fixes are failure.

## Primary truth

The primary truth for any run is the **per-run packet** under:

- `.jules/runs/<run-id>/envelope.json`
- `.jules/runs/<run-id>/decision.md`
- `.jules/runs/<run-id>/receipts.jsonl`
- `.jules/runs/<run-id>/result.json`
- `.jules/runs/<run-id>/pr_body.md`

## Storage rules

### Agents may write
- unique per-run packet files
- friction items under `.jules/friction/open/`
- persona-local notes under `.jules/personas/<persona>/notes/`

### Agents must not write
- shared append-only ledgers as primary truth
- shared daily logs as primary truth
- shared runbooks/policy/templates unless the prompt explicitly says the run is a Jules-scaffolding run

## Shared directories

- `policy/` — shared policy, gate profiles, shard maps, schemas
- `runbooks/` — neutral templates and packet docs
- `friction/` — structured future-work queue
- `personas/` — persona-local notes and README files
- `runs/` — per-run packets
- `index/` — optional generated summaries

## Persona instruction surface

Prompt-critical guidance must live in the individual persona README files under
`.jules/personas/<persona>/README.md`.

Those files are the direct execution surface for persona-specific Jules runs.
Shared docs in `.jules/README.md`, `runbooks/`, or `policy/` may summarize or
reinforce that guidance, but they do not replace persona-local instructions.

## Learning and improvement intent

The intent of keeping run packets, friction items, and persona notes in-repo is to:

- document what Jules attempted
- preserve receipts for later reviewers and LLMs
- identify recurring failure modes and friction
- improve future prompts, shards, templates, and gate profiles
