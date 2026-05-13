# Plan: <title>

- Status: active
- Related proposal:
- Related spec:
- Related ADR:
- Related issues:

## Goal

State the concrete end state for this implementation sequence.

## Non-goals

- List work that must stay out of this plan.

## Work Packets

1. First narrow PR:
2. Second narrow PR:
3. Follow-up or checkpoint:

## Validation

Each implementation PR should run the relevant subset of:

```bash
cargo xtask doc-artifacts --check
cargo xtask docs --check
cargo xtask proof-policy --check
cargo xtask affected --base origin/main --head HEAD --json-output target/proof/affected.json
cargo xtask proof --profile affected --base origin/main --head HEAD --plan
cargo fmt-check
git diff --check
```

## Stop Conditions

- Stop if the plan requires a behavior contract that no spec owns.
- Stop if the plan requires a durable architecture decision that no ADR owns.
- Stop if validation becomes noisy or broader than the lane justifies.

## Checkpoint History

- YYYY-MM-DD:
