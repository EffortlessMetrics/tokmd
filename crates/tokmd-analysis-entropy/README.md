# tokmd-analysis-entropy

Entropy profiling helpers for tokmd analysis receipts. This crate samples file
content to identify potential secrets or suspicious blobs.

## API

- `build_entropy_report` — compute an `EntropyReport` for a repository.
