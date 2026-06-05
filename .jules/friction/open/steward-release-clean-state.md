id: FRIC-20240508-002
persona: Steward
style: Stabilizer
shard: tooling-governance
status: open

## Problem
Publishxtask surface demands a completely clean git state which can cause friction during automated builds or local debug sessions where untracked ignored files exist.

## Evidence
- files / paths: `xtask/src/tasks/publish.rs`
- related run ids: steward_1

## Why it matters
Causes false positive rejections of publish plans.

## Done when
- [ ] `git status --porcelain` check correctly ignores files that are configured in `.gitignore` or allows specific non-code ignored artifacts.
