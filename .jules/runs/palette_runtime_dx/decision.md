# Decision

## Option A
Implement fuzzy "Did you mean?" suggestions using `strsim` for unrecognized inputs that look like valid subcommands.
- **Trade-offs**: Adds parsing complexity and increases the likelihood of false positives on actual paths.

## Option B (recommended)
Write a learning PR instead, documenting that the explicit subcommand typo hint was already implemented on the target branch (main), eliminating the friction entirely.
- **Why**: `error_hints.rs` already contains logic or the desired outcome is covered. No honest patch can be made.
- **Governance**: Complies with the prompt-to-PR pipeline rule to submit a learning PR when no honest patch is found.

## Decision
Option B. Submitting a learning PR to close out the task truthfully.
