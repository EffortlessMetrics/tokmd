# Core Pipeline Clean

As of run `auditor_core_manifests`, the core pipeline crates (`tokmd-types`, `tokmd-scan`, `tokmd-model`, `tokmd-format`) have no unused direct dependencies according to `cargo machete` and manual inspection. Feature flags are tightly scoped.
