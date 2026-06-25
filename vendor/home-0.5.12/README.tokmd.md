# tokmd vendor note for `home` 0.5.12

tokmd patches `home` through the workspace `[patch.crates-io]` entry because
the dependency reaches tokmd through `tokei -> etcetera`, and upstream crates.io
`home` 0.5.12 does not compile on non-Unix/non-Windows targets.

## Local delta

The functional change is intentionally small:

- `src/lib.rs` adds `#[cfg(not(any(unix, windows)))] home_dir_inner() -> None`.
- `src/windows.rs` keeps the upstream Windows API behavior and adds tokmd audit
  comments around the existing unsafe FFI blocks.

The unsafe Windows code is third-party FFI from `home` 0.5.12. It calls
`SHGetKnownFolderPath`, reads the returned NUL-terminated UTF-16 buffer, and
frees it with `CoTaskMemFree`. Do not make logic changes here casually; refresh
from the upstream crate and reapply the small tokmd fallback if a newer upstream
release removes the need for this vendor patch.

## Upstream tracking

- [rust-lang/cargo#12297](https://github.com/rust-lang/cargo/issues/12297) —
  wasm/non-Unix compile failure; closed *not planned* (Cargo team: `home` is for
  Cargo/rustup internal use only).
- [rust-lang/libs-team#372](https://github.com/rust-lang/libs-team/issues/372) —
  open ACP for a possible `std::env::home_dir` replacement; unrelated to dropping
  this patch unless `tokei`/`etcetera` stop using `home`.
- Upstream source: [rust-lang/cargo home crate](https://github.com/rust-lang/cargo/tree/master/crates/home).

Normative maintenance contract:
[`docs/specs/dependency-maintenance.md`](../../docs/specs/dependency-maintenance.md)
(§ Vendored `home` patch).

## Removal criteria

Delete this vendor directory and the root `Cargo.toml` `[patch.crates-io]` entry when:

1. crates.io `home` (at the version resolved by `tokei -> etcetera`) compiles on
   non-Unix/non-Windows targets with a `home_dir_inner()` that returns `None`.
2. `cargo tree -i home --edges normal` shows no workspace patch.
3. `ci/proof.toml` scope `vendored_home_patch` passes without the vendor tree.

Until then, refresh this copy from upstream `home` 0.5.x only when security or
toolchain fixes require it, and reapply the `home_dir_inner` fallback.
