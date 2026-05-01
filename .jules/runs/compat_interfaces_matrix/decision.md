## Option A: Add #[allow(dead_code)] to `ExportMetaLite` and `ExportBundle` in `crates/tokmd/src/export_bundle.rs`

This is what we've already done! The dead code warnings were surfacing when features were not active, or in certain builds.

**Structure:** Avoids the compilation warnings on restricted environments or limited feature sets.
**Velocity:** The fix is already applied and is very minimal.
**Governance:** Fixes compatibility warnings.

## Option B: Leave it broken.

This leaves warnings in limited configurations.

**Decision:**
Go with Option A. We have already applied the fix and tested it.
