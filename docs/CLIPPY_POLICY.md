# Clippy policy

Tokmd treats Clippy as a governed engineering surface, not as local taste in
individual crate manifests. The root workspace owns the active lint baseline,
and every workspace member inherits it with `[lints] workspace = true`.

## Policy goals

The active policy is intentionally workspace-wide:

- panic-family shortcuts are denied in production code and tests;
- futures, locks, results, and errors must not be silently discarded;
- string, slice, AST, path, process, numeric, async, and unsafe footguns are
  either denied or raised as review warnings;
- suppressions must be narrow and justified;
- planned Rust 1.94 and 1.95 lint flips are tracked before the MSRV ratchet.

Tokmd currently tracks the repository MSRV in `policy/clippy-lints.toml`; the
policy gate requires it to match `workspace.package.rust-version`.

## Files

- `Cargo.toml` contains `[workspace.lints.rust]` and `[workspace.lints.clippy]`,
  the active lint block inherited by member crates.
- `clippy.toml` is reserved for repo-specific disallowed methods, types, and
  macros. It must not contain test carveouts such as `allow-unwrap-in-tests`.
- `policy/clippy-lints.toml` is the machine-readable active and planned lint
  ledger.
- `policy/clippy-debt.toml` records temporary repo-local exceptions. Debt must
  have an owner, reason, path, lint, and expiry.
- `policy/no-panic-allowlist.toml` is the semantic policy receipt for any
  reviewed panic-family exception.
- `policy/non-rust-allowlist.toml` is the structured receipt for intentional
  non-Rust files and surfaces.

## Suppression style

Prefer fixing code over suppressing lints. When a suppression is unavoidable,
use `#[expect(..., reason = "...")]` at the narrowest scope possible. Do not add
blanket `#[allow]` attributes or broad Clippy category suppressions.

## Test posture

Tokmd does not use test carveouts. Tests should return `Result` and use `?` for
fallible setup instead of `unwrap`, `expect`, or panic-driven fixture handling.
