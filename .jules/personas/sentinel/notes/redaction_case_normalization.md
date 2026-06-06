# Sentinel Note: Redaction Case Normalization

During a review of the redaction mechanism in `tokmd-format` (`src/redact/extensions.rs`), we confirmed that extensions containing mixed casing (e.g. `file.JSON`) are safely normalized to lowercase (e.g. `.json`). This prevents any entropy or hidden metadata from crossing the trust boundary via arbitrary casing tricks. No patch was required as the current state already implements the correct boundary logic.
