# Surveyor Decision

## 🧭 Options considered

### Option A: Clean up workspace dependency versions (Recommended)
- **What it is**: The root `Cargo.toml` centralizes workspace dependencies with `{ path = "crates/...", version = "1.14.0" }`. However, internal crates have hardcoded versions instead of using `.workspace = true`, like `tokmd-model = { path = "../tokmd-model", version = ">=1.9, <2" }`. Replace these hardcoded constraints with `.workspace = true` to unify versions through the workspace.
- **Why it fits this repo and shard**: Surveyor focuses on workspace structure and dependency direction. Centralized dependency management prevents drift and is an explicit goal of the workspace root metadata. This is exactly what `workspace = true` is designed for. The issue of version consistency and drift in internal crates is mentioned in the memory ("cargo xtask version-consistency only validates the [workspace.dependencies] table ... It does not detect version drift in inline dependency declarations inside subcrate Cargo.toml files").
- **Trade-offs**: Structure improves by centralizing dependencies. Velocity improves by reducing places to bump versions. Governance improves through single-source-of-truth. Risk is low since paths ensure local resolution during development anyway.

### Option B: Restructure the `tokmd-cockpit` and `tokmd-analysis` dependency boundary
- **What it is**: Find structural refactoring points across `tokmd-cockpit` and `tokmd-analysis` to separate concerns better.
- **When to choose it instead**: If the current dependency graph shows true architectural leakage rather than simple metadata debt.
- **Trade-offs**: Much larger blast radius. Changing the boundary might be useful, but fixing workspace dependency declaration is more objective, more verifiable, and has less risk of breaking the build.

## ✅ Decision
I will pursue Option A. It addresses a clear workspace structure and version-management problem where sub-crates hardcode cross-workspace dependencies instead of inheriting them from `[workspace.dependencies]`. This perfectly fits Surveyor's goal of improving workspace structure and addressing dependency issues without generic cleanup.
