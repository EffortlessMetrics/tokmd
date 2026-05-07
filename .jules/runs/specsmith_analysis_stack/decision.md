# Decision

## Option A
Fix `count_tags` in `crates/tokmd-analysis/src/content/io.rs` to correctly use word boundaries instead of simple `.matches()` counting. Simple substring matching can falsely match tags within words (like `todo_app`).

## Option B
Add extensive regex parsing for tags to support more robust scenarios.

## Chosen Option
Option A is chosen as it provides an immediate fix without bringing in heavy dependencies or a significant performance penalty like full regex while still passing existing and new scenario tests.
