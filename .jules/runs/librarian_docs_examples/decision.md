# Option A: Fix `docs/reference-cli.md` global arguments drift

**What it is:**
The `docs/reference-cli.md` has a hand-written "Global Arguments" section. This section claims `These arguments apply when you invoke tokmd directly without an explicit subcommand. They describe the root language-summary surface, not a universal flag set shared by every subcommand.`
However, the actual output of `tokmd --help` includes `--format`, `--top`, and `--files` flags on the default `lang` subcommand, which are entirely missing from this "Global Arguments" table in the docs. Also, `docs/reference-cli.md` is currently missing the `--children` argument that was added to the default command.

I will update the "Global Arguments" section in `docs/reference-cli.md` to accurately reflect the actual flags output by `tokmd --help`.

**Why it fits:**
- Factual docs drift directly matching the "Librarian" mission.
- Easy to verify via `--help`.
- The shard notes include `docs/**` and `reference-cli.md` is definitely in scope.
- We have explicit memory note: "In `tokmd`, `cargo xtask docs --check` only verifies dynamically injected subcommand outputs. Hand-written sections (such as `Global Arguments` in `docs/reference-cli.md`) are not checked and must be validated manually for drift." This perfectly aligns with this finding.

**Trade-offs:**
- Low risk, high value docs alignment. No runtime risk.

# Option B: Add `baseline` / `gate` tests in `xtask`

**What it is:**
While exploring the CLI help text, I could add more doctests/examples for `gate` or `baseline`.

**When to choose it instead:**
If the primary goal was testing coverage instead of pure docs drift.

**Trade-offs:**
- Higher risk of test brittleness.

# Decision

**Option A**. It directly addresses a known drift gap where `xtask docs --check` cannot help, fulfilling the Librarian persona's mission precisely by fixing a missing factual coverage on common usage in reference drift.
