# .claude

Checked-in Claude adapter surface for this repo.

Tracked here:
- runtime-specific settings and hooks
- checked-in Claude agent manifests and command shims
- adapter docs that explain how Claude maps onto the shared repo contract

Not tracked here:
- worktrees
- caches
- transcripts
- other runtime-only state

Shared repo policy should live outside this directory. `.claude/` is the Claude-specific adapter over that shared contract.
