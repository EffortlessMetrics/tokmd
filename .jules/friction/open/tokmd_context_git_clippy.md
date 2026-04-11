# `tokmd-context-git` fails workspace clippy checks

## Description
The `tokmd-context-git` crate fails clippy with an `unnecessary-sort-by` warning in its tests, which breaks `cargo xtask gate --check`.

## Context
When running `cargo clippy --workspace --tests -- -D warnings`, it reports an issue in `crates/tokmd-context-git/tests/deep_w67.rs:112:5`: `sorted.sort_by(|a, b| b.1.cmp(&a.1))` where it suggests `sorted.sort_by_key(|b| std::cmp::Reverse(b.1))`.

## Suggested Fix
Update the code in `crates/tokmd-context-git/tests/deep_w67.rs` to use `sort_by_key` with `std::cmp::Reverse` as suggested by clippy.
