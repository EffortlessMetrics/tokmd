# Decision

## Investigated
- Inspected the current `.jules/friction/open/` directory to see what friction items exist.
- Found multiple friction items:
  - `FRIC-20260413-001.md`: Windows fuzz target MSVC issue
  - `FRIC-20260428-001.md`: Missing nightly toolchain for `cargo fuzz` in sandbox
  - `cargo_fuzz_asan_linker_failure.md`: Linker errors with `cargo fuzz` + ASAN
  - `cargo_mutants_schema_drift.md`: `all_features = true` invalid in `.cargo/mutants.toml`
  - `librarian_doctest_git_dependency.md`: Git dependency in `cockpit_workflow` doctest
  - `surveyor_workspace_learning.md`: Unused dependency in `tokmd-fuzz` and `cargo machete` absence

- The `cargo_mutants_schema_drift.md` states:
  > The `.cargo/mutants.toml` configuration used `all_features = true`, which is invalid in `cargo-mutants` versions v25.0+. This causes the mutation testing tool to fail on launch. The correct configuration uses `additional_cargo_args = ["--all-features"]`.
- Checked `.cargo/mutants.toml` and saw that this change (`additional_cargo_args = ["--all-features"]`) is **already applied** to the file! The friction item is out-of-date and resolved.

- The `cargo_fuzz_asan_linker_failure.md`, `FRIC-20260413-001.md`, and `FRIC-20260428-001.md` are all duplicates or variations of the same friction: Fuzzing in standard environments fails because it requires `nightly` and runs into ASAN linker issues or missing MSVC symbols. These should be consolidated into one documented friction item to clear up the backlog and make future work easier.

## Options Considered

### Option A: Consolidate the fuzz tooling friction items
- **What it is**: Combine the three similar fuzz-related friction items (`FRIC-20260413-001.md`, `FRIC-20260428-001.md`, `cargo_fuzz_asan_linker_failure.md`) into a single, cohesive friction document that tracks the broader issue of `cargo fuzz` environment setup constraints. Remove the redundant files. Clean up the `cargo_mutants_schema_drift.md` file since the issue it describes is already fixed in `.cargo/mutants.toml`. Add these fixed/consolidated items to `.jules/friction/done/` where appropriate, or simply delete them if consolidated into one remaining `open/` item.
- **Why it fits**: Consolidating recurring friction themes is the #1 target ranking for the Archivist persona. It directly improves Jules' ability to understand blocked paths without reading fragmented files.
- **Trade-offs**: Structure (improves clarity) / Velocity (minor clean up effort) / Governance (keeps knowledge base tidy).

### Option B: Summarize per-run packets
- **What it is**: Write a script to continuously aggregate all `.jules/runs/*/result.json` files into `.jules/runs/README.md`.
- **When to choose it instead**: If the friction backlog was already clean.
- **Trade-offs**: Redundant, as I already did this manually via bash, and fixing the duplicated fuzz friction is higher value.

## Decision
**Option A**. I will consolidate the three fuzz friction items into a single, clear `FRIC-fuzz-toolchain-blocker.md` in `.jules/friction/open/`. I will then move the old duplicate files and the fixed `cargo_mutants_schema_drift.md` file into `.jules/friction/done/`. This perfectly aligns with the Archivist's mission to consolidate recurring friction themes and clean up scaffolding.
