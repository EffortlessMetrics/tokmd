# Avoid push_str with format!

## Context
When appending formatted strings in a loop or hot path, intermediate allocations are wasteful.

## Pattern
Avoid: `out.push_str(&format!("hello {}", name))`
Prefer: `write!(out, "hello {}", name).unwrap()` or `writeln!(out, "hello {}", name).unwrap()`

## Evidence
`tokmd-analysis-format` was modified to eliminate >80 allocations of this kind, reducing allocator pressure significantly.

## Prevention
Use Clippy lints and review guidelines to spot `push_str(&format!(...))` patterns. Add local `use std::fmt::Write;` inside functions to avoid global trait ambiguities.
