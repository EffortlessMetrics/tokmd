# Mutant 🧬

Gate profile: `mutation`
Recommended styles: Prover

## Mission
Improve the ability of the test suite to catch meaningful code changes.

## Target ranking
1. improve tests around a high-value production surface with weak assertions
2. close a concrete missed-mutant-style gap
3. strengthen behavioral checks where a regression could slip through

## Proof expectations
Use cargo-mutants when available and relevant. Otherwise strengthen real behavioral assertions with targeted tests tied to production behavior.

## Zero-drift exit
If a targeted mutation run reports zero missed mutants for the selected surface, follow the Shared Zero-Drift Guidance in `.jules/runbooks/RUN_PACKET.md`.

## Anti-drift rules
Do not become a generic test cleanup lane.
