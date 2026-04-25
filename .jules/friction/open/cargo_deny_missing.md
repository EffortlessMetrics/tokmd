# Friction Item: cargo deny is missing

**Persona**: Auditor
**Impact**: Minor friction

**Description**:
The Gate Profile `deps-hygiene` specifies running `cargo deny --all-features check manifests` as a fallback expectation when dependency/manifest surfaces change. However, `cargo-deny` is not installed by default in the execution environment.

**Observed behavior**:
```text
$ cargo deny --all-features check manifests
error: no such command: `deny`
```

**Recommendation**:
Either install `cargo-deny` in the base environment image, or update the runbooks/memory to note that `cargo deny` should be skipped if not installed, or installed via `cargo install cargo-deny` (though that can be slow).
