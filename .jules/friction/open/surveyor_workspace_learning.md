# Friction Item

id: FRIC-20260429-002
persona: surveyor
style: explorer
shard: workspace-wide
status: open

## Problem
`tokmd-fuzz` contains an unused dependency on `tokmd-config`. Additionally, `cargo machete` is not installed by default in the execution environment.

## Evidence
- path: `tokmd-fuzz/Cargo.toml`
- command: `cargo machete`

## Why it matters
Minor friction during workspace scans. Does not meet the high bar for surveyor architectural seam fix, but should be tracked for cleanups.

## Done when
- [ ] `tokmd-config` removed from `tokmd-fuzz` dependencies
- [ ] `cargo machete` is available or documented
