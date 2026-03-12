---
# Friction item

id: FRIC-20231024-001
tags: [bolt, perf]

## Pain
`tokmd-analysis-format` uses `out.push_str(&format!(...))` everywhere. This is extremely wasteful because `format!` allocates a new `String` every time, only for it to be immediately appended to `out` and dropped.

## Evidence
- `crates/tokmd-analysis-format/src/lib.rs`
- > 80 instances of `push_str(&format!(...))`

## Done when
- [ ] Replaced with `writeln!(out, ...).unwrap()` or `write!(out, ...).unwrap()` which writes directly to the existing string buffer without intermediate allocations.
---
