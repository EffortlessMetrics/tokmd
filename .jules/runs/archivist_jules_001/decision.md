# Option A: Delete superseded fuzzer friction items

The friction index shows three items that have been "superseded by" `fuzz_toolchain_blocker`.

- `FRIC-20260428-001`
- `FRIC-20260413-001`
- `cargo_fuzz_asan_linker_failure`

Since they are superseded, their presence is redundant and pollutes the friction index and `.jules/friction/done/` folder. This option cleans up these files, reducing duplication in the index and the folder.

Trade-offs: Structure: High (removes duplication). Velocity: High (quick to do). Governance: High (ensures the single source of truth for the issue).

# Option B: Add a section to fuzzer README

Update `.jules/personas/fuzzer/README.md` to directly reference the findings in `fuzz_toolchain_blocker`, bringing the knowledge closer to the persona using it.

Trade-offs: Low (duplicates info, might go out of date).

# Decision: Option A

I will proceed with Option A to clean up the workspace and generate a new index using `cargo xtask jules-index`. This is the exact type of consolidation work an Archivist is expected to do.
