# Decision

## Option A
Fix an edge-case regression in TODO counting in `crates/tokmd-analysis`. The `count_tags` API and `count_delimited_tags` were returning unexpected results for overlapping strings and standalone tags, but it appears they were correctly tested but slightly unaligned. Actually, I found a different target: `build_todo_report` was calling `count_delimited_tags` which prevents normal string matching of TODO tags when they are part of a larger string without being explicitly delimited, but other parts of the code used `count_tags`. The fix is simple: change `count_delimited_tags` back to `count_tags` inside `build_todo_report`, which aligns it with the standard `count_tags` behavior used throughout tests. It also adds a BDD test to lock in the delimited-tags edge cases.

## Option B
Focus on `is_text_like` binary detection gaps or hashing improvements. However, this is more suited for the `steward` or `architect` persona. The target `Specsmith` ranking prioritizes edge-case regression not locked in by tests and BDD/integration work around analysis.

## Decision
**Option A**. Replacing `count_delimited_tags` with `count_tags` inside `build_todo_report` fixes an unintended restriction on how TODO tags are counted in the final report, ensuring parity with the test suite. A new test file `bdd_tags.rs` proves the edge case behavior of delimited vs non-delimited tags.
