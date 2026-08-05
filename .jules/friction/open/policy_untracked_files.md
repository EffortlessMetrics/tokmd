# Friction Item: Extraneous untracked test fixtures and scripts

## Context
When running `cargo xtask check-file-policy --strict`, it fails pointing out that:
- `fixtures/syntax/python/native_boundary.py`
- `fixtures/syntax/typescript/component.tsx`
- `fixtures/syntax/typescript/native_boundary.ts`
- `scripts/check-no-bare-self-hosted.sh`

do not match any non-Rust allowlist glob. These files existed in the repository prior to my changes (likely generated during the original git checkout/test run) but are not staged by my tool or tracked.

## Friction
The `check-file-policy` task enforces strict non-Rust file paths, but the repository contains testing/stub files that cause `check-file-policy` to fail by default. This makes it impossible to naturally pass CI locally without deleting these untracked files or adding allowlists to the policy.

## Recommendation
Future developers should ensure these loose scripts and `fixtures/` are either tracked in the allowlist glob (e.g., `xtask/src/tasks/policy.rs`) or removed from the baseline tree to not cause out-of-the-box pipeline friction.
