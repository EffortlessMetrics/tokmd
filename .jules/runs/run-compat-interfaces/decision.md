# Option A: Conditionally compile git invocations
Wrap `tokmd_git::git_cmd` usage in `#[cfg(feature = "git")]` blocks, providing fallback behavior when the git feature is disabled. This fixes the `--no-default-features` compilation error and respects feature boundaries.

# Option B: Make `tokmd-git` a required dependency
Remove `optional = true` from `tokmd-git` dependency and always include it. This would remove the ability to compile tokmd without git support, breaking the existing feature matrix.

# Decision
Option A. It correctly fixes the `--no-default-features` compat build issue by respecting the intended feature boundary, which aligns with our memory: "The `tokmd` crate has unguarded usages of `tokmd_git::git_cmd()` that cause compilation to fail when built with `--no-default-features`. Such external tool invocations must be gated with `#[cfg(feature = "git")]`."
