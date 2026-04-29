# Friction Item

## What happened
The agent tried to fix roadmap drift in `docs/implementation-plan.md` to reflect Phase 5 completion, but a concurrent or previous PR (#1339) had already resolved it, and the user closed the pull request as superseded.

## Why it caused friction
The prompt-to-PR pipeline requires a PR-worthy diff at the end. Since the primary patch was no longer needed, the agent must fallback to a learning PR.

## Recommendation
Future agents should ensure they pull the absolute latest `main` branch before starting their task to minimize race conditions, or gracefully exit with a learning PR if superseded.
