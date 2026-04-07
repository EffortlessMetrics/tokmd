## 🧭 Options considered
### Option A (recommended)
- Extract deterministic fuzzer inputs into `proptest` suites on primary shards (config, scan-args, gate) and fix the broken doclink.
- Achieves proof-improvement for parser/input hardening seamlessly within standard `cargo test`, ensuring tests run on every commit across all platforms, not just during dedicated fuzzing sessions. Fixes immediate technical debt (doclink warning) safely.

### Option B
- Rewrite the `cargo-fuzz` targets directly and attempt to execute them via a nightly container, leaving `proptest` implementation for another persona.
- Extremely slow and brittle CI integration, failing the "proof-improvement" mission by hiding proofs behind specialized tooling.

## ✅ Decision
Option A. Migrating deterministic fuzz checks to `proptest` is vastly superior for repository velocity and CI stability, actively hardening the input parser layer in an accessible manner. The doclink fix is a trivial but necessary inclusion for overall workspace health.
