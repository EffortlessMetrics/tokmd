# Palette: Surface missing policy file errors cleanly

Previously, running `tokmd gate . --policy does_not_exist.json` would silently swallow the file-not-found error and generic fallback text "No policy or ratchet rules specified." Instead of highlighting the user's typo, it felt like the `--policy` flag was ignored entirely.

### Option A: Change `load_policy` to distinguish explicit missing policies
By returning `Result<Option<PolicyConfig>>`, we can propagate errors when an explicit `--policy` path or config-provided policy fails to load, but safely return `Ok(None)` if no policy was requested.

### Option B: Add a pre-flight file check in `gate.rs`
Before calling `load_policy`, verify `args.policy` exists. This solves the immediate `--policy` flag issue but leaves the config-provided policy path silently swallowing errors.

### Decision
Option A was chosen. `load_policy` now correctly handles `.ok()` boundaries, ensuring explicit invalid policy paths halt execution and report helpful errors natively. I also cleaned up an `anyhow` formatting duplication where `GateError::IoError(e)` and its `source` were double-printing `(os error 2)`.

### Verification
- `cargo test -p tokmd` passes.
- `cargo clippy -p tokmd` is clean.
- Manual verification of `tokmd gate . --policy missing.json` correctly shows:
  ```
  Error: Failed to load policy from missing.json: Failed to read policy file: No such file or directory (os error 2)
  ```
