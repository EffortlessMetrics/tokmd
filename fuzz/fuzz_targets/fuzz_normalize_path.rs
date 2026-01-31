#![no_main]
use libfuzzer_sys::fuzz_target;
use std::path::Path;
use tokmd_model::normalize_path;

/// Max input size to prevent pathological parse times
const MAX_INPUT_SIZE: usize = 4 * 1024; // 4KB for path strings

fuzz_target!(|data: &[u8]| {
    if data.len() > MAX_INPUT_SIZE {
        return;
    }
    if let Ok(s) = std::str::from_utf8(data) {
        let p = Path::new(s);
        let result = normalize_path(p, None);

        // Invariant: Output never contains backslashes (always forward slashes)
        assert!(
            !result.contains('\\'),
            "normalize_path output must not contain backslashes: {result:?}"
        );

        // Invariant: Output should not start with "./" (gets stripped)
        assert!(
            !result.starts_with("./"),
            "normalize_path output must not start with './': {result:?}"
        );

        // Invariant: Output length is bounded by input length
        // (we only strip characters, never add)
        assert!(
            result.len() <= s.len(),
            "normalize_path output ({}) must not exceed input length ({})",
            result.len(),
            s.len()
        );

        // Test with prefix stripping
        let prefix = Path::new("src");
        let result_with_prefix = normalize_path(p, Some(prefix));

        // Same invariants apply with prefix
        assert!(
            !result_with_prefix.contains('\\'),
            "normalize_path output must not contain backslashes: {result_with_prefix:?}"
        );
        assert!(
            !result_with_prefix.starts_with("./"),
            "normalize_path output must not start with './': {result_with_prefix:?}"
        );
        // With prefix stripping, output should be <= input length
        assert!(
            result_with_prefix.len() <= s.len(),
            "normalize_path output ({}) must not exceed input length ({})",
            result_with_prefix.len(),
            s.len()
        );
    }
});
