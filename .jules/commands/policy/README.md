# Policy

`scheduled_tasks.json` is the repo's truth source for scheduled-agent behavior.

Keep it small. Treat it like config, not prose.

## Key knobs

- `selection_strategy`
  - `random`: reduces collisions across many scheduled runs
  - `priority`: focuses effort on the biggest problem first

- `default_gates`
  Merge-confidence gates the agent should run by default.

- `gate_commands`
  The concrete commands for each gate.

- `pr_title_style`
  We put the change first. Persona suffix at the end.

- `two_lane_selection`
  Friction backlog plus scout discovery.
