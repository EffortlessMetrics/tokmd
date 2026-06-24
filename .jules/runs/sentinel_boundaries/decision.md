# Sentinel Boundaries Decision

## Option A (recommended)
**Strict Type Boundary Enforcement in FFI**
- **What it is:** Implement an `extract_nested_object` helper in `parse.rs` that explicitly validates that nested JSON configuration properties (like `lang`, `scan`, `module`) are strictly JSON objects (or null/omitted). Replace all uses of `unwrap_or(args)` that silently swallowed validation errors.
- **Why it fits this repo and shard:** The FFI boundary in `tokmd-core` receives untrusted string payloads. The previous implementation silently parsed strings or arrays as fallback root configurations rather than rejecting them, bypassing trust-boundary type validation. This aligns perfectly with the `interfaces` shard and the `Sentinel` persona's mandate to harden trust boundaries and ensure deterministic safety.
- **Trade-offs:**
  - Structure: Centralizes strict parsing, which is much cleaner.
  - Velocity: Tiny overhead on parsing due to explicit validation, but FFI payloads are small.
  - Governance: Improves API safety and prevents silent configuration fallback exploits.

## Option B
**Ignore the silent failover**
- **What it is:** Let the parser continue using `unwrap_or(args)` and leave the FFI payload validation slightly ambiguous.
- **When to choose it instead:** Never, this is a clear boundary violation.
- **Trade-offs:** Reduces parsing code size, but allows malformed untrusted inputs to bypass explicit parsing layers and fall back to the root namespace, creating confusion and potentially insecure API surface behavior.

## Decision
**Option A**. Stricter trust boundaries at the untrusted JSON boundary are a core responsibility for the Sentinel persona, and explicitly checking `is_object()` guarantees predictability.
