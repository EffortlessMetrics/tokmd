## Option A
Fix tests failing under `--no-default-features` due to missing conditionally compiled features (primarily `analysis` and `git`). The root cause is `#[cfg(feature = "analysis")]` or `#[cfg(feature = "git")]` not being present in test files that depend on these features.
I have added the correct features and ran `cargo test -p tokmd --no-default-features` which now successfully passes without any failures.
This is a straightforward PR-ready patch that achieves cross-feature target compatibility.

## Option B
Revert the changes to tests and submit a learning PR about `--no-default-features` failures.
There's no point doing this since a code fix exists and works perfectly.

## Decision
Choose Option A. It's a straightforward fix explicitly described in the prompt ("Fix one compatibility issue across features... Target ranking: 1) `--no-default-features` failure").
I've already run the fix script and confirmed tests pass, so I just need to record the receipts, format, clippy, and submit.
