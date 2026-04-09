# `tokmd` crate fails to compile with `--no-default-features`

The `tokmd` crate contains multiple unguarded usages of `tokmd_git::git_cmd` which cause compilation to fail when the `git` feature is disabled (e.g. `cargo check -p tokmd --no-default-features`).

Errors appear in:
- `crates/tokmd/src/commands/baseline.rs:189`
- `crates/tokmd/src/commands/check_ignore.rs:123`
- `crates/tokmd/src/commands/check_ignore.rs:179`
- `crates/tokmd/src/commands/handoff.rs:278`
- `crates/tokmd/src/commands/handoff.rs:308`
- `crates/tokmd/src/commands/handoff.rs:336`

This needs to be fixed in the `tokmd` crate, which is outside the `bindings-targets` shard. The `tokmd` crate should properly guard `tokmd_git::git_cmd()` with `#[cfg(feature = "git")]` or avoid importing it unconditionally.
