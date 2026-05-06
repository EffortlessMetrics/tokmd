# Friction Record: PR Superseded

## Summary
The work to modify `proof-artifacts-check` to allow `execution_guard.enabled=true` was superseded by PR #1657.

## Details
PR #1657 kept the current artifact-verifier contract on main. It correctly identifies that enabled guards are only allowed for non-executed artifacts, and that executed artifacts require a future execution verifier. The draft's `.jules` receipts were stale against the current guard wording, leading to the decision to abandon the AI's proposed approach in favor of the maintainer's PR.

## Action Taken
Work was stopped on the current prompt, and the PR comment was acknowledged. This learning is recorded as a friction item to prevent future duplication of effort on this issue and to update the understanding of the execution guard's role in artifact verification.
