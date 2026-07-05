# Decision

## Option A (recommended)
Replace all instances of `out.push_str(&format!(...))` with `write!(&mut out, ...)` or `writeln!(&mut out, ...)` where applicable in `crates/tokmd-format`. The benchmark shows that `write!` to an existing string is vastly faster than allocating an intermediate String with `format!()` just to append its slice to the output string. This reduces unnecessary heap allocations across the formatting pipeline without breaking output determinism.

## Option B
Batch strings up into Vecs and then join them at the end. This adds more memory usage and intermediate state and doesn't clearly win over `write!` which avoids temporary String allocations in the first place.

## Decision
Go with Option A. `out.push_str(&format!(...))` is an anti-pattern in Rust that forces a temporary heap allocation for the intermediate String. Using `write!(out, ...)` directly appends to the target string buffer, which fits our "unnecessary allocations / cloning / string building" optimization target.
