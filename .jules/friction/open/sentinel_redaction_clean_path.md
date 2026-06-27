# False alarm on `clean_path` normalization

We explored an assumption that `clean_path` inside `tokmd-format/src/redact/mod.rs` would fail to clean overlapping segments like `/././` because of how `str::replace` works. However, the existing implementation uses `while normalized.contains("/./")`, which correctly handles the overlapping segments (after one replacement, the string still contains `/./`, so it loops again). The existing boundary hardening is correct.
