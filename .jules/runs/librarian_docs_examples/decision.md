## Problem
The `docs/handoff.md` and `docs/reference-cli.md` documentation has missing example test coverage for the `tokmd handoff --no-git` example. Also, the `tokmd handoff --budget 128k --strategy spread` command is covered, but I should verify the parameters in `docs/handoff.md` and `docs/reference-cli.md`.
Wait, there is missing test coverage for `tokmd handoff --no-git` and `tokmd handoff --budget 64k --strategy spread` in `crates/tokmd/tests/docs.rs`. Also the example in `docs/handoff.md` uses `128k` but `docs/reference-cli.md` uses `64k`. I should make them consistent and add a test for it.

## Option A
Make `docs/handoff.md` and `docs/reference-cli.md` consistent by updating the budget parameter in one of them, and add executable tests in `crates/tokmd/tests/docs.rs` for `tokmd handoff --budget <X> --strategy spread` and `tokmd handoff --no-git`. This guarantees the documentation examples will not silently drift.

## Option B
Just fix the consistency between `docs/handoff.md` and `docs/reference-cli.md` and don't add tests.

## Decision
Option A. This adheres to the gate profile expectation "Docs and examples must execute or compile where possible" and the `docs-executable` gate profile.
