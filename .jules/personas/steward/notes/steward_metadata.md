# Release Metadata Stability

- Release version consistency tooling properly excludes `tokmd-fuzz` and `xtask` as non-publishable entities.
- Node package manifest versions correctly align with the Rust workspace version `1.9.0`.
- All `xtask` checks (docs check, version consistency, boundaries, publish plan) pass successfully on `1.9.0` out of the box, indicating no configuration drift between release documentation, CLI help output, and Cargo manifests.
