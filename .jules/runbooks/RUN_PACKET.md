# Jules Run Packet

Each run writes a self-contained packet under:

`.jules/runs/<run-id>/`

Required files:

- `envelope.json`
- `decision.md`
- `receipts.jsonl`
- `result.json`
- `pr_body.md`

## Why this exists

Per-run packets are the primary truth for Jules work. They are easier to review, easier to compare mechanically, and safer for concurrent one-shot agents than shared append-only ledgers or daily logs.

## File roles

### `envelope.json`
Machine-readable planning and execution envelope.

Includes:
- prompt id
- persona
- style
- primary shard
- gate profile
- allowed outcomes
- Option A / Option B and final selection
- artifact paths
- command list

### `decision.md`
Narrative context for later reviewers and LLMs.

Must include:
- what was inspected
- Option A / Option B
- why one option was chosen
- what was intentionally not pursued

### `receipts.jsonl`
One JSON object per command or verification step as it happens.

Recommended fields:
- `ts_utc`
- `phase`
- `cwd`
- `cmd`
- `status`
- `summary`
- `key_lines`
- `artifacts`

### `result.json`
Machine-readable result summary.

Includes:
- outcome type (`patch`, `proof_patch`, `learning_pr`)
- reviewer-facing title
- summary
- target paths
- proof summary
- gates run
- friction items created
- persona notes created
- rollback
- follow-ups

### `pr_body.md`
Reviewer-facing narrative built from the receipts and result files, not from memory.

## Learning PR rule

If the run does not justify a code/docs/test patch, it must still finish with a **learning PR** containing:
- the run packet
- one or more friction items
- optional persona notes

That keeps the prompt-to-PR pipeline unblocked and preserves the learning.
