# Decision

## Target identification
In `crates/tokmd/src/commands/gate.rs`, the `load_policy` function does this:
```rust
    // Load policy from file, CLI args, or config (may be None if only ratchet is used)
    let policy = load_policy(&args, resolved).ok();
```
Because of `.ok()`, any errors from `load_policy` (like file not found or invalid format) are completely suppressed and turned into `None`.

Then the handler continues, and if neither `policy` nor `ratchet_config` is found, it just prints a generic message: "No policy or ratchet rules specified...".

Wait, what if the user specifically passed `--policy does_not_exist.json`?
`load_policy` actually looks at `args.policy`. If the file is missing, it returns an `Err`. But because of `.ok()`, that error is swallowed! The user gets the generic message instead of "Failed to load policy from does_not_exist.json: No such file or directory".

This is exactly the type of runtime DX issue that fits the Palette persona ("unclear or low-context error messages").
Swallowing `.ok()` on explicit CLI arguments hides the actual error from the user.

Let's check `load_ratchet_config` to see if it does the same.

`load_ratchet_config` correctly uses `?`. `load_baseline` also correctly uses `?`.
Only `load_policy` is hidden behind `.ok()`.

Why is `load_policy` hidden behind `.ok()`?
If a user just runs `tokmd gate .` with no flags, `load_policy` is called. It will look for `--policy`, which is `None`. It will look in config, which might be `None`. It will eventually return `Err("No policy specified")`. If it used `?`, then `tokmd gate . --baseline base.json --ratchet-config rc.toml` would fail with "No policy specified", because it doesn't have a policy (it has ratchet config).

So the logic was trying to say: "Try to load policy. If it fails, that's fine, maybe we only have ratchet rules."
BUT doing `.ok()` swallows *all* errors. Even if the user *did* provide `--policy invalid.json`, the explicit `fs::read` failure would be swallowed, turning it into `None`, and then it might fail later with "No policy or ratchet rules specified" or quietly run just the ratchet rules without the policy!

To fix this properly, `load_policy` should distinguish between "No policy found anywhere" and "I tried to load a specific policy but it was invalid/missing".
Actually, `load_policy` returns `Option<PolicyConfig>`. Let's check `load_policy` implementation.

Ah, `load_policy` returns `Result<PolicyConfig>`, and at the end it does:
```rust
    // No policy found
    bail!("No policy specified")
```

If we change `load_policy` to return `Result<Option<PolicyConfig>>`, we can return `Ok(None)` when no policy is specified, and return explicit errors when parsing/reading fails.
Then in `handle` we can do:
```rust
    let policy = load_policy(&args, resolved)?;
```

This ensures we don't swallow errors like `Failed to load policy from does_not_exist.json: No such file or directory` but still gracefully handle the case where no policy was provided.

Let's test this change.

This fix changes the behavior to explicitly report missing/invalid policy files when the user explicitly requests one, rather than swallowing the error and pretending no policy was passed. This dramatically improves error discoverability (since `does_not_exist.json: No such file or directory` is surfaced instead of "No rules specified").

Wait, let's look at the error output:
```
Error: Failed to load policy from does_not_exist.json: Failed to read policy file: No such file or directory (os error 2): No such file or directory (os error 2)
```
The error has a nested repetition: "Failed to read policy file: No such file or directory (os error 2)". This repetition is usually caused by multiple `with_context` wrappers inside `PolicyConfig::from_file` and here. But the underlying file not found error still gives the path hints properly. Let's see if we can clean up the repetition.

The repetition `Failed to read policy file: No such file or directory (os error 2): No such file or directory (os error 2)` is coming from `anyhow` chaining `IoError` containing the underlying `std::io::Error`.

In `tokmd-gate/src/types.rs`:
```rust
impl std::fmt::Display for GateError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::IoError(e) => write!(f, "Failed to read policy file: {e}"),
            // ...
        }
    }
}
```
And `anyhow::Context` chaining:
```rust
    if let Some(policy_path) = &args.policy {
        let config = PolicyConfig::from_file(policy_path)
            .with_context(|| format!("Failed to load policy from {}", policy_path.display()))?;
        return Ok(Some(config));
    }
```

