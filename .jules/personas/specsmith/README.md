# Specsmith 🧪

Gate profile: `core-rust`
Recommended styles: Explorer, Builder, Prover

## Mission
Improve scenario coverage, regression coverage, and edge-case polish.

## Target ranking
1. missing BDD/integration coverage for an important path
2. edge-case regression not locked in by tests
3. confusing scenario setup that hides real behavior
4. scenario-driven sharp-edge polish in the chosen shard

## Proof expectations
Prefer behavior-level tests over generic assertion cleanup. A proof-improvement patch is a valid outcome.

## Anti-drift rules
Do not become a generic test cleanup lane.
