# Pre-normalize Constants Outside Loops

**Context**:
When iterating over thousands of items (e.g., files in a repo) and performing an operation that involves a constant parameter (e.g., stripping a path prefix), repeatedly normalizing or processing that constant inside the loop is wasteful.

**Pattern**:
Identify constant parameters that require processing (e.g., path normalization, regex compilation) and process them *once* before the loop. Pass the processed version to the inner logic.

**Example (Before)**:
```rust
for item in items {
    // Allocates and processes prefix for every item!
    process(item, Some(raw_prefix));
}

fn process(item: &Path, prefix: Option<&Path>) {
    let prefix_norm = prefix.map(|p| normalize(p)); // Expensive
    // ...
}
```

**Example (After)**:
```rust
let prefix_norm = raw_prefix.map(|p| normalize(p)); // Once
for item in items {
    process_optimized(item, prefix_norm.as_deref());
}

fn process_optimized(item: &Path, prefix_norm: Option<&str>) {
    // Fast
}
```

**Evidence**:
In `tokmd-model`, pre-normalizing the strip prefix reduced execution time from ~261ns to ~69ns per call (~3.8x speedup) in the benchmark.

**Prevention**:
- Watch for `Cow::to_string_lossy()` or `replace()` inside loops on variables that don't change per iteration.
- Extract helper functions that take pre-processed data.
