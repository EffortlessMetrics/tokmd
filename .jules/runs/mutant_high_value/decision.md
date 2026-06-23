# Decision

## Option A: Add missing mutant-killing tests for `diff::render` formatters
- **What it is:** The codebase has inline comments like `// Kills mutants in format_delta function` next to test functions in `crates/tokmd-format/src/diff/render.rs` but the tests for other closely related helper functions, like `format_delta_colored`, `format_pct_delta_colored`, and `percent_change`, are completely missing. This leaves behavioral gaps in testing the display formatting logic.
- **Why it fits:** It's exactly the kind of test-suite improvement to catch code regressions that the Mutant persona focuses on. It directly addresses untested logic handling display logic and formatting math in a core component.
- **Trade-offs:** Minimal trade-offs. Adding tests is low risk.

## Option B: Add mutant-killing tests for `analysis::fun_outputs` math
- **What it is:** Add more fine-grained assertions in `crates/tokmd-format/src/analysis/tests.rs` to cover `render_obj_coordinate_math` which claims to catch arithmetic mutants.
- **Why it fits:** Aligns with Mutant's goal, but tests already exist that assert the current math implementations fairly strongly.
- **Trade-offs:** Weaker return on investment than Option A since the `fun_outputs` math already has explicit mutant-killing tests.

## Decision
I'll proceed with **Option A**, adding the missing unit tests for `format_delta_colored`, `format_pct_delta_colored`, and `percent_change` in `crates/tokmd-format/src/diff/render.rs`. This will directly close a concrete assertion gap for production code that currently has zero direct test coverage.
