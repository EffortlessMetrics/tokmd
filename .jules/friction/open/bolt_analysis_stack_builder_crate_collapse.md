# Friction Item: Crate Collapse
The `tokmd-analysis-topics` crate (and likely other analysis support crates) was collapsed into a different structure on `main`. Patches targeting `crates/tokmd-analysis-topics/src/lib.rs` will fail to merge or rebase.
The string allocation optimization (`get_mut` vs `.entry().or_insert()`) for term aggregation remains a valid performance target and should be ported to the new layout.
