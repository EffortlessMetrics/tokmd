# Decision

## Focus
Extract inline deterministic sorting logic for `LangRow`, `ModuleRow`, and `FileRow` into public standalone functions, and refactor tests to use them to eliminate duplicate closures and guarantee shared invariants.

## Options Considered

### Option A: Extract standalone sorting functions
- **What it is:** Create `pub fn sort_lang_rows`, `pub fn sort_module_rows`, and `pub fn sort_file_rows` in `crates/tokmd-model/src/lib.rs` and update all inline usages in main and test code to call these functions.
- **Why it fits:** Reduces duplicate inline sorting closures. Guarantees that model tests assert on the exact same sorting logic as the production pipeline.
- **Trade-offs:**
  - **Structure:** Better encapsulation.
  - **Velocity:** Low risk, straightforward refactor.
  - **Governance:** Tightens determinism guarantees across test and production boundaries.

### Option B: Improve snapshot tests for deterministic output
- **What it is:** Generate new determinism-focused snapshot scenarios for large row sets.
- **Why it fits:** It is an alternative way to prove determinism without refactoring code.
- **Trade-offs:** Does not fix the underlying structural gap that tests redefine the sorting closures manually.

## Selection
**Option A** is selected. It's the most effective way to eliminate duplicated determinism definitions ("sharp edges") between tests and production logic.
