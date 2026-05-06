# Clippy Policy

tokmd treats Clippy as a governed engineering surface, not as an ad hoc local
preference file. The workspace policy has three goals:

1. keep production code and tests panic-free by default;
2. prevent silent failure patterns such as ignored futures, ignored `Result`s,
   and swallowed error conversions; and
3. make every lint suppression explicit, narrow, and reviewable.

## Active baseline

The active lint baseline lives in the root `Cargo.toml` under
`[workspace.lints.rust]` and `[workspace.lints.clippy]`. Workspace members should inherit that baseline as each crate is cleaned up with:

```toml
[lints]
workspace = true
```

The machine-readable source ledger is `policy/clippy-lints.toml`. It records the
same active lints, their levels, policy class, and rationale, plus planned Rust
1.94 and Rust 1.95 flips that should not be enabled until the MSRV ratchets.

## No test carveouts

The policy is workspace panic-free, not just production panic-free. Do not add
Clippy test carveouts such as:

```toml
allow-unwrap-in-tests = true
allow-expect-in-tests = true
allow-panic-in-tests = true
allow-indexing-slicing-in-tests = true
allow-dbg-in-tests = true
```

Tests should return `Result` when setup or fixture loading can fail, and should
use assertion helpers that preserve useful failure context without unchecked
`unwrap`, `expect`, or `panic!` calls.

## Suppression style

Prefer fixing the code. When a temporary suppression is unavoidable, use a narrow
`#[expect(..., reason = "...")]` at the smallest scope that explains why the
exception is currently correct and what evidence supports it. Do not use blanket
`#[allow]` attributes for policy lints.

Temporary lint debt belongs in `policy/clippy-debt.toml` with a lint, path,
owner, reason, and expiry date. Silent debt is not allowed.

## Repo-local Clippy configuration

`clippy.toml` is reserved for repo-specific Clippy policy knobs such as
disallowed methods, types, or macros. It must not weaken the workspace baseline
or enable test carveouts.

## Allowlist model

Structured allowlists live under `policy/`:

- `policy/no-panic-allowlist.toml` uses semantic path + family + selector
  identity for panic-family exceptions, with advisory `last_seen` line/column
  data only.
- `policy/non-rust-allowlist.toml` records non-Rust programming surfaces with
  owner, reason, surface, classification, and the checks that cover them.

These files are intentionally reviewable policy receipts. Expiring exceptions
should be removed or renewed deliberately.

## Policy gate

Run:

```bash
cargo xtask check-lint-policy
```

The gate verifies MSRV and ledger alignment, root active/planned lint consistency, no test
carveouts, malformed member lint overrides, structured debt fields, and expired
debt. Member inheritance is staged crate-by-crate so policy infrastructure can merge before the cleanup stack flips every crate to blocking Clippy enforcement.
