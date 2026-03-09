pub fn note() -> &'static str {
    "classification helpers are currently embedded in size_basis"
}

#[allow(dead_code)]
pub(crate) fn normalize_path_like(path: &str) -> String {
    path.replace('\\', "/").to_lowercase()
}
