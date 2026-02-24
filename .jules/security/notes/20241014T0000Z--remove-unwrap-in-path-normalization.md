# Remove unwrap() in Path Normalization

## Pattern
Replace `unwrap()` calls on `strip_prefix()` with `while let Some(...)` or similar safe handling, especially when iterating on string slices.

## Context
Path normalization loops often strip prefixes (e.g., `./`). Using `unwrap()` assumes `starts_with` check is perfectly synchronized, but it introduces panic potential and often unnecessary allocations (calling `to_string()` repeatedly).

## Solution
Use `while let Some(stripped) = s.strip_prefix(...)` to safely iterate on the slice without allocation or unwrap.

## Example
```rust
// Before
while s.starts_with("./") {
    s = s.strip_prefix("./").unwrap().to_string();
}

// After
while let Some(stripped) = s.strip_prefix("./") {
    s = stripped;
}
```
