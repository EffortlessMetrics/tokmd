# PR Glass Cockpit

Make review boring. Make truth cheap.

## 💡 Summary
Improved the `tokmd init` command to provide friendlier feedback. It now explicitly states which template was used (e.g., 'default', 'rust') and provides a helpful hint about available templates if the default is used implicitly.

## 🎯 Why (user/dev pain)
Previously, `tokmd init` (especially in non-interactive mode or when defaults are used) just said "Wrote ./.tokeignore". Users didn't know:
1. That other templates (like `rust`, `node`, `python`) exist.
2. Which template was actually applied.
3. What to do next.

## 🔎 Evidence (before/after)

**Before:**
```bash
$ tokmd init
Wrote ./.tokeignore
```

**After:**
```bash
$ tokmd init
Initialized ./.tokeignore using 'default' template.
Hint: Use --template <NAME> for specific defaults (rust, node, python...).
Ready! Run 'tokmd' to scan your code.
```

## 🧭 Options considered
### Option A (recommended)
- **Modify `init_tokeignore` return type:** Change the library function to return the created path instead of printing directly. Move printing logic to the CLI command handler.
- **Why:** Keeps the library pure(r) and allows the CLI to provide rich, context-aware user feedback.
- **Trade-offs:** Small breaking change for the internal `tokmd-tokeignore` crate API.

### Option B
- **Just add more prints in the library:**
- **Why:** Simpler change.
- **Trade-offs:** Harder to control output formatting from the CLI layer; library becomes "chatty".

## ✅ Decision
**Option A**. Better separation of concerns and allows for a more polished CLI experience.

## 🧱 Changes made (SRP)
- `crates/tokmd-tokeignore/src/lib.rs`: Changed `init_tokeignore` to return `Result<Option<PathBuf>>` and removed internal printing.
- `crates/tokmd/src/commands/init.rs`: Updated to handle the result, print the success message, and conditionally show hints.
- `docs/reference-cli.md`: Updated via `cargo xtask docs --update` to reflect CLI drift.

## 🧪 Verification receipts
Copied from run envelope:
```json
    {
      "cmd": "tokmd init --force",
      "exit_status": 0,
      "output": "Initialized ./.tokeignore using 'default' template.\nHint: Use --template <NAME> for specific defaults (rust, node, python...).\nReady! Run 'tokmd' to scan your code."
    },
    {
      "cmd": "tokmd init --template rust --force",
      "exit_status": 0,
      "output": "Initialized ./.tokeignore using 'rust' template.\nReady! Run 'tokmd' to scan your code."
    },
    {
      "cmd": "tokmd init --print",
      "exit_status": 0,
      "output": "# .tokeignore (starts with content...)"
    }
```

## 🧭 Telemetry
- **Change shape:** CLI output text change + internal API refactor.
- **Blast radius:** Low. Only affects `tokmd init`.
- **Risk class:** Low.
- **Merge-confidence gates:**
    - `cargo build`: PASS
    - `cargo test`: PASS
    - `cargo fmt`: PASS
    - `cargo clippy`: PASS
    - `cargo xtask docs --check`: PASS (after update)

## 🗂️ .jules updates
- Updated `palette/ledger.json` with run details.
- Created `palette/envelopes/20260227-132125.json`.
