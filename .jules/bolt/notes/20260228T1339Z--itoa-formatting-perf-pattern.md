# Avoid String Allocations in Hot Loops

When creating hashes or sorting loops that require string equivalents of numbers, avoid `n.to_string()` or `format!("{}", n)` which allocates on the heap in an inner loop.

Instead, use the `itoa` crate to format into an allocation-free stack buffer:

```rust
let mut buf = itoa::Buffer::new();
let s = buf.format(number);
```

This ensures we don't spam the heap allocator on O(N) paths like hashing file integrity checks.
