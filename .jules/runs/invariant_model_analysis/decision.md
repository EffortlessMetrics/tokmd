# Decision

## Target
`crates/tokmd-analysis/src/api_surface/tests/properties.rs` tests the invariants of the `api_surface` report generator. However, the current tests do not have a test that specifically checks that the number of `by_module` items or `top_exporters` items is limited, as specified in the tests `crates/tokmd-analysis/src/api_surface/tests/unit.rs`.

There is a property:
```rust
// ---------------------------------------------------------------------------
// Property: by_module is sorted descending by total_items
// ---------------------------------------------------------------------------
```
But there's no limit check for `by_module` capped at 50, and `top_exporters` capped at 20.

Additionally, let's verify if `doc_density` values are properly tested inside `crates/tokmd-analysis/src/derived/tests/properties.rs`. It does have `doc_density_ratio_non_negative`, but does it have an upper bound of `1.0`?

```rust
    #[test]
    fn doc_density_ratio_in_unit_range(rows in arb_file_rows()) {
        let report = derive_report(&export(rows), None);
        prop_assert!(
            report.doc_density.total.ratio >= 0.0 && report.doc_density.total.ratio <= 1.0,
            "doc_density ratio must be in [0, 1], got {}",
            report.doc_density.total.ratio
        );
    }
```

## Options considered
### Option A (recommended)
Add the missing invariant tests in `crates/tokmd-analysis/src/derived/tests/properties.rs`:
- `doc_density_ratio_in_unit_range`: Verifies doc density ratio is <= 1.0.
- `whitespace_ratio_in_unit_range`: Verifies whitespace ratio is <= 1.0.
- `verbosity_rate_positive`: Verifies verbosity rate (bytes per line) is positive.

Add the missing invariant tests in `crates/tokmd-analysis/src/api_surface/tests/properties.rs`:
- `by_module_length_bounded`: Verifies `by_module` length is <= 50.
- `top_exporters_length_bounded`: Verifies `top_exporters` length is <= 20.
- `api_surface_public_items_lte_total`: Verifies `public_items` <= `total_items`.
- `api_surface_internal_items_lte_total`: Verifies `internal_items` <= `total_items`.

### Option B
Only add properties for `derived` crate.

## Decision
Option A. Adding these property tests ensures that invariants that hold in unit tests hold over an extensive randomly generated set of inputs, preventing future drift.