Because `GateError::IoError(e)` formats its inner error, and `source()` *also* returns `e`, anyhow's `#` formatter prints both!
If we change `Self::IoError(_)` to `write!(f, "Failed to read policy file")` it will be cleaner, or we can just leave it as is. It's not a huge issue. Actually, `error_hints.rs`'s `format` uses `format!("Error: {err:#}")` which will use anyhow's `#` format.

I'll just remove the nested `{e}` from `GateError::Display`.

Let's test if `tokmd gate` missing required arguments correctly gives the clap error.
Wait, `cargo run --bin tokmd gate` without inputs:
```
error: the following required arguments were not provided:
  <INPUT>
```
Wait, earlier `tokmd gate` failed with missing `receipt` argument, but in `w66` the gate signature changed to take positional `[INPUT]`.

Wait! The fix I made for `gate.rs` successfully fixes a bug where `tokmd gate . --policy missing.json` fails silently and just reports "No policy or ratchet rules specified".
Is there any other low hanging fruit?
Let's check `diff`.
`cargo run --bin tokmd -- diff a b` gave a good error.
What if `tokmd analyze --preset receipt does_not_exist`?
Gives: `Error: Path not found: does_not_exist`

What about `baseline.rs`?

The error for `context . --budget abc` is very good.

Let's check if the change I made in `gate.rs` is sufficient for a PR-ready patch for Palette.
The assignment is:
- Target ranking:
1) unclear or low-context error messages
2) confusing diagnostics
3) CLI help/default/usage sharp edges
4) public API ergonomics in code-facing surfaces
5) output wording that causes real confusion

The `tokmd gate` command silently ignoring a missing or malformed explicitly passed policy file is definitely an unclear/low-context error (Target ranking #1).
When I passed `--policy does_not_exist.json`, it did not tell me it couldn't find the file; it gave the generic output "No policy or ratchet rules specified."
This is a huge DX failure if someone misspells their policy filename or has a syntax error in their TOML file.
My fix to `load_policy` correctly propagates the error if an explicit file is requested, and returns `Ok(None)` only if no policy rules were found when gracefully falling back.

I also fixed the nested error formatting in `tokmd-gate/src/types.rs`.
Wait, I haven't fixed the nested error formatting yet. Let me do that now.

Ah, I already patched it via test3.diff! Perfect. Now the error message is clean and perfectly reflects the issue:
```
Error: Failed to load policy from does_not_exist.json: Failed to read policy file: No such file or directory (os error 2)

Hints:
- Verify the input path exists and is readable.
- Use an absolute path to avoid working-directory confusion.
```

This DX fix handles both problems:
1. `tokmd gate` no longer silently ignores explicit policy paths that fail to load.
2. The nested error repetition for GateError::IoError has been cleaned up.

The change touches the primary shard bounds `crates/tokmd/` and `crates/tokmd-gate/` (wait, is `tokmd-gate` allowed?). The allowed paths in envelope are:
- `crates/tokmd-config/**`
- `crates/tokmd-core/**`
- `crates/tokmd/**`

Wait, `crates/tokmd-gate` is NOT in the allowed paths list! I modified `crates/tokmd-gate/src/types.rs`.
If I can't touch `tokmd-gate`, I should revert the nested error fix. The `tokmd-gate` error string isn't a huge deal, but I can just stick to `crates/tokmd/` to stay within the boundaries. Or I can keep it since "Adjacent paths allowed only if a coherent fix requires them". But actually, `crates/tokmd/src/commands/gate.rs` is definitely in `crates/tokmd/**`.

Let's revert the `tokmd-gate` changes just to be safe and perfectly compliant. Actually, `GateError` repetition isn't the primary focus. The main bug is the `.ok()` in `crates/tokmd/src/commands/gate.rs` which swallowed the error entirely.

The error duplication from `anyhow` is minor. The main bug in `crates/tokmd/src/commands/gate.rs` is exactly the target Palette aims to fix. Swallowing the file error entirely resulted in terrible DX ("No policy specified" instead of pointing out the broken path the user typed).

Let's do a quick workspace test check to make sure my fix to `gate.rs` does not break any existing tests that relied on the broken behavior.
