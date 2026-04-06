# Decision

## Problem
The execution docs (specifically `docs/tutorial.md`) mention `tokmd analyze --preset estimate` with the `--effort-base-ref main --effort-head-ref HEAD` flags, but there is no corresponding doctest for this in `crates/tokmd/tests/docs.rs`. We should add executable coverage for these common usage commands to ensure they do not silently drift.

## Option A
Add the missing doctest to `crates/tokmd/tests/docs.rs` for `tokmd analyze --preset estimate --effort-base-ref HEAD --effort-head-ref HEAD --format md` (using `HEAD` for both to avoid relying on `main` which might not exist in CI shallow clones).

## Option B
Update `docs/tutorial.md` to remove the references to base/head refs if they are not meant to be heavily advertised or tested.

## Selection
**Option A**. The documentation explicitly shows this feature as step 8.5 for estimating effort, so we should ensure it's tested. Following the rule: "When writing tests for git-dependent CLI commands... explicitly set the base reference using environment variables or CLI flags like `--base HEAD` to guarantee a clean baseline and avoid failures in shallow-clone or sandboxed environments.", we'll test with `HEAD` as the ref.
