# Decision

## Option A (recommended)
Add BDD coverage for the `Health` preset focusing on its derivation of TODO metrics, validating that its end-to-end integration with the content module correctly populates metrics when files are scanned.
- Matches Specsmith's mission to improve scenario coverage.
- Anchors behavior in a clear, narrative test case.
- Trade-offs: Minor increase in test suite duration due to temporary file I/O for `todo` scanning.

## Option B
Do not add the test, leaving the end-to-end `Health` preset TODO scanning untested at the integration level.
- Saves minor I/O cost in tests.
- Trade-offs: Leaves a gap in regression coverage around feature-gated behavior inside an important preset.

## Decision
Selected Option A. It explicitly fills an integration gap and locks in the expected behavior of the Health preset against real file scanning.
