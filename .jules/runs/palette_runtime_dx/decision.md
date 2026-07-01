## Target Selection
I noticed that the `--max-commits` and `--max-commit-files` arguments for `tokmd analyze` and `tokmd badge` commands lacked validation. While `tokmd context` correctly used `value_parser = super::validate::positive_usize` to prevent a degenerate zero value, `analyze` and `badge` did not, allowing `0` to silently perform no git history scanning or to exit with a non-obvious error deeper down.

## Options considered
### Option A (recommended)
- Add `value_parser = super::validate::positive_usize` to `max_commits` and `max_commit_files` in `crates/tokmd/src/cli/parser/analysis.rs` and `crates/tokmd/src/cli/parser/badge.rs`.
- Why it fits this repo and shard: It directly improves the runtime developer experience by catching invalid CLI usage (0 commits) at parse time with a clear error message, matching the behavior already implemented for `tokmd context`. It is within the `interfaces` shard (specifically `crates/tokmd/src/cli/parser`).
- Trade-offs: Small structure change for improved velocity and clarity for end-users.

### Option B
- Modify the git scanning logic to treat `0` as an error or a no-op instead of validating at the CLI boundary.
- When to choose it instead: If CLI validation is impossible or if 0 should be a valid configuration value with special semantic meaning.
- Trade-offs: Poorer UX since the error is delayed and might not attribute directly to the command line argument. It also deviates from the established pattern in `validate.rs`.

## Decision
I chose Option A because it provides immediate, clear feedback to the user at the CLI parsing stage, preventing silent failures or confusing downstream errors. This perfectly aligns with the Palette persona's goal of improving runtime developer experience and fixing CLI help/usage sharp edges.
