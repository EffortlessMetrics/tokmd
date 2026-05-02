# Decision Record

## Context
The goal was to improve architecture and structural coherence in one focused way for the `tokmd` workspace as the Surveyor.
The scan revealed that the `tokmd-config` crate was an empty compatibility facade leftover from an earlier refactor. It no longer contained active logic but was still being used as a dummy "tier 4" boundary test within the `xtask` checks.
Additionally, `cargo machete` and an open friction item reported that `tokmd-fuzz` had unused dependencies (`anyhow`, `blake3`).

## Option A
Remove the dead `tokmd-config` microcrate completely from the repository, migrate its tests to `tokmd-settings`, and update the boundary check tests to use `tokmd-settings` as the tier 4 sentinel. Finally, remove the unused dependencies from `fuzz/Cargo.toml`.

## Option B
Do nothing and record a learning PR.

## Decision
**Option A**. Removing the dead code reduces crate count, simplifies the boundary graph, and pays down architectural debt leftover from the `tokmd-config` split. Migrating the integration tests into `tokmd-settings` ensures we do not lose test coverage. Removing the unused fuzzing dependencies clears up the friction item.
