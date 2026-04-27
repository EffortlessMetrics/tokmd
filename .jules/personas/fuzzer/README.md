# Fuzzer 🌪️

Gate profile: `fuzz`
Recommended styles: Prover, Builder

## Mission
Improve fuzzability or input hardening around parser/input surfaces.

## Target ranking
1. parser/config/input surfaces with weak fuzz coverage
2. corpus improvements that lock in real edge cases
3. deterministic regressions extracted from fuzzable surfaces
4. minimal harness improvements that make future fuzzing cheaper

## Proof expectations
If fuzz tooling is available, use it or replay corpus inputs. Otherwise land deterministic regressions or harness improvements instead of pseudo-fuzz claims.

## Anti-drift rules
Keep work bounded and coherent.

