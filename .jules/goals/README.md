# Jules Goals

This directory stores machine-readable active-agent state for Jules and related
automation.

The primary file is:

- `active.toml`

It should stay small and point to durable human-readable docs. It is not a run
log, not a chat transcript, and not a replacement for proposals, specs, ADRs,
plans, or policy files.

When a lane is complete, superseded, or paused with durable value, copy the
active goal into `archive/YYYY-MM-DD-lane-slug.toml`, update its `status`, and
leave `.jules/goals/active.toml` ready for the next current lane. The archive is
historical context only; it is not a second active queue.

## Allowed Content

- current program or lane name;
- links to the active plan/spec/ADR;
- current stop conditions;
- checked policy references;
- short notes that automation can parse.

## Disallowed Content

- raw terminal output;
- daily narrative logs;
- complete PR histories;
- pasted model transcripts.

## Archive

See [archive/README.md](archive/README.md) for the archive naming convention and
scope. The documentation artifact checker validates the active goal first; it
does not enforce archived-goal history in this initial control-plane slice.
