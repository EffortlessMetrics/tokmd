# No dependency hygiene drift in core pipeline

During investigation of `crates/tokmd-types`, `crates/tokmd-scan`, `crates/tokmd-model`, and `crates/tokmd-format`, no actionable dependency removal or feature tightening targets were found.

The dependencies are already minimized and well-aligned with their requirements.
`cargo machete` and `cargo deny` pass without issue.
