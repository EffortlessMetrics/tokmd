# Friction Item

id: librarian_doctest_git_dependency
persona: librarian
style: prover
shard: core-pipeline
status: closed

## Problem
The `cockpit_workflow` public API doctest is marked as `no_run` and skipping validation because it implicitly requires an active Git repository to execute (it fails with `not inside a git repository` when executed normally).

## Evidence
- Surface: Doctests (`crates/tokmd-core/src/lib.rs`)

## Why it matters
This violates the `docs-executable` gate profile expectation. The inability to mock Git state easily without polluting the file system or creating complex temporary fixtures means we cannot test this public API in docs, risking silent drift.

## Done when
- [ ] Provide a `MockGit` trait or abstraction over Git operations that can be swapped in tests.
- [x] Or provide a helper function in tests that safely creates a temporary directory, runs `git init`, creates a dummy commit, and executes the doctest inside that tempdir so that it correctly compiles and runs.

## Closeout

- The `cockpit_workflow` public API doctest now runs against a temporary git
  repository instead of `no_run`.
- The fixture disables local `commit.gpgsign` and `tag.gpgsign` so host-global
  signing configuration cannot block the example's commits.
- Verified with `cargo test -p tokmd-core --features cockpit --doc --verbose`.
