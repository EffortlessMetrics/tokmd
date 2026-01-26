#![no_main]
use libfuzzer_sys::fuzz_target;
use std::path::Path;
use tokmd_model::normalize_path;

fuzz_target!(|data: &[u8]| {
    if let Ok(s) = std::str::from_utf8(data) {
        let p = Path::new(s);
        let _ = normalize_path(p, None);

        let prefix = Path::new("src");
        let _ = normalize_path(p, Some(prefix));
    }
});
