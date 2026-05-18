# Friction Item: Test setup boundary execution

id: sentinel_git_command_tests
persona: sentinel
style: stabilizer
shard: interfaces
status: closed

## Context
When searching for raw `Command::new("git")` usages that violate the trusted
`tokmd_git::git_cmd()` boundary, several instances were found in test setup
logic and developer tooling.

## Friction
Those occurrences created confusion during threat modeling: some Git calls are
operational product behavior, while others only scaffold local fixture repos or
drive `xtask` proof/publishing utilities.

## Resolution
Sentinel guidance now distinguishes product/runtime subprocess boundaries from
test fixture setup and xtask-local repository plumbing. Raw subprocess calls in
tests or developer tooling are not automatically security-significant; patch
them only when they create a real trust, determinism, or maintenance problem.

## Evidence
- `.jules/runs/sentinel_boundaries/decision.md`
- `.jules/runs/sentinel_boundaries/pr_body.md`
- `.jules/personas/sentinel/README.md`

## Done when
- [x] Sentinel guidance explicitly distinguishes product/runtime subprocess
  boundaries from test fixture setup and developer tooling.
- [x] Future Sentinel runs have a clear path to record the classification
  without forcing a fake security patch.
