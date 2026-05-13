# Jules Goals

This directory stores machine-readable active-agent state for Jules and related
automation.

The primary file is:

- `active.toml`

It should stay small and point to durable human-readable docs. It is not a run
log, not a chat transcript, and not a replacement for proposals, specs, ADRs,
plans, or policy files.

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
