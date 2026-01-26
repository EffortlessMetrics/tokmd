#![no_main]
use libfuzzer_sys::fuzz_target;
use tokmd_model::module_key;

fuzz_target!(|data: &[u8]| {
    if let Ok(s) = std::str::from_utf8(data) {
        // We fuzz the path.
        // We pick reasonable defaults for roots and depth to exercise logic.
        let roots = vec!["crates".to_string(), "packages".to_string()];
        let _ = module_key(s, &roots, 2);

        // Try other depths
        let _ = module_key(s, &roots, 1);
        let _ = module_key(s, &roots, 10);
    }
});
