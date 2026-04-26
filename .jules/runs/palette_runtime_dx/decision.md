## Options

**Problem:** `tokmd` produces confusing error hints when a user provides a path that does not exist. The hint generically says "If this was meant to be a subcommand, it is not recognized", which is confusing since they may have already provided a valid subcommand and the nonexistent path is just an argument.

**Target Ranking:** 1) unclear or low-context error messages

### Option A (recommended)
Update the error heuristic in `crates/tokmd/src/error_hints.rs` to include the actual `bad_path` in the hint, e.g., `"If '{bad_path}' was intended as a subcommand..."` and to only emit this subcommand hint if the path looks vaguely like a subcommand (e.g. no slashes or dots).
This gives the user much more context about what token `tokmd` was evaluating when it guessed it might be a typoed subcommand.

- Fits the repo and shard: Touches the exact UX diagnostic pipeline in `tokmd/src/error_hints.rs`.
- Trade-offs: Requires a small state variable to propagate the extracted path to the hint emitter, but safely falls back if parsing fails.

### Option B
Try to plumb whether a subcommand was actually recognized down to the error formatter so it never emits subcommand hints for positional arguments.
- When to choose: If the CLI parser state was readily available in the error type.
- Trade-offs: Too invasive. The error formatter takes an `anyhow::Error` and the CLI parser doesn't inject clap context into path-not-found I/O errors.

## ✅ Decision
Option A. It's a localized, high-value DX improvement that directly addresses low-context diagnostics without altering error signatures across the crate.
