# Friction Item

id: compat-wasm-pack-args
persona: compat
style: builder
shard: bindings-targets
status: closed

## Problem
`wasm-pack test` argument placement is easy to get wrong when future runs need
to pass Cargo feature flags or profile flags through to wasm test builds.

## Evidence
- A compat-targets investigation found no repository bug in the wasm, Python,
  or Node binding matrix.
- The only durable friction was command syntax confusion around where
  `wasm-pack test` expects feature-related arguments.

## Why it matters
Future compatibility runs should not convert external command-line friction
into fake tokmd code changes.

## Done when
- [x] The bindings-targets runbook documents the exact `wasm-pack test`
  feature-argument form used by tokmd.
- [x] Future compat prompts can distinguish external runner syntax friction
  from a repository feature-interaction bug.

## Closeout

- `.jules/personas/compat/README.md` now records the exact tokmd wasm forms:
  `wasm-pack test --node crates/tokmd-wasm --no-default-features` and
  `wasm-pack test --node crates/tokmd-wasm --features analysis`.
- The runbook explicitly treats misplaced feature/profile flags as external
  runner syntax friction, not as a tokmd compatibility bug.
