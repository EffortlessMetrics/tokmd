use super::TokmdError;

impl From<anyhow::Error> for TokmdError {
    fn from(err: anyhow::Error) -> Self {
        from_anyhow(err)
    }
}

impl From<serde_json::Error> for TokmdError {
    fn from(err: serde_json::Error) -> Self {
        Self::invalid_json(err)
    }
}

impl From<std::io::Error> for TokmdError {
    fn from(err: std::io::Error) -> Self {
        Self::io_error(err)
    }
}

fn from_anyhow(err: anyhow::Error) -> TokmdError {
    let chain: Vec<String> = err.chain().map(|e| e.to_string()).collect();
    let primary = chain.first().cloned().unwrap_or_else(|| err.to_string());
    let haystack = chain.join(" | ").to_ascii_lowercase();

    if let Some(path) = extract_path_not_found(&chain) {
        return TokmdError::path_not_found_with_suggestions(&path);
    }

    if is_bounded_path_violation(&haystack) {
        return TokmdError::invalid_path(primary);
    }

    TokmdError::internal(primary)
}

fn extract_path_not_found(chain: &[String]) -> Option<String> {
    for message in chain {
        if let Some((_, path)) = message.split_once("Path not found: ") {
            return Some(path.trim().to_string());
        }
    }
    None
}

fn is_bounded_path_violation(haystack: &str) -> bool {
    haystack.contains("scan root must not be empty")
        || haystack.contains("bounded path must not be empty")
        || haystack.contains("bounded path must be relative")
        || haystack.contains("bounded path must not contain parent traversal")
        || haystack.contains("bounded path escapes scan root")
}
