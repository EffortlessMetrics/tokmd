# Exploration & Decision

The assignment asks to improve tests around a high-value production surface with weak assertions, or close a concrete missed-mutant gap in the `core-pipeline` shard (`crates/tokmd-types/**`, `crates/tokmd-scan/**`, `crates/tokmd-model/**`, `crates/tokmd-format/**`).

During exploration, I looked at `crates/tokmd-model/src/module_key/mod.rs` which implements the `module_key` function.
While it has some tests, the original logic for removing leading `./` was:
```rust
if let Some(stripped) = p.strip_prefix("./") {
    p = stripped.to_string();
}
```
This only removed a single leading `./`. If the input path had multiple leading `./` such as `././src/main.rs`, the resulting path would be `./src/main.rs`, and then `trim_start_matches('/')` would leave it as `./src/main.rs`.
Running the `module_key` function on `././src/main.rs` returned `.` because `rsplit_once('/')` splits it at the last slash, extracting `.` as the directory. This is incorrect. The intended behavior, as tested with a while loop, correctly reduces `././src/main.rs` to `src/main.rs`, and then the first directory is correctly identified as `src`.

## Option A (recommended)
Patch `crates/tokmd-model/src/module_key/mod.rs` to use a `while` loop instead of an `if` statement to strip *all* leading `./` prefixes. Add a property test or targeted test to prove that multiple leading `./` sequences are handled correctly without panicking or returning incorrect keys like `.`.

- **Trade-offs:**
  - *Structure:* Simple, localized fix.
  - *Velocity:* High, immediate win.
  - *Governance:* Aligns with the Mutant persona's goal to tighten behavioral checks.

## Option B
Return a learning PR noting that `tokmd-model` tests are already extremely exhaustive and use proptest, and `cargo mutants` is not available.

## Decision
I choose **Option A**. The gap was found by analyzing the path normalization rules in `module_key`. Changing `if let` to `while let` handles repeated `./` segments, which could easily slip through the existing tests since they didn't explicitly test `././` prefixes. The fix is a one-liner and adding tests explicitly covers the behavioral edge case.
