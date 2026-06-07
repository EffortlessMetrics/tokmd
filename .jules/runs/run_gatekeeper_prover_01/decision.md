# Option A (recommended)

Tighten missing CockpitReceipt contract tests inside `tokmd/tests/cockpit_integration.rs` by adding checks for `test_ratio` in the composition sections of existing tests.

- **Structure**: High. Matches missing coverage.
- **Velocity**: High. Quick fix.
- **Governance**: High. Improves contract determinism per shard focus.

# Option B

Do an extensive overhaul of schema validations, digging into all `crates/tokmd-types/src` schema definitions.

- **Structure**: Medium. Adds deep tests but could violate SRP.
- **Velocity**: Low. Could be slow and break existing assumptions.
- **Governance**: Medium. Adds complexity.

Decision: Option A
