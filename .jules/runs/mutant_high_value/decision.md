## Problem
The `clean_path` function in `tokmd-format::redact::mod.rs` normalizes separators and resolves `.` segments, but it does not resolve parent directory segments (`..`). This allows logical path bypasses like `src/../src/main.rs` to produce a different hash than `src/main.rs`, violating the determinism guarantee and leaking directory structure info.

## Options considered

### Option A (recommended)
Update `clean_path` in `crates/tokmd-format/src/redact/mod.rs` to fully resolve `..` segments using a stack-based approach after normalizing separators. Add tests to prove identical hashes.
- **Structure:** Keeps all path normalization logic contained in `clean_path`.
- **Velocity:** Simple string manipulations without touching `std::path::Path`, which is OS-specific and might behave differently on Windows vs Unix.
- **Governance:** Ensures determinism and prevents structure leakage, aligning with `contracts-determinism` profile fallback expectations and the Mutant persona.

### Option B
Use `std::path::Path::components()` and `Component::ParentDir`.
- **Structure:** Rely on standard library.
- **Velocity:** `Path::components()` can be complex when crossing platforms (e.g., handling `Prefix` on Windows).
- **Governance:** Higher risk of subtle platform differences because `tokmd` promises cross-platform path hash determinism.

## Decision
Option A. It's safe, cross-platform deterministic (treating `\` as `/` consistently), and closes the logical bypass gap.
