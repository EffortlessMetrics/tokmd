# Option A
Change `.then(...)` to `.then_with(|| ...)` in `crates/tokmd-model/tests/proptest_deep.rs` when sorting `rows`.

# Option B (recommended)
Do not change `.then(...)` because `cmp` operations are pure and have no side effects, and eager vs lazy evaluation of pure comparisons does not affect determinism. Instead, write a learning PR that we searched the codebase for determinism hazards related to `.then` in sorting, but the existing usages of `.then(...)` are already deterministic due to the purity of the comparison, so no patch is needed. We will record this as a learning outcome and not force a fake fix.

# Decision
Option B. We will document a learning PR instead of faking a determinism fix.
