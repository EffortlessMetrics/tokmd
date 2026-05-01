---
id: bolt_core_pipeline_perf_walls
persona: Bolt
style: Refactorer
shard: core-pipeline
status: open
---

# Friction Item: Core Pipeline Performance Walls

When attempting to optimize hot paths in `crates/tokmd-model` (specifically `normalize_path`), it was found that the existing string manipulation logic is already relatively well-optimized. Attempts to simplify the prefix-stripping logic or use byte slice searches (`.as_bytes().contains(&b'\\')`) resulted in minimal, noise-level performance gains (< 2% in benchmarks) and risked introducing double-allocations or behavioral regressions in trailing slash normalization.

A coherent, single-story performance improvement patch could not be honestly justified without hallucinating metrics. The `core-pipeline` shard may require broader structural refactoring (e.g., using a global path interner or `&str` reference tracking) to achieve significant performance wins, rather than localized string micro-optimizations.
