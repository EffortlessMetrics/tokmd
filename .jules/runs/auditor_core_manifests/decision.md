## Options considered

### Option A (recommended)
- what it is: Remove the unused `serde` dependency from `crates/tokmd-model`.
- why it fits this repo and shard: It directly targets the primary mission of landing a boring, high-signal dependency hygiene improvement in the core-pipeline shard. `serde` is a direct dependency in `tokmd-model` but isn't used by the actual library code (only in tests where `serde_json` is used).
- trade-offs: Structure / Velocity / Governance. Minimal impact; improves dependency hygiene.

### Option B
- what it is: Remove unused `tokmd-scan`, `tokmd-format`, `tokmd-model` dev-dependencies from `crates/tokmd-types`.
- when to choose it instead: If they were actually unused. However, they are used in `crates/tokmd-types/tests/`.
- trade-offs: Would require migrating tests, which goes against the "boring" mandate.
