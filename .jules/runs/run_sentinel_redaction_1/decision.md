## Target: Redaction Leakage Prevention

**Problem**: The CycloneDX SBOM format output in `crates/tokmd-format/src/lib.rs` doesn't currently apply redaction to `FileRow` paths and modules when serializing the output. Looking at `write_export_cyclonedx`, it directly transforms `export.rows` without applying `redact_rows(..., args.redact)`.

Let's check `write_export_cyclonedx`.
