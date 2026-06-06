# Rust 1.95 Compatibility Audit

**Date:** 2026-05-08
**Auditor:** Claude (automated)
**Toolchain tested:** `rustc 1.95.0 (59807616e 2026-04-14)`
**Workspace version at audit:** `1.12.0`

## Summary

The tokmd workspace is fully compatible with Rust 1.95.0. All checks passed with
no code changes required for compatibility.

## Checks performed

| Check | Command | Result |
|-------|---------|--------|
| Formatting | `cargo fmt --all -- --check` | ✓ Clean |
| Compilation | `cargo check --workspace --all-features --exclude tokmd-python` | ✓ Clean |
| Clippy | `cargo clippy --workspace --all-targets --all-features --exclude tokmd-python -- -D warnings` | ✓ Clean |
| Quality gate | `cargo xtask gate --check` | ✓ Clean (exit 0) |

## Planned lint measurement

The following planned lints from `policy/clippy-lints.toml` were probed under
`-W <lint>` flags to measure noise before activation:

| Lint | MSRV gate | Violations found |
|------|-----------|-----------------|
| `same_length_and_capacity` | 1.94 | 0 |
| `manual_ilog2` | 1.94 | 0 |
| `decimal_bitwise_operands` | 1.94 | 0 |
| `needless_type_cast` | 1.94 | 0 |
| `manual_checked_ops` | 1.95 | 0 |
| `manual_take` | 1.95 | 0 |
| `manual_pop_if` | 1.95 | 0 |
| `duration_suboptimal_units` | 1.95 | 0 |
| `unnecessary_trailing_comma` | 1.95 | 2 |

The two `unnecessary_trailing_comma` violations are in test-only macros:

- `crates/tokmd-format/tests/analysis_format/deep_analysisfmt_w48.rs:287` —
  trailing comma inside `assert!(..., fmt,)` call
- `crates/tokmd-analysis/src/near_dup/tests/deep2.rs:735` —
  trailing comma inside `assert_eq!(...,)` call

Both are single-character fixes (remove trailing comma before `)`) with no
semantic effect.

## Conclusion

No Rust code changes are required for Rust 1.95 compatibility. The workspace
compiles, checks, and tests cleanly. All planned 1.94/1.95 lint candidates
are zero-violation except `unnecessary_trailing_comma`, which has two trivial
test-file fixes.

The MSRV can be safely raised from `1.93` to `1.95`. The lint ratchets
(excluding `disallowed_fields`) can be activated as part of the same release
cycle with only the two trailing-comma test fixes required.
