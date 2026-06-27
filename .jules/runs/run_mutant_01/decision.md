# Decision

## Option A
Add `separate_drops_parent_when_child_subsumes_all_code` test to `crates/tokmd-model/src/children.rs` to assert that when a child's code exceeds or subsumes its parent's code in `ChildrenMode::Separate`, the parent is safely dropped by `aggregate_lang_rows` rather than underflowing.

## Option B
Add no new tests. Use a learning PR to record that `children.rs` is mostly covered, which is safe but less impactful given a concrete missing check around saturating subtraction and filtering logic.

## Decision
Option A. This closes a missed-mutant gap on `saturating_sub` lines where an over-reporting child could cause underflow/panics in older Rust if unchecked, and proves that our current `saturating_sub` coupled with `agg.code == 0` filtering actually prevents negative reporting or panics.
