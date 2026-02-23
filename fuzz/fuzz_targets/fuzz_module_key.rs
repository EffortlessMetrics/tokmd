#![no_main]
use libfuzzer_sys::fuzz_target;
use tokmd_module_key::module_key;

/// Max input size to prevent pathological parse times
const MAX_INPUT_SIZE: usize = 4 * 1024; // 4KB for path strings

fuzz_target!(|data: &[u8]| {
    if data.len() > MAX_INPUT_SIZE {
        return;
    }
    if let Ok(s) = std::str::from_utf8(data) {
        // We fuzz the path.
        // We pick reasonable defaults for roots and depth to exercise logic.
        let roots = vec!["crates".to_string(), "packages".to_string()];

        let key2 = module_key(s, &roots, 2);
        let key1 = module_key(s, &roots, 1);
        let key10 = module_key(s, &roots, 10);

        // Invariant: Output never contains backslashes (always forward slashes)
        assert!(
            !key2.contains('\\'),
            "module_key output must not contain backslashes: {key2:?}"
        );
        assert!(
            !key1.contains('\\'),
            "module_key output must not contain backslashes: {key1:?}"
        );
        assert!(
            !key10.contains('\\'),
            "module_key output must not contain backslashes: {key10:?}"
        );

        // Invariant: module key is always non-empty (at minimum "(root)")
        assert!(!key2.is_empty(), "module_key output must not be empty");
        assert!(!key1.is_empty(), "module_key output must not be empty");
        assert!(!key10.is_empty(), "module_key output must not be empty");

        // Invariant: determinism - same inputs produce same outputs
        assert_eq!(
            module_key(s, &roots, 1),
            key1,
            "module_key must be deterministic for depth=1"
        );
        assert_eq!(
            module_key(s, &roots, 2),
            key2,
            "module_key must be deterministic for depth=2"
        );
        assert_eq!(
            module_key(s, &roots, 10),
            key10,
            "module_key must be deterministic for depth=10"
        );

        // Invariant: if path starts with a root, module key should start with that root
        // (after normalization)
        let normalized = s.replace('\\', "/");
        let trimmed = normalized.trim_start_matches("./").trim_start_matches('/');
        for root in &roots {
            if trimmed.starts_with(&format!("{root}/")) && key2 != "(root)" {
                assert!(
                    key2.starts_with(root),
                    "module_key for path starting with '{root}/' should start with '{root}': got {key2:?}"
                );
            }
        }
    }
});
