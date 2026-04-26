#[allow(dead_code)]
pub(crate) fn normalize_path_like(path: &str) -> String {
    path.replace('\\', "/").to_lowercase()
}
