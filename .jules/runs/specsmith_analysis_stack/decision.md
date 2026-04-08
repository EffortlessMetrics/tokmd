# Decision

## Option A (recommended)
Write targeted integration tests in `crates/tokmd-analysis-content/tests` to kill remaining mutation gaps in `content.rs`:
- Boundary checks for `max_file_bytes`.
- Mathematical operators when calculating duplicated and wasted bytes.
- Mathematical operators when calculating duplication density per module.
- Mutation of multiplication to addition in the limits initialization (`128 * 1024`).

This perfectly aligns with the `Specsmith` persona's goal of improving edge-case coverage and closing mutation gaps without making noisy logic changes.

## Option B
Find generic assertions to clean up or do stylistic test refactors.
This violates the strict directive of not being a generic test cleanup lane.

## Decision
I am proceeding with Option A to create a proof-improvement patch that resolves all caught mutants in `tokmd-analysis-content`.
