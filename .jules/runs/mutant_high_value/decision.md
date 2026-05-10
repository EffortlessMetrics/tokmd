# Decision

## Option A
Strengthen the tests for `tokmd-model` aggregation by explicitly ensuring that `bytes` and `tokens` logic metrics are accumulated properly for `FileKind::Child` in both `Collapse` and `Separate` child node modes within `create_lang_report_from_rows`. This satisfies the mutant and explicitly closes the behavioral test gap around report generation.

## Option B
Refactor `aggregate.rs` to loop differently, or drop mutation tests altogether.

## Conclusion
Option A is chosen because it strengthens assertions around high-value aggregation logic surfaces. The patch clearly addresses missing `+=` implementations in model building tests.
