# Handoff Prompt

Status: active workflow guide.

Use this when you have generated a `.handoff/` bundle and want Codex, Claude,
Cursor, or another coding agent to consume it without inventing context from
chat history.

This is a prompt template, not a second planning system. The handoff bundle and
linked receipts remain the evidence sources.

## Copy-Ready Prompt

```text
Work only from the provided tokmd handoff bundle unless I explicitly give you
additional repo context.

Read `.handoff/work-order.md` first. Use it as the task map, evidence summary,
and stop-condition list.

Use `.handoff/code.txt` as the bounded source bundle. Use `.handoff/manifest.json`
for the authoritative artifact index, included files, exclusions, and token
budget. Use `.handoff/map.jsonl` only when you need full path lookup.

If `.handoff/review-links.json` exists, treat it as a handle to cockpit review
evidence. Open the linked `review-map.md` for review order and reproduction
commands, and open the linked review-packet verifier receipt before trusting
packet-local hashes.

If `.handoff/proof-links.json` exists, treat it as a handle to proof-route,
affected-proof, and proof-plan evidence. A proof route is selection and
skip-policy evidence, not executed proof. A proof plan is expected proof, not
executed proof. Do not claim proof passed until required proof commands have
run or are explicitly deferred.

If `.handoff/evidence-packet-links.json` exists, treat it as a handle to a tokmd
evidence packet manifest. Use its `review_priority` as first-read order, not as
a verdict, and resolve or acknowledge partial/failed packet status before
claiming complete review evidence.

Treat missing, stale, degraded, skipped, or unavailable evidence as work to
resolve, not as passing proof.

Do not broaden the lane unless the work order asks for it. Do not promote
advisory proof, enable default Codecov upload, add AST default behavior, create
a merge verdict, or change release state.

Stop when the requested change is complete and the listed proof expectations
are satisfied or explicitly deferred with a clear reason.
```

## Required Guardrails

- Linked review, proof, and evidence-packet evidence is a handle, not copied or
  verified proof.
- `work-order.md` summarizes linked receipts, but the linked verifier, proof
  receipts, and evidence packet manifest remain the evidence sources of truth.
- Missing, stale, degraded, skipped, unavailable, partial, or failed evidence is
  not passing proof.
- Planned proof is not executed proof.
- Cockpit and handoff outputs are not merge verdicts.
- Advisory proof, coverage, mutation, browser output, and Codecov upload remain
  advisory unless policy explicitly promotes them.

## Related References

- [Handoff bundles](../handoff.md)
- [User paths](../user-paths.md)
- [Artifact glossary](../artifacts.md)
- [Source-of-truth workflow](source-of-truth.md)
