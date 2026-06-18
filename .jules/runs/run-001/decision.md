# Option A (recommended)
Modify `clean_path` in `tokmd-format` and `normalize_path` in `tokmd-model` (as well as `normalize_scan_input` in `scan_args`) to correctly traverse and collapse `..` path segments.
- Fits the repo and shard by fixing deterministic hash generation to protect structural layout, exactly as required by the instruction about path redaction and normalization.
- Trade-offs: Increases path normalization logic complexity slightly but solves real vulnerabilities in directory leak potential and makes `fuzz_scan_args` properties more bulletproof.

# Option B
Only strip `..` mechanically via regex replacement or `replace("../", "")`.
- Choose it when complexity needs to be minimized entirely.
- Trade-offs: Fails when `foo/bar/../../baz` has nested traversal. The traversal logic is more sound.
