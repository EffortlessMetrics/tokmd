# Plans

Plans own implementation sequencing. They turn accepted direction into small
reviewable PR packets, validation commands, dependencies, and stop conditions.

## Use This Directory For

- milestone plans;
- PR-by-PR work packets;
- validation ladders;
- migration order;
- explicit stop or pivot conditions.

## Do Not Use It For

- final behavior contracts;
- durable architecture decisions;
- machine-checkable policy;
- raw execution logs.

## Suggested Shape

```md
# Plan: <title>

- Status: active | paused | complete | superseded
- Related proposal:
- Related spec:
- Related ADR:
- Related issues:

## Goal

## Non-goals

## Work Packets

## Validation

## Stop Conditions

## Checkpoint History
```

Plans should be updated when sequencing changes. Once a plan is complete, leave
a short checkpoint and link to the merged PRs rather than continuing to append
daily status.
