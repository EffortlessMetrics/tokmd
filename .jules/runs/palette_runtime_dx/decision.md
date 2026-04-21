# Runtime Error Message Styling Improvements

## Option A (Recommended): Add ANSI styling to CLI errors and hints
- **What it is**: Update `tokmd::format_error` and `tokmd/src/bin/tok.rs` (and `tokmd.rs`) to use ANSI colors for the CLI's error outputs. The `Error:` prefix will be styled in bold red and the `Hints:` section header in bold yellow.
- **Why it fits this repo and shard**: The shard focuses on CLI interfaces and runtime developer experience. The default error output (`eprintln!("{}", tokmd::format_error(&err));`) currently prints entirely in plain text, meaning errors don't stand out in crowded terminal output. The repo already relies on the `console` crate (and `anstyle` indirectly) for UI components. By making errors and hints visually distinct, we meaningfully improve the runtime DX.
- **Trade-offs**:
    - *Structure*: Minimal changes confined to `crates/tokmd/src/error_hints.rs` and optionally the bin files.
    - *Velocity*: Fast, easy to implement, safely avoids business logic.
    - *Governance*: Purely visual change, adheres to standard CLI conventions (red for error, yellow for warning/hint).

## Option B: Refactor all `eprintln!` to use `tracing::error!`
- **What it is**: Replace manual `eprintln!` calls with proper structured logging (`tracing::error!`).
- **When to choose it instead**: If the project needs robust diagnostic pipelines, central log aggregation, or programmatic telemetry over terminal output.
- **Trade-offs**: High blast radius requiring widespread modification of all `eprintln!` occurrences across crates. Adds dependencies and configuration requirements not strictly aligned with a simple, focused DX fix per prompt.

## ✅ Decision
**Option A**. It's low risk, explicitly targets the requested "runtime DX" and "CLI error" domain, has a small surface area, and adds clear value by making failure states highly readable without structural overhaul.
