# Option A (recommended)
**Use reference-based aggregations in derived reports.**
Many derived reports (e.g., `build_max_file_report`, `build_lang_purity_report`, `build_nesting_report`, `build_polyglot_report`, `build_boilerplate_report`) loop over thousands of `FileStatRow`s or `FileRow`s and allocate `String`s or clone entire `FileStatRow`s repeatedly in the inner loops. By using `BTreeMap<&str, ...>` and storing references during the hot path, we eliminate thousands of unnecessary allocations and `String` clones, converting them to owned versions only for the final, truncated output.
- **Fit:** Perfectly aligns with the `Bolt` persona's mandate to eliminate unnecessary allocations and string building.
- **Trade-offs:**
  - *Structure:* Marginally more lifetime usage inside function bodies, but localized and invisible to the API.
  - *Velocity:* Quick and safe to implement.
  - *Governance:* Preserves exact determinism and external outputs.

# Option B
**Parallelize derived report generation.**
Wrap the calls to `build_max_file_report`, `build_lang_purity_report`, etc. inside a `rayon::join` or `par_iter`.
- **Fit:** Increases throughput, but doesn't solve the underlying allocation inefficiency.
- **Trade-offs:** Increases complexity, threads, and binary size. The sequential overhead is mostly allocation-bound right now anyway, so fixing allocations should come first.

# Decision
Proceeding with **Option A** to directly eliminate the waste at its source rather than just throwing threads at it.
