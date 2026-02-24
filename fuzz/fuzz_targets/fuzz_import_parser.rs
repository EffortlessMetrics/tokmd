#![no_main]

use libfuzzer_sys::fuzz_target;
use tokmd_analysis_imports::{normalize_import_target, parse_imports, supports_language};

fuzz_target!(|data: &[u8]| {
    let text = String::from_utf8_lossy(data);
    let mut parts = text.splitn(2, '\n');
    let lang = parts.next().unwrap_or_default();
    let body = parts.next().unwrap_or_default();

    let lines: Vec<&str> = body.lines().take(512).collect();
    let imports = parse_imports(lang, &lines);

    let _ = supports_language(lang);
    for import in imports {
        let normalized = normalize_import_target(&import);
        let _ = normalize_import_target(&normalized);
    }
});
