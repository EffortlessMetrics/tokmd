# Explicit Progress Events

When running long or multi-step workflows, emit explicit progress events so operators can track state transitions.

## Suggested event pattern

1. `start` — announce task intent and scope.
2. `in_progress` — announce each major step with a sequence marker (for example `2/5`).
3. `done` — announce completion and summarize outputs.

## Event content checklist

- A stable step counter (for example `Progress: 3/5`).
- A short description of what just completed.
- The next action to be taken.

## Example

- `Progress: 1/3 — Validated configuration. Next: run scan.`
- `Progress: 2/3 — Scan complete. Next: render report.`
- `Progress: 3/3 — Report generated.`
