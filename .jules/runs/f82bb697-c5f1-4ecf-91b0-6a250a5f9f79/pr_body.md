## Description

This PR improves executable documentation testing (the "Docs as tests" principle) by adding test coverage for CLI recipes that were previously untested, particularly for `tokmd context` and `tokmd gate`. It ensures these public CLI interface recipes do not silently drift.

## Changes

* Added tests for `tokmd context` variants:
  * `--budget 128k`
  * `--mode bundle`
  * `--strategy spread`
  * `--compress`
  * `--mode json`
* Added tests for `tokmd gate` variants:
  * Default behavior
  * `--format json`
  * `--fail-fast`

All new tests exactly match documented recipes in `docs/reference-cli.md`.
