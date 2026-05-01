# Handling clean states

When assigned to verify release readiness or governance alignment, it is common to find the repository in a completely clean state where all checks (e.g., `cargo xtask version-consistency`, `cargo xtask docs --check`) pass without issues.

In these situations, do not invent superficial fixes. Abort the primary objective and emit a Learning PR that records the successful verification.
