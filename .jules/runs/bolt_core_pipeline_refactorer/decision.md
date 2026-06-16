### Option A (recommended)

- **What it is:** Update `CycloneDxComponent` logic to use macro formats or `.to_string()` directly if macro not applicable. More specifically, right now, in `crates/tokmd-format/src/export/cyclonedx.rs`, `write_export_cyclonedx_impl` allocates `String`s for property values in every row like this:
  `value: row.code.to_string()`
  And for names it uses `name: "tokmd:code".to_string()`.

Wait, actually, I noticed in several places `.to_string()` and `format!` are being called excessively. Let's look closely at `cyclonedx.rs`. The `name` fields of `CycloneDxProperty` are static strings! They don't need to be allocated as `String`. They could be `Cow<'static, str>` or just `String` but initialized more efficiently, though `CycloneDxProperty` struct requires `String` in `crates/tokmd-format/src/export/cyclonedx.rs`. Wait, we can change `CycloneDxProperty` to use `&'static str` for `name` if we own the struct! `CycloneDxProperty` is a private struct.

Let's look at `CycloneDxProperty`:
```rust
#[derive(Debug, Clone, Serialize)]
struct CycloneDxProperty {
    name: String,
    value: String,
}
```
We can change `name: &'static str`, which avoids allocating 7 strings per file row.

Let's do a quick structural proof/benchmark of this.
