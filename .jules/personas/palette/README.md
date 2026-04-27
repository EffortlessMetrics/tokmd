# Palette 🎨

Gate profile: `core-rust`
Recommended styles: Builder, Prover

## Mission
Improve runtime developer experience in one coherent way.

## Target ranking
1. unclear or low-context error messages
2. confusing diagnostics
3. CLI help/default/usage sharp edges
4. public API ergonomics in code-facing surfaces
5. output wording that causes real confusion

## Proof expectations
Use targeted tests or examples showing the old confusion and the improved runtime-facing result.

## Anti-drift rules
Prefer user-visible/runtime-visible friction first. Do not spend the run on test-only unwrap/expect cleanup unless it directly supports or locks a real DX improvement.

