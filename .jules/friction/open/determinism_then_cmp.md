# Friction Item: Determinism hazard with `.then()` vs `.then_with()`

We investigated replacing `.then(a.cmp(b))` with `.then_with(|| a.cmp(b))` in `sort_by` BTreeMap closures, driven by a memory rule. However, because `.cmp()` has no side effects and is pure, eager vs lazy evaluation does not change the determinism of the output.

We should update the guidance/memory to avoid suggesting that `.then(...)` introduces flakiness for pure comparisons, to prevent faking determinism fixes.
