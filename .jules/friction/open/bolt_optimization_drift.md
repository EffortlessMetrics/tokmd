## Drift during Bolt optimization

When applying `.to_owned()` over `.to_string()` in `tokmd-model` and `tokmd-format` to eliminate `Display` trait overhead, the branch drifted from `main` and inadvertently reverted shipped redaction behavior and older test/doc surfaces. The PR was closed as obsolete.
