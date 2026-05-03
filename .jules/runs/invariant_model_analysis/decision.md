# Decision

## Option A
Add more property-based tests targeting COCOMO constants/calculations in `derive_report` and `compute_maintainability_index` properties. The COCOMO metrics are computed internally as fixed formula based on lines of code. There's missing tests regarding how different models scale, but the current `derive_report` uses fixed organic variables. The `compute_maintainability_index` calculates index from cyclomatic complexity, halstead volume, and lines of code. It already has extensive tests.

Instead, let's look at `derived::mod::derive_report`.
The `top_offenders_bounded_by_ten` property checks that the `top.largest_lines.len() <= 10`. Wait, what if there are fewer than 10 files? It checks `<=` so it's fine. What if there are > 10 files? It guarantees it takes at most 10.
Wait, let's look at `integrity_hash_is_64_hex_chars`. `blake3` hash output string should be 64 characters long.

Let's find properties implied by docs/contracts but not yet tested in `tokmd-analysis`.
What about `crates/tokmd-analysis/src/maintainability/tests/mathematical_invariants.rs`? It tests `compute_maintainability_index`. We can add proptest that `compute_maintainability_index` gives same grade for very small changes, or check continuousness.

Let's look at `crates/tokmd-analysis/src/derived/tests/proptest_w40.rs`. It has `doc_density_between_zero_and_one`, `whitespace_ratio_non_negative`, `cocomo_effort_monotonic`.
Can we tighten `distribution_gini_in_unit_range` to show that if all values are identical, gini is exactly 0.0?
Or that `distribution_mean_between_min_and_max`? We have that.

Another missing invariant: in `tokmd_analysis::derived::derive_report`, `verbosity.total.rate` (bytes per line). Bytes / lines.
Is there an invariant that `bytes >= lines` (for non-empty lines)? Wait, no.
Is there an invariant for `nesting` max >= avg?

Let's add a property-based test that:
- `verbosity.total.rate` is bounded by `min_bytes_per_line` and `max_bytes_per_line`.
- `reading_time` is strictly monotonic with respect to `code` lines.
- `derive_report` produces identical `totals` as the sum of its row lines.

Actually, we already have many `derived` tests. Let's see if we can find something in `crates/tokmd-analysis/src/grid/` or `crates/tokmd-analysis/src/entropy/`.

## Option B
Add property-based tests to `tokmd-analysis/src/grid/tests/properties.rs`.

Let's check `crates/tokmd-analysis/src/entropy/tests/properties.rs`.
