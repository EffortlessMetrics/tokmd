# Decision

## Option A (recommended)
Add mutation tests to `crates/tokmd-model/src/lib.rs` targeting the `avg` function.
- **Why it fits:** The `avg` function handles mathematics and edge-cases (e.g., division by zero) without explicit bounds tests. Adding tests for rounding, integers, and zero covers 9 mutation gaps flagged by cargo-mutants.
- **Trade-offs:**
  - Structure: Minor addition of a test block.
  - Velocity: Quick patch that closes an obvious gap.
  - Governance: Strengthens the `mutation` gate profile in a core component.

## Option B
Add mutation tests to `crates/tokmd-scan/src/math/mod.rs`.
- **When to choose:** If the scan crate had untested logic.
- **Trade-offs:** The mathematical functions in the scan crate already had 100% mutation coverage, making Option A the correct choice.

## Decision
Option A was chosen. I appended targeted tests to `tokmd-model` specifically covering `avg(lines, files)` behavior, filling the assertion gap.
