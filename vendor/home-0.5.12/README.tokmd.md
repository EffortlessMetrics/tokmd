# tokmd vendor note for `home` 0.5.12

tokmd patches `home` through the workspace `[patch.crates-io]` entry because
the dependency reaches tokmd through `tokei -> etcetera`, and upstream `home`
0.5.12 does not provide a non-Unix/non-Windows fallback.

The functional local delta is intentionally small:

- `src/lib.rs` adds `#[cfg(not(any(unix, windows)))] home_dir_inner() -> None`.
- `src/windows.rs` keeps the upstream Windows API behavior and adds tokmd audit
  comments around the existing unsafe FFI blocks.

The unsafe Windows code is third-party FFI from `home` 0.5.12. It calls
`SHGetKnownFolderPath`, reads the returned NUL-terminated UTF-16 buffer, and
frees it with `CoTaskMemFree`. Do not make logic changes here casually; refresh
from the upstream crate and reapply the small tokmd fallback if a newer upstream
release removes the need for this vendor patch.
