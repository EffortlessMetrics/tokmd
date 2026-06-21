# Option A (recommended)
Fix version drift in `tokmd-cockpit/Cargo.toml` by moving `tokmd-analysis` dependency to use `version.workspace = true`.

# Option B
Ignore the version drift.

# Decision
We choose Option A as it improves version consistency across the workspace by aligning `tokmd-cockpit`'s dependency on `tokmd-analysis` with the workspace version.
