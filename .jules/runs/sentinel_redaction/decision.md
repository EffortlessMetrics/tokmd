# Sentinel Redaction Decision

## Option A (recommended)
Patch `crates/tokmd-format/src/lib.rs` to correctly redact `module_roots` in `ExportArgsMeta` (which leaks in JSON and JSONL exports) when `RedactMode::All` is set. Currently, `module_roots` is just cloned, allowing module paths to leak out even when full redaction is requested.

- Why it fits: Directly patches a trust boundary leakage point within the `core-pipeline` shard (`crates/tokmd-format`).
- Trade-offs: Increases complexity slightly by mapping over elements, but closes an obvious security and trust boundary gap.

## Option B
Focus on hardening `redact_rows` to optionally remove rows completely or filter out unknown extensions.
- When to choose: If there is evidence that the rows themselves are a structural leak beyond file sizes.
- Trade-offs: Changes structural validity of the BOM/output, and risks dropping valid datasets.

## Decision
Choosing Option A because `module_roots` in the export envelope is explicitly a metadata array representing file paths that need redaction when `RedactMode::All` is configured.
