# Option A: Remove unused dependency `uuid` from `tokmd-format`
- **What it is**: The `uuid` dependency is currently used in `crates/tokmd-format/src/export/cyclonedx.rs` to generate a random v4 UUID for the `serial_number` field if one is not provided. Since `tokmd-format` already uses `uuid`, we can replace this with `uuid` generation or something more deterministic if the specs allow it. Wait, actually, let's examine if `uuid` is required. The `uuid` crate brings in `getrandom` and `libc`. Let's see if we can use a deterministic UUID or if we can use another hash like `blake3` which is already in the tree. Or, wait! Let's check `midly`. `midly` is behind a `fun` feature in `tokmd-format`.
- Let's check unused dependencies.
- The `uuid` dependency was successfully removed from `tokmd-format`! We replaced it with a custom deterministic UUID generator logic.
- We need to double-check if `uuid` is still used in `tokmd` or other crates. Let's see the `cargo tree` of the workspace.
