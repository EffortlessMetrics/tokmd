# Maintainer takeover (Jules PR -> replacement PR)

Use this when a Jules PR is close but not mergeable, or when you want to reconcile multiple small PRs.

## Outcomes (no third option)
1) Ship a replacement PR (clean, on latest base, verified gates). Close original as superseded.
2) Close and document, but still fix forward with a better PR if a scoped improvement exists.

## Invariants
- Pin repo targeting to origin.
- Cherry-pick onto latest base first.
- Run merge-confidence gates.
- PR description includes telemetry and receipts.

## Telemetry minimum
- change shape
- blast radius
- risk and rollback
- verification receipts (commands and outcomes)
- decision log (A/B and why chosen)

This is “review artifacts, not chats”.
