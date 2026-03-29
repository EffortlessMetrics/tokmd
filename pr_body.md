## 💡 Summary
Eliminated `O(N)` string allocations within hot analysis loop reductions by switching `BTreeMap<String, T>` to `BTreeMap<&str, T>` via zero-cost borrows across `tokmd-model`, `tokmd-analysis-content`, and `tokmd-analysis-derived`.

## 🎯 Why (perf bottleneck)
Iterating over thousands (or millions) of discovered file rows forced a `row.lang.clone()` and `row.module.clone()` for every single entry lookup or insertion in tracking collections. This incurred heavy string allocations inside very hot inner loops.

## 📊 Proof (before/after)
- **Structural proof**: `entry(row.lang.clone())` changed to `entry(row.lang.as_str())`. Since the original `file_rows` vector owns the strings and outlives the map generation step, we can borrow the references safely without duplicating string payloads.

## 🧭 Options considered
### Option A (recommended)
- Use `&str` for intermediate map keys, mapping to `String` only during final vector creation (which is bounded by distinct key counts, not file counts).
- Trade-offs: Zero-cost structure at the minor expense of adding a small `by_lang_embedded` in `tokmd-model` since dynamic formats couldn't borrow.

### Option B
- Rely on `Cow<str>`.
- Trade-offs: Overhead for every check if it's owned vs borrowed, whereas pure borrows inside the loop cleanly avoid checking completely.

## ✅ Decision
Option A strictly minimizes allocation cost and relies purely on standard lifetimes without runtime branching.

## 🧱 Changes made (SRP)
- `crates/tokmd-model/src/lib.rs`: Switched `by_lang` to `BTreeMap<&str, ...>`.
- `crates/tokmd-analysis-content/src/content.rs`: Switched `path_to_module` and `module_bytes` to `&str`, removed `.clone()`.
- `crates/tokmd-analysis-derived/src/lib.rs`: Used `&str` for `by_module` and `by_lang` reduction passes.

## 🧪 Verification receipts
- `cargo test -p tokmd-model -p tokmd-analysis-content -p tokmd-analysis-derived` (PASS)
- `cargo xtask gate --check` (PASS)
- `cargo check --all-features` (PASS)
- `cargo bench` correctly compiles

## 🧭 Telemetry
- Risk class: Low (Standard safe Rust borrow checker handles it, no logic mutations).

## 🗂️ .jules updates
- Updated Bolt run envelopes.
