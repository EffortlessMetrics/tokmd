## Decision

**Gap**: The ROADMAP specifies "Add practical examples to command help for analyze, diff, context, gate, cockpit, handoff, run, and export. Keep examples short and tested where practical."
In our review, we found that `analyze`, `context`, `cockpit`, and `handoff` already had `#[command(after_help = "...")]` with examples. But `diff`, `export`, `gate`, and `run` were missing these examples.

### Option A (recommended)
Add `#[command(after_help = "Examples:\n ...")]` to the clap argument structures for the `diff`, `export`, `gate`, and `run` commands in `crates/tokmd/src/cli/parser/*.rs`.

*   **What it is**: Update clap macros in `crates/tokmd/src/cli/parser/{diff,export,gate,run}.rs` to include `after_help` properties with practical examples.
*   **Why it fits this repo and shard**: The tooling-governance shard focuses on tooling, workflows, and docs. This change directly fulfills the "1. CLI help examples" item in `ROADMAP.md` that asks to add examples to `diff`, `export`, `gate`, and `run`.
*   **Trade-offs**:
    *   *Structure*: Follows the exact pattern already established for `analyze`, `context`, `cockpit`, and `handoff`.
    *   *Velocity*: Fast to implement, no new architectural changes.
    *   *Governance*: High alignment with the existing ROADMAP task.

### Option B
Wait for a larger refactor to unify CLI examples using a centralized markdown file or a macro that tests them at build time.

*   **What it is**: Build a system to pull examples from markdown files and inject them into `clap` `after_help` to ensure they stay up-to-date.
*   **When to choose it instead**: If the examples were complex, lengthy, or highly subject to drift without execution, a centralized system might be better.
*   **Trade-offs**: Too heavy for the current goal. The existing commands already use `#[command(after_help = "...")]`. Building a complex tool is out of scope for a one-shot PR and violates the "Do not add another artifact wrapper without a consumer" swarm rule.

## ✅ Decision
Option A. It's direct, aligned with the roadmap task, and mirrors the exact pattern already implemented in the codebase for the other CLI commands listed in the roadmap.
