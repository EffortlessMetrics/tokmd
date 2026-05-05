---
id: compat_interfaces_matrix_superseded
persona: Compat
style: Builder
shard: interfaces
status: open
---
# Redundant Fix Superseded by Upstream PR

The intended compatibility patch (suppressing dead code warnings in `export_bundle.rs` under `--no-default-features`) was superseded by PR #1552. That PR used a stricter, structurally preferred `#[cfg(feature="analysis")]` boundary definition instead of the weaker `#[allow(dead_code)]` approach. Following standard procedure, the redundant fix was gracefully aborted in favor of generating a learning PR.
