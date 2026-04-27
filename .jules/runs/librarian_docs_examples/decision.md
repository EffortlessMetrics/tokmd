## Option A (recommended)
Update the `tokmd check-ignore` command to return an error when the given path does not exist.

- Why it fits this repo and shard: It enforces deterministic behavior in CI environments, preventing silent failures when users mistype a file name, directly aligning with the documentation in `docs/reference-cli.md`.
- Trade-offs:
  - Structure: Minimal change to `check_ignore.rs` and its associated tests.
  - Velocity: Modifies multiple tests that check for `not ignored` outputs, requiring updates to standard error assertions.
  - Governance: Aligns actual system behavior with the explicit contract laid out in documentation.

## Option B
Update the documentation to indicate that `check-ignore` gracefully handles missing files by reporting them as "not ignored (file not found)" with exit code 0.

- When to choose it instead: If the goal is strictly a non-failing utility that parses globs lazily without needing strict system validation on path existence.
- Trade-offs: Degrades the CI safety guarantees of `check-ignore`. The `docs/reference-cli.md` explicitly calls out the change in v1.3.0 to fail on missing paths to prevent CI silent failures. Option B would roll back this intended governance.

## Decision
I chose Option A because the documentation explicitly states: "As of v1.3.0, specifying a non-existent input path returns exit code 1 with an error message, rather than succeeding with empty output. This prevents silent failures in CI pipelines." Fixing the code to match this intended, stricter contract provides a better user experience and fulfills the stated governance policy.
