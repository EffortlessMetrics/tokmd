# Friction Item

id: compat-wasm-pack-args
persona: compat
style: builder
shard: bindings-targets
status: open

## Problem
`wasm-pack test` argument placement is easy to get wrong when future runs need to pass Cargo feature flags or profile flags through to wasm test builds.

## Evidence
- A compat-targets investigation found no repository bug in the wasm, Python, or Node binding matrix.
- The only durable friction was command syntax confusion around where `wasm-pack test` expects feature-related arguments.

## Why it matters
Future compatibility runs should not convert external command-line friction into fake tokmd code changes.

## Done when
- [ ] The bindings-targets runbook documents the exact `wasm-pack test` feature-argument form used by tokmd.
- [ ] Future compat prompts can distinguish external runner syntax friction from a repository feature-interaction bug.
