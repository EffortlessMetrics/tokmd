# Decision

## Option A (Fix missing subcommands error)
Improve `tokmd` error handling for path not found when an unrecognised subcommand is provided.
When a user runs an unknown subcommand (e.g. `tokmd foo`), it produces the confusing generic error: "Error: Path not found: foo", suggesting that `foo` was supposed to be a file/directory path to scan.
We can improve this by using clap's error formatting to suggest if the provided path string matches or is close to an existing subcommand, and modify the "Path not found" generic error hint. Alternatively, change clap configuration so it actually catches unknown subcommands.

## Option B (Fix `analyze --explain` missing key error)
The error format produced by `tokmd analyze --explain missing-key` outputs the hint inline with the error message and also adds a Hints section, leading to redundant output.
We can refine `suggestions` in `error_hints.rs` and the original error thrown to produce a single clear hint.

## Chosen path
Option A is stronger as it addresses a core CLI ergonomics issue: `tokmd` interprets any unknown argument as a path due to the `[PATH]...` positional argument.
By default, clap will capture unknown commands into `PATH`, so when users mistype a subcommand (e.g. `tokmd anlyze`), they get "Path not found: anlyze" instead of a helpful "Did you mean 'analyze'?" error.
We can add a check in `tokmd::run` or earlier to explicitly catch when the user provided a single path that strongly resembles a subcommand (e.g. using `strsim`), or simply improve the hints output in `error_hints.rs`. Let's actually use clap's `allow_external_subcommands` or just add a custom check in `tokmd::run` using `strsim` which is already a dependency. Wait, `tokmd` uses clap's positional arguments for paths, so if a user types `tokmd anlyze`, `anlyze` goes into `paths`. We can intercept this either before parsing or after parsing in `commands::dispatch` or `lib.rs`. Let's explore how paths are resolved.
Actually, in `crates/tokmd/src/error_hints.rs`, there's already:
```rust
    if haystack.contains("path not found")
        // ...
        push_hint(
            &mut out,
            "If this was meant to be a subcommand, it is not recognized. Use `tokmd --help`.",
        );
```
But we could make it actively suggest the closest subcommand using `strsim::jaro_winkler` or `strsim::levenshtein` if the missing path is a single word with no slashes.
