# Context
We noticed multiple usages of `.unwrap()` in `crates/tokmd/tests/bdd_*_scenarios_w50.rs` and other BDD files (like `bdd_scenarios_w71.rs`, `bdd_scenarios_w75.rs`).

# Pattern
Test files often rely on `.unwrap()` to assert file existence, regex matching, json deserialization, and std output conversions. This reduces developer experience (DX) and makes diagnosing broken tests difficult since error traces are swallowed.

# Evidence
For example, lines like:
`let json: Value = serde_json::from_str(&String::from_utf8(output.stdout).unwrap()).unwrap();`

# Prevention Guidance
Whenever possible, tests should return `anyhow::Result<()>` and use `?` to bubble up I/O and deserialization errors. For situations where `Result` cannot be used, `.expect("descriptive failure message")` must be used over `.unwrap()` to ensure clear panic traces.

# Links
- PR: PENDING
