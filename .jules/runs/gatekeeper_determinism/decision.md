# Decision

## Option A (recommended)
Add more `HashMap` usage checks in integration/unit tests to ensure future code does not violate the "no HashMap, use BTreeMap" rule.
This aligns with the `Gatekeeper` persona focus on determinism (BTreeMap prevents non-deterministic outputs which breaks snapshots and golden tests).

## Option B
Update `CONTRIBUTING.md` or `docs/testing.md` to mention BTreeMap usage.

## Decision
I will choose **Option A**. The repo already has some BTreeMap assertions, but adding a specific test to assert that `HashMap` / `HashSet` are not used in `crates/tokmd-types`, `crates/tokmd-scan`, `crates/tokmd-model`, and `crates/tokmd-format` will prevent drift. Wait, there's `xtask boundaries-check` or similar, maybe I should add a check inside `crates/tokmd-types/tests/determinism_props.rs` or `crates/tokmd/tests/determinism.rs` that literally does a string/regex check for HashMap/HashSet on the source tree, similar to linting, or maybe I can rely on a `clippy.toml` restriction. Let me check if there's a `clippy.toml`.
