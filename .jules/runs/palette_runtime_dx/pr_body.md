## 💡 Summary
Implemented a Levenshtein-distance based typo detection for "Path not found" errors in `tokmd`. When a user typos a subcommand (e.g. `anolyze` instead of `analyze`), `clap` natively parses it as a `[PATH]` positional argument instead of returning an unknown subcommand error. This change hooks into `error_hints.rs` to intercept that failed path and suggest the correct subcommand, directly improving CLI ergonomics.

## 🎯 Why
Currently, if a user typos a subcommand (like `anolyze`), they get a generic error:
```
Error: Path not found: anolyze

Hints:
- Verify the input path exists and is readable.
- Use an absolute path to avoid working-directory confusion.
- If this was meant to be a subcommand, it is not recognized. Use `tokmd --help`.
```
This is a sharp edge for runtime developer experience because the generic suggestion forces them to read `--help` or hunt for the typo. A direct "Did you mean..." suggestion immediately unblocks the user.

## 🔎 Evidence
- **Observed Behavior:** `cargo run -- anolyze` gives a generic path error.
- **Receipt:** `cargo run -- anolyze` with the fix applied now gives:
```
Error: Path not found: anolyze

Hints:
- Did you mean the subcommand `analyze`?
- Verify the input path exists and is readable.
- Use an absolute path to avoid working-directory confusion.
```

## 🧭 Options considered
### Option A (recommended)
- Implement a manual Levenshtein distance check in `error_hints.rs`.
- Why it fits: Low blast radius, confined entirely to the module meant for UX improvements. Does not require adding external crates like `strsim` which aligns with strict dependency governance. Avoids fragile hacks to `clap` parsing.

### Option B
- Modify `clap` struct to add hidden aliases for common typos.
- Why to choose it: Only if there are 1-2 very common typos and you want `clap` to handle it natively.
- Trade-offs: Doesn't scale to all typos, pollutes completion logic, requires tracking a hardcoded list of permutations.

## ✅ Decision
Option A was chosen. It provides an immediate and accurate suggestion for typos without altering the core CLI struct or introducing new third-party dependencies.

## 🧱 Changes made (SRP)
- `crates/tokmd/src/error_hints.rs`: Added `levenshtein` distance function and updated `suggestions()` to parse the missing path and check it against known subcommands. Added unit tests verifying typo detection.

## 🧪 Verification receipts
```text
cargo run -- anolyze
cargo test -p tokmd -- test_error_hints
cargo test -p tokmd
```

## 🧭 Telemetry
- Change shape: Feature enhancement
- Blast radius: Output / Formatting (Error formatting only)
- Risk class: Low
- Rollback: Revert the additions in `crates/tokmd/src/error_hints.rs`.
- Gates run: `cargo test -p tokmd`

## 🗂️ .jules artifacts
- `.jules/runs/palette_runtime_dx/envelope.json`
- `.jules/runs/palette_runtime_dx/decision.md`
- `.jules/runs/palette_runtime_dx/receipts.jsonl`
- `.jules/runs/palette_runtime_dx/result.json`
- `.jules/runs/palette_runtime_dx/pr_body.md`

## 🔜 Follow-ups
None.
