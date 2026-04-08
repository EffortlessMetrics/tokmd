# Option A: Add `test = false` to `tokmd-python` and remove `xtask` excludes.

This handles the friction where `cargo test --all-features --workspace` fails to link PyO3 `extension-module` targets natively. Removing the excludes improves our lint and check coverage during `cargo xtask gate`.
