# .jules

Checked-in Jules adapter surface for this repo.

Tracked here:
- Jules-specific settings, hooks, agents, and command shims
- curated Jules history that is intentionally kept in git
  - notably `.jules/deps/**` receipts, ledgers, and run notes
- adapter docs that explain how Jules maps onto the shared repo contract

Not tracked here:
- worktrees
- ephemeral runs
  - notably root-level `.jules/runs/`
- caches
- transcripts
- other runtime-only state

`.jules/` is not spillover by default. The cleanup target is runtime state, not durable Jules history.
