# Decision

## Options considered
### Option A (recommended)
Update the documentation to match the current CLI schema using `cargo xtask docs --update`, and fix the `cargo xtask gate` failure by updating the excluded run directories.
- Fits the `tooling-governance` shard by updating the documentation and governance tooling.
- Aligned with the Builder style and Librarian persona, which focuses on factual doc quality and executable examples.

### Option B
Only update the documentation, but leave the `cargo xtask gate` failure alone.
- Since this shard covers workspace tooling (`xtask/**`), failing tests in `xtask` should be fixed along with the docs.

## Decision
Proceed with **Option A**. The `docs/reference-cli.md` drifted because it lacked `<!-- HELP: -->` blocks. Adding these and running `cargo xtask docs --update` restores synchronization. The `cargo xtask gate` check was also failing due to missing `".jules/runs"` in its list of runtime states, which we fixed to satisfy `cargo test -p xtask`.
