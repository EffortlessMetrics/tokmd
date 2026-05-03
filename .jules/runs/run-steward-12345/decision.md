# Option A: Remove stale `RUSTSEC-2023-0071` from `deny.toml`
The cargo-deny execution produced a warning `warning[advisory-not-detected]: advisory was not encountered` for `RUSTSEC-2023-0071`. This means the vulnerable crate has been updated or removed from the dependency tree, so we can clean up the configuration file.
This fits the Steward persona as it's a metadata hygiene improvement.

# Option B: Add `--profile` support to `xtask gate`
The prompt executes `cargo xtask gate --profile governance-release`, which currently fails because `--profile` is not a valid argument for `GateArgs`. We could add the argument and implement profile-based skipping of checks.
This is a bit more involved and might not be considered "low-risk release-surface fix".

# Decision
We will proceed with Option A because it's a verifiable, low-risk, high-confidence metadata cleanup aligned with the Steward persona.
