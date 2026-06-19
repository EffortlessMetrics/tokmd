# Decision

## Option A (recommended)
Add an explicit integration test validating the exact behavior of `tokmd <nonexistent-file.rs>`, proving that `tokmd` defaults to the `lang` subcommand, interprets the bare token as a path, and correctly emits the "Error: Path not found: <bad-path>" global error fallback. We can add this to `crates/tokmd/tests/cli_error_paths_w51.rs` or `crates/tokmd/tests/cli_errors_w66.rs`.

**Why it fits:**
The `interfaces` shard covers the CLI interface. Memory emphasizes that an unrecognized bare string argument (like `tokmd unknown_word`) defaults to the `lang` subcommand and is interpreted as a `PATH` argument, resulting in a "Path not found" error. If that token looks like a bare subcommand (no slash, no dot, no hyphen), the hint machinery kicks in and rewrites it to an unrecognized subcommand error. However, if it's a file path like `missing/file.rs`, it outputs "Path not found: missing/file.rs". Currently, there are unit tests in `src/error_hints.rs` proving the internal rendering behavior, and some e2e tests like `lang_nonexistent_path_fails` (which passes `tokmd /tmp/...`). But we don't have a specific test explicitly passing `tokmd missing/file.rs` (a bare non-existent path that looks like a path, not a bare token).

**Trade-offs:**
- Structure: High alignment with Specsmith (improve edge-case polish and regression coverage).
- Velocity: Quick to implement, solidifying behavior lock.
- Governance: Meets `core-rust` gating.

## Option B
Find missing edge cases around `--help` flags and add tests for them.

**When to choose:**
If we find missing tests for help.

**Trade-offs:**
- Slower, less targeted than Option A.

## Decision
Option A. Adding an explicit BDD-style or integration test that ensures `tokmd missing/file.rs` outputs `Path not found: missing/file.rs` without being rewritten to `Unrecognized subcommand 'missing/file.rs'`.
