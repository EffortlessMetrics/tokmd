with open("crates/tokmd/tests/determinism_hardening_w51.rs", "r") as f:
    content = f.read()

old = """    fn check_no_backslash_paths(v: &Value, path: &str) {
        match v {
            Value::String(s)
                if !path.contains("version")
                    && !path.contains("hash")
                    && !path.contains("blake3")
                    && !path.contains("integrity")
                    && s.contains('\\\\')
                    && s.contains(std::path::MAIN_SEPARATOR) =>
            {
                // Only flag if it looks like a file path
                if s.contains(".rs") || s.contains(".js") || s.contains(".md") {
                    panic!("backslash in path-like string at {path}: {s}");
                }
            }
            Value::String(_) => {}"""

new = """    fn check_no_backslash_paths(v: &Value, path: &str) {
        match v {
            Value::String(s)
                if !path.contains("version")
                    && !path.contains("hash")
                    && !path.contains("blake3")
                    && !path.contains("integrity")
                    && s.contains('\\\\')
                    && s.contains(std::path::MAIN_SEPARATOR)
                    && (s.contains(".rs") || s.contains(".js") || s.contains(".md")) =>
            {
                panic!("backslash in path-like string at {path}: {s}");
            }
            Value::String(_) => {}"""

content = content.replace(old, new)
with open("crates/tokmd/tests/determinism_hardening_w51.rs", "w") as f:
    f.write(content)
