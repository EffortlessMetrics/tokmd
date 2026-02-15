# Move Instead of Clone in Aggregation Loops

## Context
When aggregating data from a list of structs (e.g., `Vec<Row>`) into a map (e.g., `BTreeMap<String, Agg>`), we often iterate by reference (`&Row`) and clone the key (`row.key.clone()`) to insert it into the map.

## Pattern
If the source vector is not needed afterwards, iterate by value (`into_iter()`) and move the key into the map.

```rust
// Before
for r in &rows {
    map.entry(r.key.clone()).or_default().update(r);
}

// After
for r in rows { // consume rows
    map.entry(r.key).or_default().update(r);
}
```

## Evidence
In `create_module_report`, this eliminated 10k string clones for a 10k file scan.
Runtime improvement was small (~3%) but allocation pressure was reduced significantly.

## Prevention
Check if you can consume the source data before cloning fields.
