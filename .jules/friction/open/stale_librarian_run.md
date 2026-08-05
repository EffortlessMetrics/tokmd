# Friction Item: Stale Librarian Run
This branch (`jules/librarian-docs-debug-fix`) successfully fixed CLI flag drift in `docs/debugging.md`, but it was closed as stale because it predates the 1.15.0 release and is no longer an active priority.

The changes made:
- Updated `cargo run -p tokmd -- run --path . --out target/tokmd-debug` to `cargo run -p tokmd -- run . --output-dir target/tokmd-debug`
- Updated `cargo run -p tokmd -- analyze --path . --format json` to `cargo run -p tokmd -- analyze . --format json`
- Updated `cargo run -p tokmd -- cockpit --base origin/main --head HEAD --out target/cockpit-debug` to `cargo run -p tokmd -- cockpit --base origin/main --head HEAD --artifacts-dir target/cockpit-debug`

If these documentation errors persist on `main`, they can be salvaged from this branch history.
