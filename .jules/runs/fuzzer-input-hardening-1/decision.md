# Decision

## Option A
Add fuzzing constraints / targets around CLI argument parsing `resolve_export`, `resolve_module`, or `resolve_lang` which currently lack deep validation constraints during config fallback operations. Alternatively, fix existing unhandled panic edge-cases in TOML loading or path normalization. Since the fuzz targets are already doing a solid job testing the invariants of parsing, I will focus on adding a determinism test/property test for the cli module `resolve_lang`, `resolve_export`, and `resolve_module` falling back to their respective profile default values vs overriding.

## Option B
Enhance the existing fuzzers with `cargo-mutants` gaps for `tokmd_format::redact`.

## Decision
I'll go with Option A and strengthen the test coverage in `crates/tokmd/tests/config_resolution.rs` via properties or deterministic assertions around `resolve_module` and `resolve_export` default handling behavior. This is directly related to parser/config/input surfaces with weak fuzz coverage (as integration assertions) that map back to deterministic regressions from config-driven execution.
