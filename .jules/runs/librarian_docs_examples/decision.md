# Decision

## Option A: Fix GitHub Action Version Drift
The `Cargo.toml` and most docs specify version `1.11.0`, but `README.md`'s GitHub Action example is using `version: '1.10.0'`. I will fix this version drift. This satisfies the target ranking "README/example drift from actual behavior".

## Option B: Fix the missing check-docs command
It looks like `cargo xtask check-docs` is missing, but actually it is `cargo xtask docs --check`. This is not really a docs drift.

## Decision
Option A. It's a very clear fix to `README.md` to ensure the version snippet points to `1.11.0`, consistent with `docs/github-action.md` and `Cargo.toml` (`1.11.0`).

