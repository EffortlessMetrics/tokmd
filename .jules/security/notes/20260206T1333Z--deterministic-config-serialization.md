# Deterministic Config Serialization

**Context**: Found `HashMap` usage in `TomlConfig` (`crates/tokmd-config/src/lib.rs`) during a security scout run. This caused non-deterministic key ordering when the struct was serialized (e.g., to JSON or TOML), which violates the project's requirement for deterministic outputs.

**Pattern**: `std::collections::HashMap` does not guarantee iteration order. When used in structs that are serialized (via Serde) for user output or state persistence, it leads to flaky tests and unstable file contents (diff noise).

**Prevention**:
- Prefer `std::collections::BTreeMap` for all map fields in configuration and receipt structs.
- `BTreeMap` sorts keys alphabetically, ensuring stable serialization.
- Use `crates/tokmd-config/tests/determinism.rs` to verify ordering.

**Links**:
- [Rust HashMap docs](https://doc.rust-lang.org/std/collections/struct.HashMap.html)
- [Rust BTreeMap docs](https://doc.rust-lang.org/std/collections/struct.BTreeMap.html)
